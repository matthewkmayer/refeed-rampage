#![allow(clippy::large_enum_variant)]

use seed::{browser::service::fetch, prelude::*, *};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

static URL_BASE: &str = include_str!("api_loc.txt");
static GITBITS: &str = include_str!("gitbits.txt");

type MealMap = Vec<Meal>;

// Model
struct Model {
    meals: MealMap,
    meal_under_construction: Meal,
    meal: Meal,
    error: Option<String>,
    page: Pages,
    login: Option<LoginInput>,
}

#[derive(Serialize, Clone, Debug)]
struct LoginInput {
    user: String,
    pw: String,
}

#[derive(Clone, Debug, PartialEq)]
enum Pages {
    Home,
    Meals,
    ViewSpecificMeal { meal_id: Uuid },
    EditMeal { meal_id: Uuid },
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
                description: "".to_string(),
                id: Uuid::new_v4(),
                photos: None,
            },
            meal: Meal {
                name: "".to_string(),
                description: "".to_string(),
                id: Uuid::new_v4(),
                photos: None,
            },
            login: None,
        }
    }
}

impl Model {
    pub fn meal_ready_to_submit(&self) -> bool {
        if !self.meal_under_construction.name.is_empty()
            && !self.meal_under_construction.description.is_empty()
        {
            return true;
        }
        false
    }
}

#[derive(Serialize)]
struct CreateMealRequestBody {
    pub name: String,
    pub id: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
struct MealCreatedResponse {}

#[derive(Debug, Clone, Deserialize)]
struct MealDeletedResponse {}

// Update
#[derive(Clone, Debug)]
enum Msg {
    // editing
    EditMeal { meal_id: Uuid },
    MealCreateUpdateName(String),
    MealCreateUpdateDescription(String),
    CreateNewMeal(Meal),
    SaveMeal(Meal),
    MealValidationError,
    MealCreated(seed::fetch::ResponseDataResult<MealCreatedResponse>),
    // deleting
    DeleteMeal { meal_id: Uuid },
    MealDeleted(seed::fetch::ResponseDataResult<MealDeletedResponse>),
    // changing page
    ChangePage(Pages),
    // fetching etc
    FetchData { meal_id: Option<Uuid> },
    MealsFetched(fetch::ResponseDataResult<MealMap>),
    MealFetched(fetch::ResponseDataResult<Meal>),
    // login
    LoginUserUpdated(String),
    LoginPwUpdated(String),
    Login { login: Option<LoginInput> },
}

/// How we update the model
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    // TODO: move these around to group like things together
    match msg {
        Msg::Login { login: lin } => match lin {
            Some(_l) => {
                log!("Got something to send");
                model.error = None;
                // log!(format!("Sending {:?}", l));
            }
            None => {
                log!("No login info to send");
                model.error = Some("Please enter a username and password".to_string());
            }
        },
        Msg::LoginUserUpdated(u) => match model.login.as_mut() {
            Some(m) => m.user = u,
            None => {
                model.login = Some(LoginInput {
                    user: u,
                    pw: "".to_string(),
                })
            }
        },
        Msg::LoginPwUpdated(pw) => match model.login.as_mut() {
            Some(m) => m.pw = pw,
            None => {
                model.login = Some(LoginInput {
                    user: "".to_string(),
                    pw,
                })
            }
        },
        Msg::SaveMeal(meal) => {
            log!("updating existing meal");
            if meal.ready_to_submit() {
                log!("ready to submit!");
                orders.skip().perform_cmd(update_meal(meal));
            } else {
                log!("error before save submission");
                model.error = Some("provide a meal first".to_string());
                orders.send_msg(Msg::MealValidationError);
            }
        }
        Msg::EditMeal { meal_id: id } => {
            log!("editing a meal");
            orders.skip().perform_cmd(fetch_meal(id));
            log!("and done fetching");
        }
        Msg::MealDeleted(_) => {
            log!("deleted!");
            orders.send_msg(Msg::ChangePage(Pages::Meals));
        }
        Msg::DeleteMeal { meal_id: id } => {
            orders.skip().perform_cmd(delete_meal(id));
        }
        Msg::MealValidationError => {
            log!("validation fail");
            model.error = Some("Fill out the fields please".to_string());
        }
        Msg::MealCreateUpdateName(name) => {
            model.meal_under_construction.name = name;
            log!(format!(
                "model meal under constr name is {}",
                model.meal_under_construction.name
            ));
        }
        Msg::MealCreateUpdateDescription(desc) => {
            model.meal_under_construction.description = desc;
            log!(format!(
                "model meal under constr desc is {}",
                model.meal_under_construction.description
            ));
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
            model.error = None;
            orders.send_msg(Msg::ChangePage(Pages::Meals));
        }
        Msg::MealCreated(Err(fail_reason)) => {
            log!(format!("sad times: {:?}", fail_reason));
            model.error = Some(format!("Couldn't create meal: {:#?}", fail_reason));
        }
        Msg::FetchData { meal_id } => {
            match meal_id {
                Some(id) => orders.skip().perform_cmd(fetch_meal(id)),
                None => orders.skip().perform_cmd(fetch_meals()),
            };
        }
        Msg::MealsFetched(Ok(meals)) => {
            log!("hey it worked");
            model.meals = meals;
            model.error = None;
        }
        Msg::MealsFetched(Err(fail_reason)) => {
            // 404 should go to 404 page
            log!(format!("error fetchin' meal: {:#?}", fail_reason));
            error!(format!(
                "Fetch error - Sending message failed - {:#?}",
                fail_reason
            ));
            model.error = Some(format!("Error fetching meals: {:?}", fail_reason));
        }
        Msg::MealFetched(Ok(meal)) => {
            model.meals = vec![];
            model.meal = meal;
            model.meal_under_construction = model.meal.clone();
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
            if let Pages::ViewSpecificMeal { meal_id } = page {
                orders.send_msg(Msg::FetchData {
                    meal_id: Some(meal_id),
                });
            }
            if let Pages::Meals = page {
                orders.send_msg(Msg::FetchData { meal_id: None });
            }
            if let Pages::EditMeal { meal_id } = page {
                orders.send_msg(Msg::FetchData {
                    meal_id: Some(meal_id),
                });
            }
            // Clears out any meal under construction if we're gonna make a new one
            if let Pages::CreateMeal = page {
                model.meal_under_construction = Meal {
                    name: "".to_string(),
                    description: "".to_string(),
                    id: Uuid::new_v4(),
                    photos: None,
                };
            }
            model.page = page;
        }
    }
}

