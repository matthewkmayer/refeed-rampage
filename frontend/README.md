## Frontend with seed-rs

https://seed-rs.org

`$ cargo make serve frontend` will make it serve what's built with `$ cargo make build frontend`.

In one window run the `cargo make build frontend` command to keep rebuilding changes and in another, `cargo make serve frontend` to serve them. Open at http://localhost:8080/ .

Docker:

`docker build -t my-service:local .` and `docker run -p 9090:9090 my-service:local` .

## publishing to github pages:

`$ cp index.html ../docs/`
(edit index.html to import and load from `./` instead of `/`)

`$ cp -r pkg ../docs`

Might have to do something with the .gitignore in the `pkg` dir though.

Issue with github pages include having to make your own single page app router handler in the 404 page. See https://github.com/rafrex/spa-github-pages and https://itnext.io/so-you-want-to-host-your-single-age-react-app-on-github-pages-a826ab01e48 for details.

Publishing `docs/` in the master branch will publish to https://matthewkmayer.github.io/refeed-rampage/ .
