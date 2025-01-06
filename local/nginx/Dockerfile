FROM nginx:alpine

RUN apk add --no-cache bash

COPY ./ssl/localhost.crt /etc/ssl/certs/localhost.crt
COPY ./ssl/localhost.key /etc/ssl/private/localhost.key

COPY nginx.conf /etc/nginx/nginx.conf

COPY ./default.conf /etc/nginx/conf.d/default.conf

EXPOSE 80 443

CMD ["nginx", "-g", "daemon off;"]
