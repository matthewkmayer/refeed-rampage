#!/bin/sh

sudo apt update
sudo apt install -y nginx
sudo systemctl enable nginx

# set up the nginx config
