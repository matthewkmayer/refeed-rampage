## Technology compatability kit

Standin for building and running the real deal. Mocks the backend for us.

From the root of the project:

`$ docker run -it -v $PWD/tck/duty.yaml:/duty.yaml -v $PWD/tck/responses:/responses --name duty gomicro/duty`

Avenues:

`$ docker run -it -v $PWD/tck/routes.yaml:/routes.yaml -p 3030:4567 --link duty:duty gomicro/avenues`