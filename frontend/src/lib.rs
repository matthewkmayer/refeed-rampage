use seed::{prelude::*, *};
use serde::{Serialize, Deserialize};
use std::iter::*;
use indexmap::IndexMap;

// Model
struct Model {
    meals: IndexMap::<Meal, u32>,
}

// Setup a default here, for initialization later.
impl Default for Model {
    fn default() -> Self {
        Self {
            meals: IndexMap::<Meal, u32>::new()
        }
    }
}

// Update
#[derive(Clone)]
enum Msg {
    GetAllMeals(IndexMap::<Meal, u32>),
}

/// How we update the model
fn update(msg: Msg, model: &mut Model, _orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::GetAllMeals(meals) => model.meals = meals,
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
            // button![simple_ev(Ev::Click, Msg::GetAllMeals), "get em"],
        ],
        h3!["What are we counting  ?"],
    ]
}

// https://seed-rs.org/guide/http-requests-and-state

#[derive(Clone, Debug)]
enum Msg2 {
    DataFetched(seed::fetch::ResponseDataResult<Meals>),
}

async fn fetch_data() -> Msg2 {
    let url = "localhost:3030/meals";
    match Request::new(url).fetch_json_data(Msg2::DataFetched).await {
        Ok(i) => return i,
        Err(e) => return e,
    }
}

// fn after_mount(_: Url, orders: &mut impl Orders<Msg2>) -> AfterMount<Model> {
//     orders.perform_cmd(fetch_data());
//     AfterMount::default()
// }

#[wasm_bindgen(start)]
pub fn render() {
    App::builder(update, view).build_and_start();
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