#[cfg(feature = "dynamo_bits")]
use dynomite::Item;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg_attr(feature = "dynamo_bits", derive(Item))]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Eq, PartialEq, Hash)]
pub struct Meal {
    #[cfg_attr(feature = "dynamo_bits", dynomite(rename = "mealName"))]
    pub name: String,
    #[cfg_attr(feature = "dynamo_bits", dynomite(partition_key))]
    pub id: Uuid,
    // This will be populated by the backend as presigned URLs for the frontend to fetch
    pub photos: Option<String>,
    pub description: String,
    pub stars: Option<i32>,
}
