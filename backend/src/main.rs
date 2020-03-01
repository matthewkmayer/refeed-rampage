use dynomite::{
    dynamodb::{
        AttributeDefinition, CreateTableInput, DynamoDb, DynamoDbClient, GetItemInput,
        KeySchemaElement, ProvisionedThroughput, PutItemInput,
    },
    retry::Policy,
    FromAttributes, Item, Retries,
};
use rusoto_core::Region;
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use warp::http::StatusCode;
use warp::Filter;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    info!("Firing up");
    // a bunch from https://github.com/seanmonstar/warp/blob/master/examples/todos.rs
    prepopulate_db().await;

    let cors = warp::cors()
        .allow_origin("http://localhost:8080")
        .allow_origin("http://127.0.0.1:8080")
        .allow_methods(vec!["GET", "POST", "DELETE", "PUT"])
        .allow_headers(vec!["content-type"]);

    let routes = meal_filters().with(&cors).with(warp::log("meals"));

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

fn meal_filters() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    a_meal_filter()
        .or(all_meal_filter())
        .or(meal_create())
        .or(meal_delete())
        .or(meal_update())
}

fn a_meal_filter() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("meals" / Uuid)
        .and(warp::get())
        .and_then(specific_meal)
}

fn all_meal_filter() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("meals").and(warp::get()).and_then(all_meals)
}

fn meal_create() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("meals")
        .and(warp::post())
        .and(json_body())
        .and_then(create_meal)
}

fn meal_delete() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("meals" / Uuid)
        .and(warp::delete())
        .and_then(delete_meal)
}

fn meal_update() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("meals" / Uuid)
        .and(warp::put())
        .and(json_body())
        .and_then(update_meal)
}

// curl -i -X DELETE http://localhost:3030/meals/1
async fn delete_meal(i: Uuid) -> Result<impl warp::Reply, Infallible> {
    let mut fake_db = db.lock().await;
    if fake_db.contains_key(&i.to_string()) {
        fake_db.remove(&i.to_string());
    } else {
        return Ok(StatusCode::BAD_REQUEST);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn specific_meal(i: Uuid) -> Result<impl warp::Reply, Infallible> {
    // we should pass one of these around instead of recreating it
    let client = DynamoDbClient::new(Region::Custom {
        name: "us-east-1".into(),
        endpoint: "http://localhost:8000".into(),
    })
    .with_retries(Policy::default());
    let m = Meal {
        id: i,
        ..Default::default()
    };
    let item = client
        .get_item(GetItemInput {
            table_name: "meals".to_string(),
            key: m.key(),
            ..GetItemInput::default()
        })
        .sync()
        .map(|result| result.item.map(Meal::from_attrs));
    match item {
        Ok(item_found) => {
            info!("success, item be all {:?}", item_found);
            let r = warp::reply::json(&item_found.unwrap().unwrap());
            return Ok(warp::reply::with_status(r, StatusCode::OK));
        }
        Err(e) => info!("It blew up :( {:?}", e),
    }
    let r = warp::reply::json(&());
    Ok(warp::reply::with_status(r, StatusCode::BAD_REQUEST))
}

async fn update_meal(i: Uuid, create: Meal) -> Result<impl warp::Reply, Infallible> {
    let mut fake_db = db.lock().await;
    if fake_db.contains_key(&i.to_string()) {
        if let Some(a) = fake_db.get_mut(&i.to_string()) {
            *a = create
        }
    } else {
        let r = warp::reply::json(&());
        return Ok(warp::reply::with_status(r, StatusCode::BAD_REQUEST));
    }
    let json = warp::reply::json(fake_db.get(&i.to_string()).unwrap());
    Ok(warp::reply::with_status(json, StatusCode::ACCEPTED))
}

async fn all_meals() -> Result<impl warp::Reply, Infallible> {
    let fake_db = db.lock().await;
    let a: Vec<&Meal> = fake_db.iter().map(|x| x.1).collect();

    Ok(warp::reply::json(&a))
}

// should work with curl -i -X POST -H "content-type: application/json" -d '{"name":"Wings","id":3,"description":"mmm"}'  http://127.0.0.1:3030/meals/
pub async fn create_meal(create: Meal) -> Result<impl warp::Reply, Infallible> {
    log::debug!("create_meal: {:?}", create);

    let client = DynamoDbClient::new(Region::Custom {
        name: "us-east-1".into(),
        endpoint: "http://localhost:8000".into(),
    })
    .with_retries(Policy::default());

    let newone = Meal {
        id: Uuid::new_v4(),
        ..create.clone() // remove clone after the dump other backing data store
    };

    let d_result = client
        .put_item(PutItemInput {
            table_name: "meals".to_string(),
            item: newone.into(),
            ..PutItemInput::default()
        })
        .sync();
    match d_result {
        Ok(_) => {
            info!("aww yiss added it");
            let r = warp::reply::json(&newone);
            return Ok(warp::reply::with_status(r, StatusCode::CREATED));
        }
        Err(e) => {
            info!("blew up: {:?}", e);
            let r = warp::reply::json(&());
            return Ok(warp::reply::with_status(r, StatusCode::BAD_REQUEST));
        }
    }
}

fn json_body() -> impl Filter<Extract = (Meal,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

async fn prepopulate_db() {
    // ---------------------------------------------------
    // Dynamo bits:

    // create rusoto client
    let client = DynamoDbClient::new(Region::Custom {
        name: "us-east-1".into(),
        endpoint: "http://localhost:8000".into(),
    })
    .with_retries(Policy::default());
    let table_name = "meals".to_string();
    let create_table_req = client.create_table(CreateTableInput {
        table_name: table_name.clone(),
        key_schema: vec![KeySchemaElement {
            attribute_name: "id".into(),
            key_type: "HASH".into(),
        }],
        attribute_definitions: vec![AttributeDefinition {
            attribute_name: "id".into(),
            attribute_type: "S".into(),
        }],
        provisioned_throughput: Some(ProvisionedThroughput {
            read_capacity_units: 1,
            write_capacity_units: 1,
        }),
        ..CreateTableInput::default()
    });
    debug!("Gonna run a future");
    let f = create_table_req.sync();
    match f {
        Ok(_) => debug!("All good making table"),
        Err(e) => {
            debug!("Issue creating table: {:?}", e);
            if !e.to_string().contains("preexisting table") {
                panic!("Ran into an issue unrelated to table pre existing");
            }
        }
    }

    let id = Uuid::parse_str("f11b1c5e-d6d8-4dce-8a9d-9e05d870b881").unwrap();
    let mut m = Meal {
        id,
        name: "Burritos".to_string(),
        photos: None,
        description: "Amazing burritos".to_string(),
    };

    let _ = client
        .put_item(PutItemInput {
            table_name: table_name.clone(),
            item: m.clone().into(),
            ..PutItemInput::default()
        })
        .sync();

    m.id = Uuid::parse_str("936DA01F9ABD4d9d80C702AF85C822A8").unwrap();
    m.name = "Pizza".to_string();
    m.description = "Delicious pizza".to_string();

    let _ = client
        .put_item(PutItemInput {
            table_name,
            item: m.into(),
            ..PutItemInput::default()
        })
        .sync();
}

#[derive(Deserialize, Serialize, Debug, Item, Clone, Default)]
pub struct Meal {
    #[dynomite(rename = "mealName")]
    name: String,
    #[dynomite(partition_key)]
    id: Uuid,
    photos: Option<String>,
    description: String,
}
