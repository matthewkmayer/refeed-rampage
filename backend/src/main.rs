use warp::Filter;

#[tokio::main]
async fn main() {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));
    // GET /meals
    let all_meals = warp::path("meals").map(|| format!("all the meals {}", all_meals()));

    let routes = warp::get().and(hello.or(all_meals));

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

fn all_meals() -> i32 {
    rand::random::<i32>()
}
