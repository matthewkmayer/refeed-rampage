use dynomite::{
    dynamodb::{
        AttributeDefinition, CreateTableInput, DynamoDb, DynamoDbClient, KeySchemaElement,
        ProvisionedThroughput,
    },
    retry::Policy,
    Retries,
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

type Db = Arc<Mutex<BTreeMap<String, Meal>>>;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    info!("Firing up");
    // a bunch from https://github.com/seanmonstar/warp/blob/master/examples/todos.rs
    let fake_db = Arc::new(Mutex::new(BTreeMap::new()));
    prepopulate_db(fake_db.clone()).await;

    let cors = warp::cors()
        .allow_origin("http://localhost:8080")
        .allow_origin("http://127.0.0.1:8080")
        .allow_methods(vec!["GET", "POST", "DELETE", "PUT"])
        .allow_headers(vec!["content-type"]);

    let routes = meal_filters(fake_db.clone())
        .with(&cors)
        .with(warp::log("meals"));

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

fn meal_filters(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    a_meal_filter(db.clone())
        .or(all_meal_filter(db.clone()))
        .or(meal_create(db.clone()))
        .or(meal_delete(db.clone()))
        .or(meal_update(db))
}

fn a_meal_filter(
    ds: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("meals" / Uuid)
        .and(warp::get())
        .and(with_db(ds))
        .and_then(specific_meal)
}

fn all_meal_filter(
    ds: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("meals")
        .and(warp::get())
        .and(with_db(ds))
        .and_then(all_meals)
}

fn meal_create(ds: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("meals")
        .and(warp::post())
        .and(json_body())
        .and(with_db(ds))
        .and_then(create_meal)
}

fn meal_delete(ds: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("meals" / Uuid)
        .and(warp::delete())
        .and(with_db(ds))
        .and_then(delete_meal)
}

fn meal_update(ds: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("meals" / Uuid)
        .and(warp::put())
        .and(json_body())
        .and(with_db(ds))
        .and_then(update_meal)
}

// curl -i -X DELETE http://localhost:3030/meals/1
async fn delete_meal(i: Uuid, db: Db) -> Result<impl warp::Reply, Infallible> {
    let mut fake_db = db.lock().await;
    if fake_db.contains_key(&i.to_string()) {
        fake_db.remove(&i.to_string());
    } else {
        return Ok(StatusCode::BAD_REQUEST);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn specific_meal(i: Uuid, db: Db) -> Result<impl warp::Reply, Infallible> {
    let fake_db = db.lock().await;
    log::info!("ds is {:?}", db);
    Ok(warp::reply::json(
        &fake_db.get_key_value(&i.to_string()).unwrap().1,
    ))
}

async fn update_meal(i: Uuid, create: Meal, db: Db) -> Result<impl warp::Reply, Infallible> {
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

async fn all_meals(db: Db) -> Result<impl warp::Reply, Infallible> {
    let fake_db = db.lock().await;
    let a: Vec<&Meal> = fake_db.iter().map(|x| x.1).collect();

    Ok(warp::reply::json(&a))
}

// should work with curl -i -X POST -H "content-type: application/json" -d '{"name":"Wings","id":3,"description":"mmm"}'  http://127.0.0.1:3030/meals/
pub async fn create_meal(create: Meal, db: Db) -> Result<impl warp::Reply, Infallible> {
    log::debug!("create_meal: {:?}", create);

    let mut d = db.lock().await;
    let new_id: Uuid;

    if !d.contains_key(&create.id.to_string()) {
        new_id = Uuid::new_v4();
        d.insert(
            new_id.to_string(),
            Meal {
                id: new_id,
                name: create.name,
                photos: None,
                description: create.description,
            },
        );
    } else {
        let r = warp::reply::json(&());
        return Ok(warp::reply::with_status(r, StatusCode::BAD_REQUEST));
    }

    // casting between usize and i32 would go away when a real backend is used
    let json = warp::reply::json(d.get(&new_id.to_string()).unwrap());
    Ok(warp::reply::with_status(json, StatusCode::CREATED))
}

fn with_db(db: Db) -> impl Filter<Extract = (Db,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

fn json_body() -> impl Filter<Extract = (Meal,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

async fn prepopulate_db(db: Db) {
    let mut d = db.lock().await;
    let mut id = Uuid::parse_str("936DA01F9ABD4d9d80C702AF85C822A8").unwrap();
    d.insert(
        id.to_string(),
        Meal {
            id,
            name: "Pizza".to_string(),
            photos: None,
            description: "Delicious pizza".to_string(),
        },
    );
    id = Uuid::parse_str("f11b1c5e-d6d8-4dce-8a9d-9e05d870b881").unwrap();
    d.insert(
        id.to_string(),
        Meal {
            id,
            name: "Burritos".to_string(),
            photos: None,
            description: "Amazing burritos".to_string(),
        },
    );
    // ---------------------------------------------------
    // Dynamo bits:

    // create rusoto client
    let client = DynamoDbClient::new(Region::Custom {
        name: "us-east-1".into(),
        endpoint: "http://localhost:8000".into(),
    })
    .with_retries(Policy::default());
    let table_name = "books".to_string();
    let create_table_req = client.create_table(CreateTableInput {
        table_name,
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
    log::debug!("Gonna run a future");
    let f = create_table_req.sync();
    log::debug!("it ran: {:?}", f);
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Meal {
    name: String,
    id: Uuid,
    photos: Option<String>,
    description: String,
}
