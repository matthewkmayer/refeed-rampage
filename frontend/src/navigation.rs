use crate::{Model, Msg, Pages};
use seed::{prelude::*, *};

pub fn home() -> Vec<Node<Msg>> {
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

pub fn nav_nodes(model: &Model) -> Vec<Node<Msg>> {
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
        match model.auth {
            Some(_) => a![
                "Logout",
                simple_ev(Ev::Click, Msg::Logout),
                style! {St::Cursor => "pointer"},
                class!["nav-link"]
            ], // later we can put user name in here: "log out Matthew"
            None => a!["Login", class!["nav-link"], attrs! {At::Href => "/login"}],
        },
    ]
}

pub fn nav(model: &Model) -> Node<Msg> {
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
