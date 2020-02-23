#![allow(clippy::large_enum_variant)]

use seed::{browser::service::fetch, prelude::*, *};
use serde::{Deserialize, Serialize};

type MealMap = Vec<Meal>;

// Model
struct Model {
    meals: MealMap,
    meal_under_construction: Meal,
    error: Option<String>,
    page: Pages,
}

#[derive(Clone, Debug, PartialEq)]
enum Pages {
    Home,
    Meals { meal_id: Option<i32> },
    CreateMeal,
    Login,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            meals: MealMap::new(),
            error: None,
            page: Pages::Home,
            meal_under_construction: Meal {
                name: "".to_string(),
                id: 0,
            },
        }
    }
}

impl Model {
    pub fn meal_ready_to_submit(&self) -> bool {
        if !self.meal_under_construction.name.is_empty() {
            return true;
        }
        false
    }
}

#[derive(Serialize)]
struct CreateMealRequestBody {
    pub name: String,
    pub id: i32,
}

#[derive(Debug, Clone, Deserialize)]
struct MealCreatedResponse {}

// Update
#[derive(Clone, Debug)]
enum Msg {
    FetchData { meal_id: Option<i32> },
    MealsFetched(fetch::ResponseDataResult<MealMap>),
    MealFetched(fetch::ResponseDataResult<Meal>),
    ChangePage(Pages),
    MealCreateUpdateName(String),
    CreateNewMeal(Meal),
    MealValidationError,
    MealCreated(seed::fetch::ResponseDataResult<MealCreatedResponse>),
}

/// How we update the model
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    log!("updating, msg is {:?}", msg);
    match msg {
        Msg::MealValidationError => {
            log!("validation fail");
            model.error = Some("Fill out the fields please".to_string());
        }
        Msg::MealCreateUpdateName(name) => {
            model.meal_under_construction.name = name;
            log!(
                "model meal under constr is {}",
                model.meal_under_construction.name
            );
        }
        Msg::CreateNewMeal(meal) => {
            log("creating a new meal");
            if model.meal_ready_to_submit() {
                log!("ready to submit!");
                orders.skip().perform_cmd(create_meal(meal));
            } else {
                log!("error before submission");
                model.error = Some("provide a meal first".to_string());
                orders.send_msg(Msg::MealValidationError);
            }
        }
        Msg::MealCreated(Ok(m)) => {
            log!("m is {:?}", m);
        }
        Msg::MealCreated(Err(fail_reason)) => {
            model.error = Some(format!("Couldn't create meal: {:#?}", fail_reason));
        }
        Msg::FetchData { meal_id } => {
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
            model.error = Some(format!("Error fetching meals: {:?}", fail_reason));
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
            model.error = Some(format!("Error fetching meal: {:?}", fail_reason));
        }
        Msg::ChangePage(page) => {
            if let Pages::Meals { meal_id } = page {
                orders.send_msg(Msg::FetchData { meal_id });
            }
            model.page = page;
        }
    }
}

