## Technology compatability kit

Standin for building and running the real deal. Mocks the backend for us.

`$ docker run -it -v $PWD/tck/duty.yaml:/duty.yaml -v $PWD/tck/responses:/responses -p 4567:456gomicro/duty`
