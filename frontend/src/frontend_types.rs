use crate::Meal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoginResp {
    pub jwt: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoginInput {
    pub user: String,
    pub pw: String,
}

pub type MealMap = Vec<Meal>;

#[derive(Clone, Debug, PartialEq)]
pub enum SortingOptions {
    StarsAsc,
    StarsDesc,
}

#[derive(Serialize)]
pub struct CreateMealRequestBody {
    pub name: String,
    pub id: Uuid,
}

// #[derive(Debug, Clone, Deserialize)]
pub type MealCreatedResponse = Meal;

#[derive(Debug, Clone, Deserialize)]
pub struct MealDeletedResponse {}
