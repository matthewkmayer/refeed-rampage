use dynomite::{
    dynamodb::{
        AttributeDefinition, CreateTableInput, DeleteItemInput, DynamoDb, DynamoDbClient,
        GetItemInput, KeySchemaElement, ProvisionedThroughput, PutItemInput, ScanInput,
    },
    retry::Policy,
    FromAttributes, Item, Retries,
};
use rusoto_core::Region;
use serde_derive::{Deserialize, Serialize};
use std::convert::Infallible;
use uuid::Uuid;
use warp::http::StatusCode;
use warp::Filter;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

static DYNAMODB_LOC: &str = include_str!("ddb_loc.txt");

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    info!("Firing up");
    // a bunch from https://github.com/seanmonstar/warp/blob/master/examples/todos.rs
    prepopulate_db().await;

    let cors = warp::cors()
        .allow_origin("http://localhost:8080")
        .allow_origin("http://127.0.0.1:8080")
        .allow_origin("https://rampage.screaming3d.com")
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
        .or(status_filter())
}

fn status_filter() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("health").and(warp::get()).and_then(healthy)
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
    let client = get_dynamodb_client();
    let m = Meal {
        id: i,
        ..Default::default()
    };

    let del = client
        .delete_item(DeleteItemInput {
            table_name: "meals".to_string(),
            key: m.key(),
            ..DeleteItemInput::default()
        })
        .sync();

    match del {
        Ok(deleted_item) => {
            info!("item got deleted {:?}", deleted_item);
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            info!("item couldn't be deleted: {:?}", e);
            Ok(StatusCode::BAD_REQUEST)
        }
    }
}

async fn healthy() -> Result<impl warp::Reply, Infallible> {
    let h = Health {
        healthy: true,
        version: "v.0.0.1-whatever".to_string(), // should have git hash in here later
    };
    let r = warp::reply::json(&h);
    Ok(warp::reply::with_status(r, StatusCode::OK))
}

// handle local vs "real" dynamodb
fn get_dynamodb_client() -> dynomite::retry::RetryingDynamoDb<DynamoDbClient> {
    match DYNAMODB_LOC.len() {
        0 => {
            info!("Using real Dynamodb");
            DynamoDbClient::new(Region::UsWest2).with_retries(Policy::default())
        }
        _ => {
            info!("Using local Dynamodb");
            DynamoDbClient::new(Region::Custom {
                name: "us-east-1".into(), // local testing only
                endpoint: DYNAMODB_LOC.into(),
            })
            .with_retries(Policy::default())
        }
    }
}

async fn specific_meal(i: Uuid) -> Result<impl warp::Reply, Infallible> {
    // we should pass one of these around instead of recreating it
    let client = get_dynamodb_client();
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

async fn update_meal(_id: Uuid, create: Meal) -> Result<impl warp::Reply, Infallible> {
    // make sure _id matches create.id
    let client = get_dynamodb_client();

    let d_result = client
        .put_item(PutItemInput {
            table_name: "meals".to_string(),
            item: create.clone().into(),
            ..PutItemInput::default()
        })
        .sync();
    match d_result {
        Ok(_) => {
            info!("aww yiss added it");
            let r = warp::reply::json(&create);
            Ok(warp::reply::with_status(r, StatusCode::ACCEPTED))
        }
        Err(e) => {
            info!("blew up: {:?}", e);
            let r = warp::reply::json(&());
            Ok(warp::reply::with_status(r, StatusCode::BAD_REQUEST))
        }
    }
}

// wow it's... not great
async fn all_meals() -> Result<impl warp::Reply, Infallible> {
    let client = get_dynamodb_client();

    let scan_all_things = client
        .scan(ScanInput {
            table_name: "meals".to_string(),
            ..ScanInput::default()
        })
        .sync();

    match scan_all_things {
        Ok(s) => {
            let doot: Vec<Meal> = s
                .items
                .unwrap()
                .iter()
                .map(|result| Meal::from_attrs(result.clone()).unwrap())
                .collect();
            let r = warp::reply::json(&doot);
            Ok(warp::reply::with_status(r, StatusCode::OK))
        }
        Err(e) => {
            info!("nope: {:?}", e);
            let r = warp::reply::json(&ErrorResp {
                error: e.to_string(),
            });
            Ok(warp::reply::with_status(
                r,
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

// should work with curl -i -X POST -H "content-type: application/json" -d '{"name":"Wings","id":3,"description":"mmm"}'  http://127.0.0.1:3030/meals/
pub async fn create_meal(create: Meal) -> Result<impl warp::Reply, Infallible> {
    log::debug!("create_meal: {:?}", create);

    let client = get_dynamodb_client();

    let newone = Meal {
        id: Uuid::new_v4(),
        ..create
    };

    let d_result = client
        .put_item(PutItemInput {
            table_name: "meals".to_string(),
            item: newone.clone().into(),
            ..PutItemInput::default()
        })
        .sync();
    match d_result {
        Ok(_) => {
            info!("aww yiss added it");
            let r = warp::reply::json(&newone);
            Ok(warp::reply::with_status(r, StatusCode::CREATED))
        }
        Err(e) => {
            info!("blew up: {:?}", e);
            let r = warp::reply::json(&ErrorResp {
                error: e.to_string(),
            });
            Ok(warp::reply::with_status(
                r,
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

fn json_body() -> impl Filter<Extract = (Meal,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

async fn is_db_avail() -> bool {
    let client = get_dynamodb_client();
    let table_name = "meals".to_string();
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
    debug!("Gonna run a future");
    let f = create_table_req.sync();
    match f {
        Ok(_) => {
            debug!("All good making table");
            true
        }
        Err(e) => {
            // local one may not be ready yet, wait and retry:
            if !e.to_string().contains("preexisting table") {
                return false;
            }
            true
        }
    }
}

async fn prepopulate_db() {
    let mut attempts = 0;
    loop {
        debug!("Waiting for the db to be available");
        if is_db_avail().await {
            debug!("DB is available");
            break;
        }
        if attempts > 10 {
            debug!("DB is not available after 10 attempts, we're out");
            panic!("Stopped waiting for the DB to become available");
        }
        attempts += 1;
        debug!("sleeping for a minute and retrying");
        std::thread::sleep(std::time::Duration::from_millis(5_000));
    }
    let client = get_dynamodb_client();
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
            debug!("Issue creating table: {:?}. Forging ahead anyways.", e);
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
pub struct ErrorResp {
    error: String,
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

#[derive(Deserialize, Serialize, Debug, Item, Clone, Default)]
struct Health {
    healthy: bool,
    version: String,
}
