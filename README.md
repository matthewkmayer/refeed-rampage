# refeed-rampage

Tracking refeed meals.

![Build](https://github.com/matthewkmayer/refeed-rampage/workflows/Build/badge.svg)

## What is this?

I tend to follow a cyclical ketogenic diet: low carbs six days a week and one refeed day a week that's high in carbs. The refeed day is also known as "rampage day" where *all the carbs* can be consumed.

This project is aimed at recording what I ate, how I liked it (will I eat the food again) and a general log on how I feel during/after the rampage.

## Goals

Serverless, modern web application with as little JavaScript as possible. The plan is to use Rust for the front and back end. This is meant to be a *usable* product so I can get a better understanding of WASM in production.

## Non-goals

I am not planning on:

* providing support for anyone else using this
* making this Amazon scale production ready

## Plans

(deleted as we move along)

2. Have front end be able to fetch and display data from the endpoint
3. Style front end
4. Add endpoints for CRUD operations
5. Frontend CRUD operations
6. Authentication?
7. How to deploy? ECS Fargate is serverless ;)
8. Backing data stores? DynamoDB?
9. TLS via a load balancer or Let's Encrypt?
