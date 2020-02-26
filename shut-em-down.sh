#!/bin/sh

# kill leftovers: backend and docker container
killall backend
docker kill rrampage
docker rm rrampage
