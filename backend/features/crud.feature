Feature: I can perform CRUD operations on meals

  Scenario: I can see all meals
    Given meals exist
    When I request all meals
    Then I see some meals

  Scenario: I can see a specific meal
    Given meals exist
    When I request to see a specific meal
    Then I can see that meal

  Scenario: I can create a new meal
    Given doot
    When doot
    Then doot

  Scenario: I can edit an existing meal
    Given doot
    When doot
    Then doot

  Scenario: I can delete a meal
    Given doot
    When doot
    Then doot
