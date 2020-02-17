#![allow(clippy::large_enum_variant)]

use seed::{browser::service::fetch, prelude::*, *};
use serde::{Deserialize, Serialize};

type MealMap = Vec<Meal>;

// Model
struct Model {
    meals: MealMap,
    error: Option<String>,
    page: Pages,
}

#[derive(Clone, Debug, PartialEq)]
enum Pages {
    Home,
    Meals { meal_id: Option<i32> },
    Login,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            meals: MealMap::new(),
            error: None,
            page: Pages::Home,
        }
    }
}

// Update
#[derive(Clone, Debug)]
enum Msg {
    FetchData { meal_id: Option<i32> },
    MealsFetched(fetch::ResponseDataResult<MealMap>),
    MealFetched(fetch::ResponseDataResult<Meal>),
    ChangePage(Pages),
}

/// How we update the model
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::FetchData { meal_id } => {
            log!("doot");
            match meal_id {
                Some(id) => orders.skip().perform_cmd(fetch_meal(id)),
                None => orders.skip().perform_cmd(fetch_meals()),
            };
        }
        Msg::MealsFetched(Ok(meals)) => {
            log!(format!("Response data: {:#?}", meals));
            model.meals = meals;
            model.error = None;
        }
        Msg::MealsFetched(Err(fail_reason)) => {
            log!("error: {:#?}", fail_reason);
            error!(format!(
                "Fetch error - Sending message failed - {:#?}",
                fail_reason
            ));
            model.error = Some("Error fetching meals".to_string());
        }
        Msg::MealFetched(Ok(meals)) => {
            log!(format!("Response data: {:#?}", meals));
            model.meals = vec![meals];
            model.error = None;
        }
        Msg::MealFetched(Err(fail_reason)) => {
            log!("error: {:#?}", fail_reason);
            error!(format!(
                "Fetch error - Sending message failed - {:#?}",
                fail_reason
            ));
            model.error = Some("Error fetching meals".to_string());
        }
        Msg::ChangePage(page) => {
            model.page = page;
        }
    }
}

// View
/// The top-level component we pass to the virtual dom.
fn view(model: &Model) -> impl View<Msg> {
    log!("meals be {:?}", model.meals);

    let page_contents = match model.page {
        Pages::Home => vec![h2!["refeed rampage home"]],
        Pages::Login => vec![h2!["login"]],
        Pages::Meals { meal_id } => {
            match meal_id {
                Some(_) => {
                    log!("meal id specific");
                    // should we have a meal detail view instead of this list which
                    // will only have one item in it?
                    let mut c = meal_list(model);
                    c.push(button![
                        simple_ev(Ev::Click, Msg::FetchData { meal_id: meal_id }),
                        "refresh this item"
                    ]);
                    c
                }
                None => {
                    log!("no meal id specific, show them all");
                    let mut c = meal_list(model);
                    c.push(button![
                        simple_ev(Ev::Click, Msg::FetchData { meal_id: None }),
                        "get all meals"
                    ]);
                    c
                }
            }
        }
    };
    let main = main![
        class!["container"],
        div![class!["jumbotron"], page_contents,],
    ];

    vec![nav(model), main]
}

// this got wet in a hurry, how can we DRY it out?
fn nav_nodes(model: &Model) -> Vec<Node<Msg>> {
    match model.page {
        Pages::Home => vec![
            ul![
                class!["navbar-nav mr-auto"],
                li![
                    class!["nav-item active"],
                    a![
                        "Home",
                        class!["nav-link"],
                        span![class!["sr-only"], "(current)"],
                        attrs! {At::Href => "/"}
                    ]
                ],
                li![
                    class!["nav-item"],
                    a!["Meals", class!["nav-link"], attrs! {At::Href => "/meals"}]
                ]
            ],
            a![
                "Login",
                class!["form-inline mt-2 mt-md-0"],
                attrs! {At::Href => "/login"},
            ],
        ],
        // match Meals with a specific meal specific or all meals:
        Pages::Meals { .. } => vec![
            ul![
                class!["navbar-nav mr-auto"],
                li![
                    class!["nav-item"],
                    a!["Home", class!["nav-link"], attrs! {At::Href => "/"}]
                ],
                li![
                    class!["nav-item active"],
                    a![
                        "Meals",
                        class!["nav-link"],
                        span![class!["sr-only"], "(current)"],
                        attrs! {At::Href => "/meals"}
                    ]
                ]
            ],
            a![
                "Login",
                class!["form-inline mt-2 mt-md-0"],
                attrs! {At::Href => "/login"},
            ],
        ],
        Pages::Login => vec![
            ul![
                class!["navbar-nav mr-auto"],
                li![
                    class!["nav-item"],
                    a!["Home", class!["nav-link"], attrs! {At::Href => "/"}]
                ],
                li![
                    class!["nav-item"],
                    a!["Meals", class!["nav-link"], attrs! {At::Href => "/meals"}]
                ]
            ],
            a![
                "Login",
                class!["form-inline mt-2 mt-md-0 active"],
                span![class!["sr-only"], "(current)"],
                attrs! {At::Href => "/login"},
            ],
        ],
    }
}

fn nav(model: &Model) -> Node<Msg> {
    nav![
        class!["navbar navbar-expand-md navbar-light bg-light mb-4"],
        a![
            "refeed rampage",
            class!["navbar-brand"],
            attrs! {At::Href => "/"}
        ],
        div![
            class!["collapse navbar-collapse"],
            id!["navbarCollapse"],
            nav_nodes(model),
        ],
    ]
}

fn meal_list(model: &Model) -> Vec<Node<Msg>> {
    match &model.error {
        Some(_e) => vec![h2!["oh no error"]],
        None => model
            .meals
            .iter()
            .map(|m| h4![format!("{:?}", m)])
            .collect(),
    }
}

// https://seed-rs.org/guide/http-requests-and-state

async fn fetch_meals() -> Result<Msg, Msg> {
    let url = "http://127.0.0.1:3030/meals";
    Request::new(url).fetch_json_data(Msg::MealsFetched).await
}

async fn fetch_meal(id: i32) -> Result<Msg, Msg> {
    let url = format!("http://127.0.0.1:3030/meals/{}", id);
    log!("fetching from {}", url);
    Request::new(url).fetch_json_data(Msg::MealFetched).await
}

fn routes(url: Url) -> Option<Msg> {
    if url.path.is_empty() {
        return Some(Msg::ChangePage(Pages::Home));
    }

    Some(match url.path[0].as_ref() {
        "meals" => match url.path.get(1).as_ref() {
            Some(page) => {
                let m_id = page.parse::<i32>().unwrap();
                Msg::ChangePage(Pages::Meals {
                    meal_id: Some(m_id),
                })
            }
            None => Msg::ChangePage(Pages::Meals { meal_id: None }),
        },
        "login" => Msg::ChangePage(Pages::Login),
        _ => Msg::ChangePage(Pages::Home),
    })
}

#[wasm_bindgen(start)]
pub fn render() {
    seed::App::builder(update, view)
        .routes(routes)
        .build_and_start();
}

#[derive(Deserialize, Serialize, Clone, Eq, PartialEq, Hash, Debug)]
struct Meal {
    name: String,
    id: u32,
}
