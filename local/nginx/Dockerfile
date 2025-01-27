FROM nginx:alpine

RUN apk add --no-cache bash

COPY ./ssl/localhost.pem /etc/ssl/certs/localhost.pem
COPY ./ssl/localhost-key.pem /etc/ssl/private/localhost-key.pem

COPY nginx.conf /etc/nginx/nginx.conf

COPY ./default.conf /etc/nginx/conf.d/default.conf

EXPOSE 80 443

CMD ["nginx", "-g", "daemon off;"]
