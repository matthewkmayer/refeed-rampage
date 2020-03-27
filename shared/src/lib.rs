use dynomite::Item;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Default, Item)]
pub struct Meal {
    #[dynomite(rename = "mealName")]
    pub name: String,
    #[dynomite(partition_key)]
    pub id: Uuid,
    pub photos: Option<String>,
    pub description: String,
}
