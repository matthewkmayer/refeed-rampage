#![allow(clippy::large_enum_variant)]

use seed::{browser::service::fetch, prelude::*, *};
use serde::{Deserialize, Serialize};
// use std::iter::*;

type MealMap = Vec<Meal>;

// Model
struct Model {
    meals: MealMap,
    error: Option<String>,
}

// Setup a default here, for initialization later.
impl Default for Model {
    fn default() -> Self {
        Self {
            meals: MealMap::new(),
            error: None,
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
            model.error = None;
        }
        Msg::DataFetched(Err(fail_reason)) => {
            log!("error: {:#?}", fail_reason);
            error!(format!(
                "Fetch error - Sending message failed - {:#?}",
                fail_reason
            ));
            model.error = Some("Error fetching meals".to_string());
        }
    }
}

// View
/// The top-level component we pass to the virtual dom.
fn view(model: &Model) -> impl View<Msg> {
    log!("meals be {:?}", model.meals);

    // let inner_nav = div![
    //     class!["col-md-2 d-none d-md-block bg-light sidebar"],
    // ];
    // let main_2 = div![
    //     class!["chartjs-size-monitor"],
    //     "main2",
    // ];
    // let main_contents = div![
    //     class!["col-md-9 ml-sm-auto col-lg-10 px-4"],
    //     main_2,
    // ];
    // let row = div![
    //     class!["row"],
    //     inner_nav,
    //     main_contents,
    // ];

    // let main_container = div![
    //     class!["container-fluid"],
    //     row,

    // ];

    let nav = nav![
        class!["navbar navbar-expand-md navbar-light bg-light mb-4"],
        a![
            "refeed rampage",
            class!["navbar-brand"],
            attrs! {At::Href => "#"}
        ],
        div![
            class!["collapse navbar-collapse"],
            id!["navbarCollapse"],
            ul![
                class!["navbar-nav mr-auto"],
                li![
                    class!["nav-item active"],
                    a![
                        "Home",
                        class!["nav-link"],
                        span![class!["sr-only"], "(current)"],
                        attrs! {At::Href => "#"}
                    ]
                ],
                li![
                    class!["nav-item"],
                    a!["Meals", class!["nav-link"], attrs! {At::Href => "#"}]
                ]
            ],
        ],
    ];

    vec![
        nav,
        // main_container,
        h3!["content here"],
    ]

    // let list = match &model.error {
    //     Some(_e) => vec![h2!["oh no error"]],
    //     None => model
    //         .meals
    //         .iter()
    //         .map(|m| h4![format!("{:?}", m)])
    //         .collect(),
    // };
    // div![
    //     nav,
    //     div![
    //         class!["container-fluid"],
    //         h3!["Meals available:"],
    //         list,
    //     button![simple_ev(Ev::Click, Msg::FetchData), "get em"],
    // ],]
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

#[derive(Deserialize, Serialize, Clone, Eq, PartialEq, Hash, Debug)]
struct Meal {
    name: String,
    id: u32,
}
