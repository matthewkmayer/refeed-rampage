#!/bin/sh

sudo apt update
sudo apt install -y nginx libssl-dev

# how can we copy the files over before the next steps?
sudo systemctl enable nginx
sudo systemctl enable rrmeals
sudo systemctl start rrmeals