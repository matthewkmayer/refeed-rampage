FROM nginx:1.17.8-alpine

ADD frontend/index.html /usr/share/nginx/html/
ADD frontend/pkg/package* /usr/share/nginx/html/pkg/

# WASM bits
