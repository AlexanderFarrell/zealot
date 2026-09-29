FROM node:22-bookworm-slim AS builder

WORKDIR /app

COPY . .

RUN npm ci
RUN npm run build -w @zealot/web

FROM nginx:1.27-alpine AS runtime

# The official nginx entrypoint renders templates here using the container
# environment, so deployments can select their API without rebuilding the UI.
ENV API_UPSTREAM=http://server:8456
COPY apps/web/nginx.conf.template /etc/nginx/templates/default.conf.template
COPY --from=builder /app/apps/web/dist /usr/share/nginx/html

EXPOSE 80