// View
/// The top-level component we pass to the virtual dom.
fn view(model: &Model) -> impl View<Msg> {
    let page_contents = match model.page {
        Pages::Home => home(),
        Pages::EditMeal { .. } => {
            // load up edit meal page for the specified meal
            create_meal_view(model)
        }
        Pages::CreateMeal => create_meal_view(model),
        Pages::Login => vec![
            h2!["login"],
            p![],
            input![
                class!["form-control col-4"],
                attrs! {At::Type => "text", At::Placeholder => "username" },
                id!["username"],
                input_ev(Ev::Input, Msg::LoginUserUpdated),
            ],
            input![
                class!["form-control col-4"],
                attrs! {At::Type => "password", At::Placeholder => "password" },
                id!["password"],
                input_ev(Ev::Input, Msg::LoginPwUpdated),
            ],
            button![
                "login",
                simple_ev(
                    Ev::Click,
                    Msg::Login {
                        login: model.login.clone()
                    }
                ),
            ],
            match &model.error {
                Some(e) => p![format!(
                    "Please enter a username and password. Error: {}",
                    e
                )],
                None => empty![],
            },
        ],
        Pages::Meals => {
            log!("no meal id specific, show them all");
            let mut c = meal_list(model);
            c.push(button![
                simple_ev(Ev::Click, Msg::FetchData { meal_id: None }),
                "🔄"
            ]);
            match &model.error {
                Some(e) => {
                    c.push(p![]);
                    c.push(h3!["Couldn't get those delicious meals:"]);
                    c.push(p![e]);
                }
                None => (),
            };
            c
        }
        Pages::ViewSpecificMeal { meal_id } => {
            log!("meal id specific");
            let mut c = vec![meal_item(&model.meal)];
            c.push(button![
                simple_ev(
                    Ev::Click,
                    Msg::FetchData {
                        meal_id: Some(meal_id)
                    }
                ),
                "refresh this item"
            ]);
            c
        }
    };
    let main = main![
        class!["container"],
        div![class!["jumbotron"], page_contents],
    ];

    vec![nav(model), main, footer()]
}

fn footer() -> Node<Msg> {
    let version_txt = match GITBITS.len() {
        0 => "dev",
        _ => GITBITS,
    };
    footer![
        class!["text-muted"],
        div![
            class!["container"],
            p![class!["float-right"], format!("version: {}", version_txt)]
        ]
    ]
}

