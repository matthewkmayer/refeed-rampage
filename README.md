# refeed-rampage (archived)

Tracking refeed meals.

![Build](https://github.com/matthewkmayer/refeed-rampage/workflows/Build/badge.svg)

## Project status

This project has been put on hold as of Nov 2020. The repository will stick around for reference but the project has been shelved.

## What is this?

I tend to follow a cyclical ketogenic diet: low carbs six days a week and one refeed day a week that's high in carbs. The refeed day is also known as "rampage day" where *all the carbs* can be consumed.

This project is aimed at recording what I ate, how I liked it (will I eat the food again) and a general log on how I feel during/after the rampage.

## Goals

Modern web application with as little JavaScript as possible. The plan is to use Rust for the front and back end. This is meant to be a *usable* product so I can get a better understanding of WASM in production. Also: be inexpensive to run.

The site is deployed at https://rampage.screaming3d.com/ . (I knew keeping an awesome domain name around for so long would pay off.)

Local dev link: http://127.0.0.1:8080/ .

## Non-goals

I am not planning on:

* providing support for anyone else using this
* making this Amazon scale production ready

## Repository structure

### Shared types

The [shared types](shared/) are put in their own project. This enforces the backend and frontend to have to same type with one authoritative source. It uses cargo feature flags to opt-in for behavior required by the backend but not the frontend. For example, DynamoDB traits via [dynomite](https://github.com/softprops/dynomite).

### Backend

A [warp](https://github.com/seanmonstar/warp) web server, using DynamoDB as the backing data store.

### Frontend

A WASM single page web app using [Seed](https://github.com/seed-rs/seed).

## Running locally

[`cargo make`](https://github.com/sagiegurari/cargo-make) is required: install by running `cargo install --force cargo-make` .

1. Start local DynamoDB, S3-like service and HTTP server for frontend with `docker-compose up -d`
2. Start backend service: change directory into `backend` and run `RUST_LOG="backend" AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY cargo run` .
3. Run frontend code with `cargo make watch frontend` . This will watch for any changes and compile it to be served by the server started above.
4. Open http://127.0.0.1:8080/ and click around.
