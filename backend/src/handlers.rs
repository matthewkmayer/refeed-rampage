use crate::s3_interactions;
use dynomite::{
    dynamodb::{DynamoDb, DynamoDbClient, GetItemInput},
    FromAttributes, Item,
};
use shared::Meal;
use uuid::Uuid;
use warp::http::StatusCode;

pub async fn specific_meal(
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
            // TODO: get image locations here and return them with presigned URLs
            // something similar to this but working:
            // let _images = s3_interactions::keys_from_list(&item_found.unwrap().unwrap().photos.unwrap())

            // temporarily get rid of compiler warnings about unused function:
            let _images = s3_interactions::keys_from_list("fake");
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
