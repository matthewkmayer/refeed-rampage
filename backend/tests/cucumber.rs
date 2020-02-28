use cucumber::cucumber;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Meal {
    name: String,
    id: i32,
    photos: Option<String>,
    description: String,
}

pub struct MyWorld {
    // You can use this struct for mutable context in scenarios.
    meals: Vec<Meal>,
    meal: Meal,
}

impl cucumber::World for MyWorld {}
impl std::default::Default for MyWorld {
    fn default() -> MyWorld {
        // This function is called every time a new scenario is started
        MyWorld {
            meals: vec![],
            meal: Meal {
                name: "".to_string(),
                id: 0,
                photos: None,
                description: "".to_string(),
            },
        }
    }
}

// https://github.com/bbqsrc/cucumber-rust
mod example_steps {
    use super::Meal;
    use cucumber::steps;

    // TODO: an AFTER step that clears myworld

    steps!(crate::MyWorld => {
        given "meals exist" |_world, _step| {
            // noop
        };

        when "I request all meals" |world, _step| {
            let resp = reqwest::blocking::get("http://127.0.0.1:3030/meals").unwrap()
            .json::<Vec<Meal>>().unwrap();
            world.meals = resp;
        };

        when "I request to see a specific meal" |world, _step| {
          let resp = reqwest::blocking::get("http://127.0.0.1:3030/meals/1").unwrap()
          .json::<Meal>().unwrap();
          world.meal = resp;
      };

        then "I see some meals" |world, _step| {
            assert_eq!(world.meals.len() > 0, true);
        };

        then "I can see that meal" |world, _step| {
          println!("meal is {:?}", world.meal);
          assert_eq!(world.meal.name.len() > 0, true);
          assert_eq!(world.meal.description.len() > 0, true);
      };
    });
}

cucumber! {
    features: "./features", // Path to our feature files
    world: crate::MyWorld, // The world needs to be the same for steps and the main cucumber call
    steps: &[
        example_steps::steps // the `steps!` macro creates a `steps` function in a module
    ]
}
