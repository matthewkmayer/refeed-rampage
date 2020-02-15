use indexmap::IndexMap;
use seed::{browser::service::fetch, prelude::*, *};
use serde::{Deserialize, Serialize};
use std::iter::*;
// use futures::Future;

type MealMap = IndexMap<Meal, u32>;

// Model
struct Model {
    meals: MealMap,
}

// Setup a default here, for initialization later.
impl Default for Model {
    fn default() -> Self {
        Self {
            meals: MealMap::new(),
        }
    }
}

// Update
#[derive(Clone, Debug)]
enum Msg {
    FetchData,
    DataFetched(fetch::ResponseDataResult<MealMap>),
}

/// How we update the model
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::FetchData => {
            log!("fetching");
            orders.skip().perform_cmd(fetch_data());
        }
        Msg::DataFetched(Ok(meals)) => {
            log!("updated");
            log!(format!("Response data: {:#?}", meals));
            model.meals = meals;
            orders.skip();
        }
        Msg::DataFetched(Err(fail_reason)) => {
            log!("error: {:#?}", fail_reason);
            error!(format!(
                "Fetch error - Sending message failed - {:#?}",
                fail_reason
            ));
            orders.skip();
        }
    }
}

// View
/// The top-level component we pass to the virtual dom.
fn view(model: &Model) -> impl View<Msg> {
    div![
        div![
            style! {
                // Example of conditional logic in a style.
                St::Color => {"gray"};
                St::Border => "2px solid #004422";
                St::Padding => unit!(20, px);
            },
            h3!["Meals available"],
            model.meals.iter().map(|m| h4![format!("{:?}", m)]),
            button![simple_ev(Ev::Click, Msg::FetchData), "get em"],
        ],
        h3!["What are we counting  ?"],
    ]
}

// https://seed-rs.org/guide/http-requests-and-state

async fn fetch_data() -> Result<Msg, Msg> {
    let url = "http://127.0.0.1:3030/meals";
    log!("boop sending request");
    Request::new(url).fetch_json_data(Msg::DataFetched).await
}

#[wasm_bindgen(start)]
pub fn render() {
    let app = seed::App::builder(update, view).build_and_start();

    app.update(Msg::FetchData);
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct Meals {
    meals: IndexMap<Meal, u32>,
}
#[derive(Deserialize, Serialize, Clone, Eq, PartialEq, Hash, Debug)]
struct Meal {
    name: String,
    id: u32,
}
