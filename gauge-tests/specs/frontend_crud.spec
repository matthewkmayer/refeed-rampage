# Refeed rampage tests

Tests for refeed rampage's frontend. End to end tests until a stubbed backend is sorted.

## See default meals

* Goto refeed rampage home
* Click "Meals"
* Page contains "Pizza"
* Page contains "Burritos"

## See a default meal's detail page

* Goto refeed rampage home
* Click "Meals"
* Click "Pizza"
* Page contains "delicious pizza"
* Page contains "delete it"

## Create a new meal

* Goto refeed rampage home
* Click "Meals"
* Click "➕"
* Fill out wings as a new meal
* Page contains "wings"

## Edit an existing meal

* Goto refeed rampage home
* Click "Meals"
* Click "✏️"
* Update the name to "Thin Crust Pizza" and description to "Thin description"
* Click "Save it"
* Page contains "Thin Crust Pizza"
* Page contains "Thin description"