// View
/// The top-level component we pass to the virtual dom.
fn view(model: &Model) -> impl View<Msg> {
    log!("meals be {:?}", model.meals);

    let page_contents = match model.page {
        Pages::Home => home(),
        Pages::CreateMeal => create_meal_view(model),
        Pages::Login => vec![
            h2!["login"],
            p![],
            p!["This will have authentication sometime."],
        ],
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

fn create_meal_view(model: &Model) -> Vec<Node<Msg>> {
    vec![
        h2!["create a meal"],
        p![],
        input!["name", input_ev(Ev::Input, Msg::MealCreateUpdateName)],
        button![
            "new",
            simple_ev(
                Ev::Click,
                Msg::CreateNewMeal(model.meal_under_construction.clone())
            )
        ],
    ]
}

fn home() -> Vec<Node<Msg>> {
    let header = h2!["refeed rampage home"];
    let contents = div![
        p![], // hacky spacing
        h5!["What is this?"],
        p![], // hacky spacing
        p!["I tend to follow a cyclical ketogenic diet: low carbs six days a week and one refeed day a week that's high in carbs. The refeed day is also known as \"rampage day\" where *all the carbs* can be consumed."],
        p!["This project is aimed at recording what I ate, how I liked it (will I eat the food again) and a general log on how I feel during/after the rampage."],
        p![],
        p!["Source code is available at ", a!["https://github.com/matthewkmayer/refeed-rampage", attrs! {At::Href => "https://github.com/matthewkmayer/refeed-rampage"}], "."]
    ];
    vec![header, contents]
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
            button!["Login", attrs! {At::Href => "/login"},],
        ],
        // match Meals with a specific meal specific or all meals:
        Pages::Meals { .. } | Pages::CreateMeal => vec![
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
            button!["Login", attrs! {At::Href => "/login"},],
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
            button![
                "Login",
                attrs! {At::Href => "/login"},
                span![class!["sr-only"], "(current)"],
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

// perhaps we don't want to make a link to the page/item we're on
fn meal_item(m: &Meal) -> Node<Msg> {
    let link = format!("/meals/{}", m.id);
    h4![a![format!("{:?}", m), attrs! {At::Href => link},]]
}

fn meal_list(model: &Model) -> Vec<Node<Msg>> {
    let mut m = match &model.error {
        Some(e) => vec![
            h2!["Couldn't fetch requested data. :("],
            p![],
            p!["nerdy reasons: ", e],
        ],
        None => model.meals.iter().map(|m| meal_item(m)).collect(),
    };

    // This should be a button that changes the browser's current URL
    // and sends a change page msg:
    m.push(a![attrs! {At::Href => "meals/create"}, "add a new one"]);
    m
}

// https://seed-rs.org/guide/http-requests-and-state

async fn fetch_meals() -> Result<Msg, Msg> {
    let url = "http://127.0.0.1:3030/meals";
    Request::new(url).fetch_json_data(Msg::MealsFetched).await
}

async fn create_meal(meal: Meal) -> Result<Msg, Msg> {
    let url = "http://127.0.0.1:3030/meals";
    Request::new(url)
        .method(Method::Post)
        .send_json(&meal)
        .fetch_json_data(Msg::MealCreated)
        .await
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
    log!("url path is {}", url.path);
    Some(match url.path[0].as_ref() {
        "meals" => match url.path.get(1).as_ref() {
            Some(page) => {
                if page == &"create" {
                    return Some(Msg::ChangePage(Pages::CreateMeal));
                }
                match page.parse::<i32>() {
                    Ok(m_id) => Msg::ChangePage(Pages::Meals {
                        meal_id: Some(m_id),
                    }),
                    Err(e) => {
                        log!("Got an error on meal id: {}", e);
                        return None;
                    }
                }
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
        .after_mount(after_mount)
        .build_and_start();
}

fn after_mount(url: Url, orders: &mut impl Orders<Msg>) -> AfterMount<Model> {
    let mut m: Model = Default::default();

    // same code as `routes`
    if url.path.is_empty() {
        m.page = Pages::Home;
    }

    m.page = match url.path[0].as_ref() {
        "meals" => match url.path.get(1).as_ref() {
            Some(page) => match page.as_ref() {
                "create" => Pages::CreateMeal,
                _ => {
                    let m_id = page.parse::<i32>().unwrap();
                    Pages::Meals {
                        meal_id: Some(m_id),
                    }
                }
            },
            None => Pages::Meals { meal_id: None },
        },
        "login" => Pages::Login,
        _ => Pages::Home,
    };

    orders.send_msg(routes(url).unwrap());
    AfterMount::new(m)
}

#[derive(Deserialize, Serialize, Clone, Eq, PartialEq, Hash, Debug)]
struct Meal {
    name: String,
    id: i32,
}
