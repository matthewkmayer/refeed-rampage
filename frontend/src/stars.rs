use crate::Msg;
use seed::{prelude::*, *};

pub fn clickable_star(rating: i32, active: bool) -> Node<Msg> {
    if active {
        span![
            "⭐",
            style! {St::Cursor => "pointer"},
            simple_ev(Ev::Click, Msg::MealCreateUpdateStars(rating)),
        ]
    } else {
        span![
            "⭐",
            style! {"color" => "transparent", "text-shadow" => "0 0 0 white", St::Cursor => "pointer"},
            simple_ev(Ev::Click, Msg::MealCreateUpdateStars(rating)),
        ]
    }
}

// each star is clickable and sends a message
pub fn clickable_stars(stars: Option<i32>) -> Node<Msg> {
    let no_stars = p![
        clickable_star(1, false),
        clickable_star(2, false),
        clickable_star(3, false),
        clickable_star(4, false),
        clickable_star(5, false),
    ];
    match stars {
        None => no_stars,
        Some(1) => p![
            clickable_star(1, true),
            clickable_star(2, false),
            clickable_star(3, false),
            clickable_star(4, false),
            clickable_star(5, false),
        ],
        Some(2) => p![
            clickable_star(1, true),
            clickable_star(2, true),
            clickable_star(3, false),
            clickable_star(4, false),
            clickable_star(5, false),
        ],
        Some(3) => p![
            clickable_star(1, true),
            clickable_star(2, true),
            clickable_star(3, true),
            clickable_star(4, false),
            clickable_star(5, false),
        ],
        Some(4) => p![
            clickable_star(1, true),
            clickable_star(2, true),
            clickable_star(3, true),
            clickable_star(4, true),
            clickable_star(5, false),
        ],
        Some(5) => p![
            clickable_star(1, true),
            clickable_star(2, true),
            clickable_star(3, true),
            clickable_star(4, true),
            clickable_star(5, true),
        ],
        _ => no_stars,
    }
}

pub fn stars(stars: Option<i32>) -> Node<Msg> {
    let no_stars = p![
        "⭐⭐⭐⭐⭐",
        style! {"color" => "transparent", "text-shadow" => "0 0 0 white"}
    ];
    match stars {
        None => no_stars,
        Some(1) => p![
            span!["⭐"],
            span![
                "⭐⭐⭐⭐",
                style! {"color" => "transparent", "text-shadow" => "0 0 0 white"}
            ]
        ],
        Some(2) => p![
            span!["⭐⭐"],
            span![
                "⭐⭐⭐",
                style! {"color" => "transparent", "text-shadow" => "0 0 0 white"}
            ]
        ],
        Some(3) => p![
            span!["⭐⭐⭐"],
            span![
                "⭐⭐",
                style! {"color" => "transparent", "text-shadow" => "0 0 0 white"}
            ]
        ],
        Some(4) => p![
            span!["⭐⭐⭐⭐"],
            span![
                "⭐",
                style! {"color" => "transparent", "text-shadow" => "0 0 0 white"}
            ]
        ],
        Some(5) => p!["⭐⭐⭐⭐⭐"],
        _ => no_stars,
    }
}
