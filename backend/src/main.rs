mod s3_interactions;

use dynomite::{
    dynamodb::{
        AttributeDefinition, CreateTableInput, DeleteItemInput, DynamoDb, DynamoDbClient,
        GetItemInput, KeySchemaElement, ProvisionedThroughput, PutItemInput, ScanInput,
    },
    retry::Policy,
    FromAttributes, Item, Retries,
};

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rusoto_core::{credential::ProfileProvider, HttpClient, Region};
use serde_derive::{Deserialize, Serialize};
use shared::Meal;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use uuid::Uuid;
use warp::http::StatusCode;
use warp::Filter;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

static BUCKET_NAME: &str = "refeed-rampage";
static DYNAMODB_LOC: &str = include_str!("ddb_loc.txt");
static S3_LOC: &str = include_str!("s3_loc.txt");
static SECUREPW: &str = include_str!("password.txt"); // this is for testing purposes
static JWT_SECRET: &str = include_str!("jwtsecret.txt");
static GITBITS: &str = include_str!("gitbits.txt");

// store jwts in memory for now
pub type JwtDb = Arc<Mutex<HashMap<String, i32>>>;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let version_txt = match GITBITS.len() {
        0 => "dev",
        _ => GITBITS,
    };
    info!("Firing up. Version {}.", version_txt);
    let c = get_dynamodb_client();
    // a bunch from https://github.com/seanmonstar/warp/blob/master/examples/todos.rs
    prepopulate_db(c.clone()).await;

    // do a connectivity check - we need the bucket
    s3_interactions::create_bucket_if_needed(S3_LOC, BUCKET_NAME).await;

    let jwtdb: JwtDb = Arc::new(Mutex::new(HashMap::new()));

    let cors = warp::cors()
        .allow_origin("http://localhost:8080")
        .allow_origin("http://127.0.0.1:8080")
        .allow_origin("http://refeed.local:8080")
        .allow_origin("https://rampage.screaming3d.com")
        .allow_methods(vec!["GET", "POST", "DELETE", "PUT"])
        .allow_headers(vec!["content-type", "Authorization"]);

    let routes = meal_filters(jwtdb, c)
        .with(&cors)
        .with(warp::log("backend"));

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

fn meal_filters(
    jwtdb: JwtDb,
    ddb_client: dynomite::retry::RetryingDynamoDb<DynamoDbClient>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    a_meal_filter(ddb_client.clone())
        .or(all_meal_filter(ddb_client.clone()))
        .or(meal_create(ddb_client, jwtdb.clone()))
        .or(meal_delete(jwtdb.clone()))
        .or(meal_update(jwtdb.clone()))
        .or(status_filter())
        .or(login_filter(jwtdb))
        .or(unauthed()) // if something rejected it, toss an unauthorized at it
}

fn unauthed() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::any().and_then(unauthed_resp)
}

fn with_jwtdb(
    db: JwtDb,
) -> impl Filter<Extract = (JwtDb,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

fn with_ddb(
    ddb_client: dynomite::retry::RetryingDynamoDb<DynamoDbClient>,
) -> impl Filter<
    Extract = (dynomite::retry::RetryingDynamoDb<DynamoDbClient>,),
    Error = std::convert::Infallible,
> + Clone {
    warp::any().map(move || ddb_client.clone())
}

fn login_filter(
    jwtdb: JwtDb,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("login")
        .and(warp::post())
        .and(with_jwtdb(jwtdb))
        .and(json_login_body())
        .and_then(login)
}

fn status_filter() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("health").and(warp::get()).and_then(healthy)
}

fn a_meal_filter(
    ddb_client: dynomite::retry::RetryingDynamoDb<DynamoDbClient>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("meals" / Uuid)
        .and(warp::get())
        .and(with_ddb(ddb_client))
        .and_then(specific_meal)
}

