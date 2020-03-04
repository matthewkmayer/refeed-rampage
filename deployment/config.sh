#!/bin/sh

sudo apt update
sudo apt install -y nginx

# how can we copy the files over before the next steps?
sudo systemctl enable nginx
sudo systemctl enable rrmeals
