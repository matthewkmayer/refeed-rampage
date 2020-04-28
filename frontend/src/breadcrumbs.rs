use crate::{Meal, Msg, Pages};
use seed::{prelude::*, *};

pub fn breadcrumbs(page: &Pages, meal_v: &Meal, meal_edit: &Meal) -> Node<Msg> {
  match page {
    Pages::Home => empty!(),
    Pages::Meals => p![
      "meals",
      style! {St::Cursor => "pointer"},
      simple_ev(Ev::Click, Msg::ChangePage(Pages::Meals))
    ],
    Pages::ViewSpecificMeal { meal_id: _ } => p![
      span![
        "meals",
        style! {St::Cursor => "pointer"},
        simple_ev(Ev::Click, Msg::ChangePage(Pages::Meals))
      ],
      span![" > "],
      span![format!(" {}", meal_v.name)],
    ],
    Pages::EditMeal { meal_id: _ } => p![
      span![
        "meals",
        style! {St::Cursor => "pointer"},
        simple_ev(Ev::Click, Msg::ChangePage(Pages::Meals))
      ],
      span![" > "],
      span![
        format!(" {} ", meal_edit.name),
        style! {St::Cursor => "pointer"},
        simple_ev(
          Ev::Click,
          Msg::ChangePage(Pages::ViewSpecificMeal {
            meal_id: meal_edit.id
          })
        )
      ],
      span![" > "],
      span!["edit"]
    ],
    Pages::CreateMeal => p![
      span![
        "meals",
        style! {St::Cursor => "pointer"},
        simple_ev(Ev::Click, Msg::ChangePage(Pages::Meals))
      ],
      span![" > "],
      span!["create new meal"]
    ],
    Pages::Login => empty!(),
  }
}