fn create_meal_view(model: &Model) -> Vec<Node<Msg>> {
    let submit_text = match model.page {
        Pages::CreateMeal => "make it",
        _ => "save it",
    };
    let new_one = match model.page {
        Pages::CreateMeal => h2!["Creating a new one"],
        _ => h2!["Editing meal"],
    };
    vec![
        new_one,
        p![],
        form![
            div![
                class!["form-row"],
                div![
                    class!["form-group col-md-6"],
                    label![attrs! {At::For => "mealname"}, "Meal name",],
                    input![
                        class!["form-control"],
                        // need to set value to model meal name on first load then use meal under construction
                        attrs! {At::Type => "text", At::Placeholder => "name", At::Value => model.meal_under_construction.name },
                        id!["mealname"],
                        input_ev(Ev::Input, Msg::MealCreateUpdateName),
                    ],
                ],
            ],
            div![
                class!["form-row"],
                div![
                    class!["form-group col-md-9"],
                    label![attrs! {At::For => "mdesc"}, "Meal description",],
                    textarea![
                        class!["form-control"],
                        attrs! {At::Type => "text", At::Placeholder => "Meal description" },
                        id!["mdesc"],
                        input_ev(Ev::Input, Msg::MealCreateUpdateDescription),
                        model.meal_under_construction.description,
                    ],
                ],
            ],
        ],
        button![
            submit_text,
            simple_ev(
                Ev::Click,
                if model.page == Pages::CreateMeal {
                    Msg::CreateNewMeal(model.meal_under_construction.clone())
                } else {
                    let mut m = model.meal_under_construction.clone();
                    if let Pages::EditMeal { meal_id } = model.page {
                        m.id = meal_id
                    }
                    Msg::SaveMeal(m)
                }
            )
        ],
        match &model.error {
            Some(e) => h3![format!("error was {}", e)],
            None => empty(),
        },
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

fn nav_nodes(model: &Model) -> Vec<Node<Msg>> {
    vec![
        ul![
            class!["navbar-nav mr-auto"],
            li![
                class![{
                    match model.page {
                        Pages::Home => "nav-item active",
                        _ => "nav-item",
                    }
                }],
                a![
                    "Home",
                    class!["nav-link"],
                    match model.page {
                        Pages::Home => span![class!["sr-only"], "(current)"],
                        _ => empty![],
                    },
                    attrs! {At::Href => "/"},
                ]
            ],
            li![
                class![{
                    match model.page {
                        Pages::Meals { .. }
                        | Pages::CreateMeal
                        | Pages::EditMeal { .. }
                        | Pages::ViewSpecificMeal { .. } => "nav-item active",
                        _ => "nav-item",
                    }
                }],
                a![
                    "Meals",
                    class!["nav-link"],
                    match model.page {
                        Pages::Meals { .. }
                        | Pages::CreateMeal
                        | Pages::EditMeal { .. }
                        | Pages::ViewSpecificMeal { .. } => span![class!["sr-only"], "(current)"],
                        _ => empty![],
                    },
                    attrs! {At::Href => "/meals"}
                ]
            ]
        ],
        a!["Login", class!["nav-link"], attrs! {At::Href => "/login"}],
    ]
}

fn nav(model: &Model) -> Node<Msg> {
    nav![
        class!["navbar navbar-light bg-light navbar-expand-sm"],
        a![
            "refeed rampage",
            class!["navbar-brand"],
            attrs! {At::Href => "/"}
        ],
        button![
            class!["navbar-toggler"],
            attrs! {
                At::Type => "button",
                At::Custom(std::borrow::Cow::Borrowed("data-toggle"))=>"collapse",
                At::Custom(std::borrow::Cow::Borrowed("data-target"))=>"#navbarCollapse"
            },
            span![class!["navbar-toggler-icon"]],
        ],
        div![
            class!["collapse navbar-collapse"],
            id!["navbarCollapse"],
            nav_nodes(model),
        ],
    ]
}

// for a detail view
fn meal_item(m: &Meal) -> Node<Msg> {
    div![
        h4![m.name, div![p![m.description, class!["lead"]],]],
        button![
            simple_ev(Ev::Click, Msg::DeleteMeal { meal_id: m.id }),
            "Delete it"
        ]
    ]
}

fn meal_list(model: &Model) -> Vec<Node<Msg>> {
    let bodies: Vec<Node<Msg>> = model
        .meals
        .iter()
        .map(|m| {
            tr![
                style! {St::Cursor => "pointer"},
                attrs! {At::Href => format!("/meals/{}", m.id)},
                td![button![
                    simple_ev(Ev::Click, Msg::EditMeal { meal_id: m.id }),
                    attrs! {At::Href => format!("/meals/{}/edit", m.id)},
                    "✏️"
                ]],
                td![m.name],
                td![m.description]
            ]
        })
        .collect();

    let l = div![
        class!["table-responsive-sm col-9"],
        table![
            class!["table table-striped table-sm"],
            thead![tr![
                th![attrs! { At::Scope => "col" }],
                th!["name", attrs! { At::Scope => "col" }],
                th!["description", attrs! { At::Scope => "col" }],
            ]],
            tbody![bodies,]
        ]
    ];
    let b = p![button![attrs! {At::Href => "/meals/create"}, "➕"]];
    vec![l, b]
}

// https://seed-rs.org/guide/http-requests-and-state

async fn fetch_meals() -> Result<Msg, Msg> {
    let url = format!("{}/meals", URL_BASE);
    log!(format!("url is {}", url));
    Request::new(url).fetch_json_data(Msg::MealsFetched).await
}

async fn delete_meal(id: Uuid) -> Result<Msg, Msg> {
    let url = format!("{}/meals/{}", URL_BASE, id);
    log!(format!("url is {}", url));
    Request::new(url)
        .method(Method::Delete)
        .fetch_json_data(Msg::MealDeleted)
        .await
}

async fn create_meal(meal: Meal) -> Result<Msg, Msg> {
    let url = format!("{}/meals", URL_BASE);
    log!(format!("Sending something to {}", url));
    Request::new(url)
        .method(Method::Post)
        .send_json(&meal)
        .fetch_json_data(Msg::MealCreated)
        .await
}

async fn update_meal(meal: Meal) -> Result<Msg, Msg> {
    let url = format!("{}/meals/{}", URL_BASE, meal.id);
    log!(format!("Sending something to {}", url));
    Request::new(url)
        .method(Method::Put)
        .send_json(&meal)
        .fetch_json_data(Msg::MealCreated)
        .await
}

async fn fetch_meal(id: Uuid) -> Result<Msg, Msg> {
    let url = format!("{}/meals/{}", URL_BASE.to_string(), id);
    Request::new(url).fetch_json_data(Msg::MealFetched).await
}

fn routes(url: Url) -> Option<Msg> {
    if url.path.is_empty() {
        return Some(Msg::ChangePage(Pages::Home));
    }
    // log!("url path is {}", url.path);
    Some(match url.path[0].as_ref() {
        "meals" => match url.path.get(1).as_ref() {
            Some(page) => {
                if page == &"create" {
                    return Some(Msg::ChangePage(Pages::CreateMeal));
                }
                match page.parse::<Uuid>() {
                    Ok(m_id) => match url.path.get(2).as_ref() {
                        Some(i) => {
                            if i == &"edit" {
                                return Some(Msg::ChangePage(Pages::EditMeal { meal_id: m_id }));
                            }
                            return Some(Msg::ChangePage(Pages::ViewSpecificMeal {
                                meal_id: m_id,
                            }));
                        }
                        None => {
                            return Some(Msg::ChangePage(Pages::ViewSpecificMeal { meal_id: m_id }))
                        }
                    },
                    Err(e) => {
                        log!("Got an error on meal id: {}", e);
                        return Some(Msg::ChangePage(Pages::Meals));
                    }
                }
            }
            None => Msg::ChangePage(Pages::Meals),
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
                _ => match page.parse::<Uuid>() {
                    Ok(m_id) => match url.path.get(2).as_ref() {
                        Some(i) => match i.as_ref() {
                            "edit" => Pages::EditMeal { meal_id: m_id },
                            _ => Pages::ViewSpecificMeal { meal_id: m_id },
                        },
                        None => Pages::ViewSpecificMeal { meal_id: m_id },
                    },
                    Err(_) => Pages::Meals,
                },
            },
            None => Pages::Meals,
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
    id: Uuid,
    photos: Option<String>,
    description: String,
}

impl Meal {
    fn ready_to_submit(&self) -> bool {
        log!("self is {:?}", self);
        if self.description.is_empty() || self.name.is_empty() {
            return false;
        }
        true
    }
}