fn all_meal_filter(
    ddb_client: dynomite::retry::RetryingDynamoDb<DynamoDbClient>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("meals")
        .and(warp::get())
        .and(with_ddb(ddb_client))
        .and_then(all_meals)
}

fn meal_create(
    ddb_client: dynomite::retry::RetryingDynamoDb<DynamoDbClient>,
    jwtdb: JwtDb,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("meals")
        .and(warp::post())
        .and(warp::header::<String>("Authorization"))
        .and(with_jwtdb(jwtdb))
        .and_then(|auth: String, jwtdb: JwtDb| async move {
            if is_authed(auth, jwtdb).await {
                Ok(())
            } else {
                Err(warp::reject::not_found())
            }
        })
        .and(json_meal_body())
        .and(with_ddb(ddb_client))
        .and_then(create_meal)
}

fn meal_delete(
    jwtdb: JwtDb,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("meals" / Uuid)
        .and(warp::delete())
        .and(warp::header::<String>("Authorization"))
        .and(with_jwtdb(jwtdb))
        .and_then(|id: Uuid, auth: String, jwtdb: JwtDb| async move {
            if is_authed(auth, jwtdb).await {
                Ok(id)
            } else {
                Err(warp::reject::not_found())
            }
        })
        .and_then(delete_meal)
}

fn meal_update(
    jwtdb: JwtDb,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("meals" / Uuid)
        .and(warp::put())
        .and(warp::header::<String>("Authorization"))
        .and(with_jwtdb(jwtdb))
        .and_then(|id: Uuid, auth: String, jwtdb: JwtDb| async move {
            if is_authed(auth, jwtdb).await {
                Ok(id)
            } else {
                Err(warp::reject::not_found())
            }
        })
        .and(json_meal_body())
        .and_then(update_meal)
}

// curl -i -X DELETE http://localhost:3030/meals/1
async fn delete_meal(i: Uuid) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
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
        .await;

    match del {
        Ok(deleted_item) => {
            info!("item got deleted {:?}", deleted_item);
            Ok(Box::new(StatusCode::NO_CONTENT))
        }
        Err(e) => {
            info!("item couldn't be deleted: {:?}", e);
            Ok(Box::new(StatusCode::BAD_REQUEST))
        }
    }
}

async fn healthy() -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let version_txt = match GITBITS.len() {
        0 => "dev".to_string(),
        _ => GITBITS.to_string().replace('\n', ""),
    };
    let h = Health {
        healthy: true,
        version: version_txt,
    };
    let r = warp::reply::json(&h);
    Ok(Box::new(warp::reply::with_status(r, StatusCode::OK)))
}

// handle local vs "real" dynamodb
fn get_dynamodb_client() -> dynomite::retry::RetryingDynamoDb<DynamoDbClient> {
    // be nice to not have to do this all the time. Use lazy_static?
    match DYNAMODB_LOC.replace("\n", "").len() {
        0 => {
            info!("Using real Dynamodb with a new client");
            // use profile provider only
            let profile_creds =
                ProfileProvider::new().expect("Couldn't make new Profile credential provider");
            let http_client = HttpClient::new().expect("Couldn't make new HTTP client");
            DynamoDbClient::new_with(http_client, profile_creds, Region::UsWest2)
                .with_retries(Policy::default())
        }
        _ => {
            info!("Using local Dynamodb with a new client");
            DynamoDbClient::new(Region::Custom {
                name: "us-east-1".into(), // local testing only
                endpoint: DYNAMODB_LOC.into(),
            })
            .with_retries(Policy::default())
        }
    }
}

async fn specific_meal(
    i: Uuid,
    client: dynomite::retry::RetryingDynamoDb<DynamoDbClient>,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
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
        .await
        .map(|result| result.item.map(Meal::from_attrs));
    match item {
        Ok(item_found) => {
            info!("success, item be all {:?}", item_found);
            let r = warp::reply::json(&item_found.unwrap().unwrap());
            return Ok(Box::new(warp::reply::with_status(r, StatusCode::OK)));
        }
        Err(e) => info!("It blew up :( {:?}", e),
    }
    let r = warp::reply::json(&());
    Ok(Box::new(warp::reply::with_status(
        r,
        StatusCode::BAD_REQUEST,
    )))
}

