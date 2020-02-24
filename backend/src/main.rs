use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::http::StatusCode;
use warp::Filter;

type Db = Arc<Mutex<BTreeMap<i32, Meal>>>;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    // a bunch from https://github.com/seanmonstar/warp/blob/master/examples/todos.rs
    let fake_db = Arc::new(Mutex::new(BTreeMap::new()));
    prepopulate_db(fake_db.clone()).await;

    let cors = warp::cors()
        .allow_origin("http://localhost:8080")
        .allow_origin("http://127.0.0.1:8080")
        .allow_methods(vec!["GET", "POST", "DELETE"])
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
        .or(meal_create(db))
}

fn a_meal_filter(
    ds: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("meals" / i32)
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

async fn specific_meal(i: i32, db: Db) -> Result<impl warp::Reply, Infallible> {
    let fake_db = db.lock().await;
    // TODO: figure out why logging doesn't work as I expected it to
    log::info!("ds is {:?}", db);
    Ok(warp::reply::json(&fake_db.get_key_value(&i).unwrap().1))
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
    let len: usize;

    if !d.contains_key(&create.id) {
        len = d.len() + 1;
        d.insert(
            len as i32,
            Meal {
                id: len as i32,
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
    let json = warp::reply::json(d.get(&(len as i32)).unwrap());
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
    d.insert(
        1,
        Meal {
            id: 1,
            name: "Pizza".to_string(),
            photos: None,
            description: "Delicious pizza".to_string(),
        },
    );
    d.insert(
        2,
        Meal {
            id: 2,
            name: "Burritos".to_string(),
            photos: None,
            description: "Amazing burritos".to_string(),
        },
    );
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Meal {
    name: String,
    id: i32,
    photos: Option<String>,
    description: String,
}
