FROM node:20-alpine AS builder

WORKDIR /app

COPY webapp/package.json webapp/package-lock.json ./
RUN npm ci

COPY webapp/ .
RUN npm run build

FROM nginx:1.27-alpine

COPY webapp/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /app/dist /usr/share/nginx/html

EXPOSE 80