async fn unauthed_resp() -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let r = warp::reply::json(&());
    Ok(Box::new(warp::reply::with_status(
        r,
        StatusCode::UNAUTHORIZED,
    )))
}

async fn update_meal(_id: Uuid, create: Meal) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    // make sure _id matches create.id
    let client = get_dynamodb_client();

    let d_result = client
        .put_item(PutItemInput {
            table_name: "meals".to_string(),
            item: create.clone().into(),
            ..PutItemInput::default()
        })
        .await;
    match d_result {
        Ok(_) => {
            let r = warp::reply::json(&create);
            Ok(Box::new(warp::reply::with_status(r, StatusCode::ACCEPTED)))
        }
        Err(e) => {
            info!("blew up: {:?}", e);
            let r = warp::reply::json(&());
            Ok(Box::new(warp::reply::with_status(
                r,
                StatusCode::BAD_REQUEST,
            )))
        }
    }
}

async fn all_meals(
    client: dynomite::retry::RetryingDynamoDb<DynamoDbClient>,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let scan_all_things = client
        .scan(ScanInput {
            table_name: "meals".to_string(),
            ..ScanInput::default()
        })
        .await;
    match scan_all_things {
        Ok(s) => {
            let doot: Vec<Meal> = s
                .items
                .unwrap()
                .iter()
                .map(|result| Meal::from_attrs(result.clone()).unwrap())
                .collect();
            let r = warp::reply::json(&doot);
            Ok(Box::new(warp::reply::with_status(r, StatusCode::OK)))
        }
        Err(e) => {
            info!("nope: {:?}", e);
            let r = warp::reply::json(&ErrorResp {
                error: e.to_string(),
            });
            Ok(Box::new(warp::reply::with_status(
                r,
                StatusCode::INTERNAL_SERVER_ERROR,
            )))
        }
    }
}

// should work with curl -i -X POST -H "content-type: application/json" -d '{"name":"Wings","id":3,"description":"mmm"}'  http://127.0.0.1:3030/meals/
pub async fn create_meal(
    _: (),
    create: Meal,
    client: dynomite::retry::RetryingDynamoDb<DynamoDbClient>,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    log::debug!("create_meal: {:?}", create);

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
        .await;
    match d_result {
        Ok(_) => {
            info!("aww yiss added it");
            let r = warp::reply::json(&newone);
            Ok(Box::new(warp::reply::with_status(r, StatusCode::CREATED)))
        }
        Err(e) => {
            info!("blew up: {:?}", e);
            let r = warp::reply::json(&ErrorResp {
                error: e.to_string(),
            });
            Ok(Box::new(warp::reply::with_status(
                r,
                StatusCode::INTERNAL_SERVER_ERROR,
            )))
        }
    }
}

// curl -i -X POST -d '{"user": "foo", "pw": "bar"}' -H "Content-type: application/json" localhost:3030/login
pub async fn login(jwtdb: JwtDb, login: Login) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    // why with the newlines?
    if login.user == "matthew" && login.pw == SECUREPW.replace('\n', "") {
        debug!("Successful login");
        // yeah should probably handle errors:
        let in_future = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 10_000_000; // forever-ish
        let claims = Claims {
            exp: in_future as u32,
            sub: login.user,
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(JWT_SECRET.as_ref()),
        )
        .unwrap(); // TODO: handle failure

        debug!("Made this jwt: {:?}", token);

        // store jwt to data store
        let mut jwts = jwtdb.lock().await;
        jwts.insert(token.clone(), 0); // should we use the value for something?
        debug!("inserted token into db: {:?}", token);

        // return jwt
        let resp = LoginResp { jwt: token };
        let r = warp::reply::json(&resp);

        let r2 = warp::reply::with_header(
            r,
            warp::http::header::SET_COOKIE,
            format!("rtoken=\"{}\"", resp.jwt),
        );
        Ok(Box::new(warp::reply::with_status(r2, StatusCode::OK)))
    } else {
        debug!("Incorrect username/pw");
        // we should see about leveraging nginx to also help with throttling to prevent brute force attempts
        std::thread::sleep(std::time::Duration::from_millis(1_000));
        let r = warp::reply::json(&());
        Ok(Box::new(warp::reply::with_status(
            r,
            StatusCode::UNAUTHORIZED,
        )))
    }
}

