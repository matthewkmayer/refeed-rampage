FROM amazonlinux:2.0.20200207.1

RUN amazon-linux-extras install -y nginx1

RUN systemctl enable nginx
# RUN systemctl start nginx

# install HTML

# WASM bits
