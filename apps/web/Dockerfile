FROM node:22-bookworm-slim AS builder

WORKDIR /app

COPY . .

RUN npm ci
RUN npm run build -w @zealot/web

FROM nginx:1.27-alpine AS runtime

COPY apps/web/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /app/apps/web/dist /usr/share/nginx/html

EXPOSE 80
