use seed::{browser::service::fetch, prelude::*, *};
use serde::{Serialize, Deserialize};
use std::iter::*;
use indexmap::IndexMap;
use futures::Future;

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
    DataFetched(seed::fetch::ResponseDataResult<IndexMap::<Meal, u32>>),
    FetchData,
}

/// How we update the model
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::FetchData => {
            println!("fetching");
            orders.skip().perform_cmd(fetch_data());
        },
        Msg::DataFetched(Ok(meals)) => {
            println!("updated");
            model.meals = meals;
        },
        _ => println!("other"),
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

async fn fetch_data() -> Msg {
    let url = "localhost:3030/meals";
    Request::new(url).fetch_json_data(Msg::DataFetched).await
}

#[wasm_bindgen(start)]
pub fn render() {
    let app = seed::App::builder(update, view)
        .build_and_start();

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