fn json_login_body() -> impl Filter<Extract = (Login,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

fn json_meal_body() -> impl Filter<Extract = (Meal,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

async fn is_db_avail(
    client: dynomite::retry::RetryingDynamoDb<dynomite::dynamodb::DynamoDbClient>,
) -> bool {
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
            read_capacity_units: 10,  // 25 max for free tier
            write_capacity_units: 10, // 25 max for free tier
        }),
        ..CreateTableInput::default()
    });
    let f = create_table_req.await;
    match f {
        Ok(_) => {
            debug!("All good making table");
            true
        }
        Err(e) => {
            // table may not be ready yet, wait and retry
            // Also, local dynamo returns a different string than real:
            let e_msg = e.to_string();
            debug!("error message is '{}'. Checking if that contains the string 'Table already exists'.", e_msg);
            if e_msg.contains("preexisting table") || e_msg.contains("Table already exists") {
                return true;
            }
            false
        }
    }
}

async fn prepopulate_db(
    client: dynomite::retry::RetryingDynamoDb<dynomite::dynamodb::DynamoDbClient>,
) {
    let mut attempts: i32 = 0;
    loop {
        debug!("Waiting for the db to be available");
        if is_db_avail(client.clone()).await {
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
            read_capacity_units: 10,  // 25 max for free tier
            write_capacity_units: 10, // 25 max for free tier
        }),
        ..CreateTableInput::default()
    });
    let f = create_table_req.await;
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
        stars: Some(4),
    };

    let _ = client
        .put_item(PutItemInput {
            table_name: table_name.clone(),
            item: m.clone().into(),
            ..PutItemInput::default()
        })
        .await;

    m.id = Uuid::parse_str("936DA01F9ABD4d9d80C702AF85C822A8").unwrap();
    m.name = "Pizza".to_string();
    m.description = "Delicious pizza".to_string();
    m.stars = Some(5);

    let _ = client
        .put_item(PutItemInput {
            table_name,
            item: m.clone().into(),
            ..PutItemInput::default()
        })
        .await;
}

async fn is_authed(auth: String, jwtdb: JwtDb) -> bool {
    debug!("Checking this jwt: {}", auth);
    let a = auth.replace("bearer: ", "");
    let token = decode::<Claims>(
        &a,
        &DecodingKey::from_secret(JWT_SECRET.as_ref()),
        &Validation::default(),
    );
    match token {
        Ok(_) => {
            // use what's returned in the Ok field to inspect token contents like the claims subject (account)
            debug!("Token is decodable");
            let d = jwtdb.lock().await;
            let c = d.contains_key(&a);
            if !c {
                debug!("JWT isn't one we know about, rejecting it");
            }
            c
        }
        Err(e) => {
            debug!("Token no good: {:?}", e);
            false
        }
    }
}

#[derive(Serialize)]
pub struct ErrorResp {
    error: String,
}

#[derive(Serialize, Debug)]
struct Health {
    healthy: bool,
    version: String,
}

#[derive(Deserialize, Debug)]
pub struct Login {
    user: String,
    pw: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    exp: u32,    // Required (validate_exp defaults to true in validation). Expiration time
    sub: String, // Optional. Subject (whom token refers to)
}

#[derive(Debug, Serialize)]
struct LoginResp {
    jwt: String,
}
