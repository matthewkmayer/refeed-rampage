## Technology compatability kit

Standin for building and running the real deal. Mocks the backend for us.

We don't want to reimplement the backend in another language, we want something that responds similarly to the real backend service. This allows us to test the frontend code without having to pay the compilation price of the backend.

Right now we're running

From the root of the project, run `duty` to provide mock responses:

`$ docker run -it -v $PWD/tck/duty.yaml:/duty.yaml -v $PWD/tck/responses:/responses --name duty gomicro/duty`

`avenues` allows us to make the CORS requests because it supports cross-origin requests:

`$ docker run -it -v $PWD/tck/routes.yaml:/routes.yaml -p 3030:4567 --link duty:duty gomicro/avenues`
