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
        .with(cors);

    let routes = warp::get().and(hello.or(all_meals));

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

fn all_meals() -> Vec<Meal> {
    let m1 = Meal {
        name: format!("meal {}", rand::random::<i32>()),
        id: rand::random::<u32>(),
    };

    vec![m1]
}

// TODO: extract these types somewhere else
#[derive(Deserialize, Serialize)]
struct Meals {
    meals: Vec<Meal>,
}

#[derive(Deserialize, Serialize)]
struct Meal {
    name: String,
    id: u32,
}
