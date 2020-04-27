use crate::frontend_types::LoginInput;
use crate::{Meal, Msg, URL_BASE};
use seed::*; // scope down?
use uuid::Uuid;
// https://seed-rs.org/guide/http-requests-and-state

pub async fn delete_meal(id: Uuid, auth: String) -> Result<Msg, Msg> {
    let url = format!("{}/meals/{}", URL_BASE.replace("\n", ""), id);
    Request::new(url)
        .method(Method::Delete)
        .header("Authorization", &format!("bearer: {}", auth))
        .fetch_json_data(Msg::MealDeleted)
        .await
}

pub async fn fetch_meals() -> Result<Msg, Msg> {
    log!("Fetching meals");
    let url = format!("{}/meals", URL_BASE.replace("\n", ""));
    Request::new(url).fetch_json_data(Msg::MealsFetched).await
}

pub async fn create_meal(meal: Meal, auth: String) -> Result<Msg, Msg> {
    let url = format!("{}/meals", URL_BASE.replace("\n", ""));
    Request::new(url)
        .method(Method::Post)
        .header("Authorization", &format!("bearer: {}", auth))
        .send_json(&meal)
        .fetch_json_data(Msg::MealCreated)
        .await
}

pub async fn login(login: LoginInput) -> Result<Msg, Msg> {
    let url = format!("{}/login", URL_BASE.replace("\n", ""));
    Request::new(url)
        .method(Method::Post)
        .send_json(&login)
        .fetch_json_data(Msg::LoginResp)
        .await
}

pub async fn update_meal(meal: Meal, auth: String) -> Result<Msg, Msg> {
    let url = format!("{}/meals/{}", URL_BASE.replace("\n", ""), meal.id);
    Request::new(url)
        .method(Method::Put)
        .header("Authorization", &format!("bearer: {}", auth))
        .send_json(&meal)
        .fetch_json_data(Msg::MealCreated)
        .await
}

pub async fn fetch_meal(id: Uuid) -> Result<Msg, Msg> {
    let url = format!("{}/meals/{}", URL_BASE.replace("\n", ""), id);
    Request::new(url).fetch_json_data(Msg::MealFetched).await
}
