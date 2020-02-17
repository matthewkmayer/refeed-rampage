use serde_derive::{Deserialize, Serialize};
use warp::Filter;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let cors = warp::cors()
        .allow_origin("http://localhost:8080")
        .allow_origin("http://127.0.0.1:8080")
        .allow_methods(vec!["GET", "POST", "DELETE"]);
    // GET /hello/warp
    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));
    // GET /meals
    let all_meals = warp::path("meals")
        .map(|| warp::reply::json(&all_meals()))
        .with(&cors);
    // GET /meals/:id
    let meal_by_id = warp::path!("meals" / i32)
        .map(|i| warp::reply::json(&specific_meal(i)))
        .with(&cors);

    let routes = warp::get().and(hello.or(meal_by_id.or(all_meals)));

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

// this is where data store interaction will happen
fn specific_meal(i: i32) -> Meal {
    Meal {
        name: format!("meal {}", rand::random::<i32>()),
        id: i,
    }
}

// this is where data store interaction will happen
fn all_meals() -> Vec<Meal> {
    let m1 = Meal {
        name: format!("meal {}", rand::random::<i32>()),
        id: rand::random::<i32>(),
    };
    let m2 = Meal {
        name: format!("meal {}", rand::random::<i32>()),
        id: rand::random::<i32>(),
    };

    vec![m1, m2]
}

// TODO: extract these types somewhere else

#[derive(Deserialize, Serialize)]
struct Meal {
    name: String,
    id: i32,
}
