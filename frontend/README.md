## Frontend with seed-rs

https://seed-rs.org

`$ cargo make serve frontend` will make it serve what's built with `$ cargo make build frontend`.

In one window run the `cargo make build frontend` command to keep rebuilding changes and in another, `cargo make serve frontend` to serve them. Open at http://localhost:8080/ .

Docker:

`docker build -t my-service:local .` and `docker run -p 9090:9090 my-service:local` .

## publishing to github pages:

`$ cp index.html ../docs/`
`$ cp -r pkg ../docs`
