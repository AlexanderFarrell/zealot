FROM rust:1.89-bookworm AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

COPY . .

RUN cargo build --release -p zealot-server

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl libsqlite3-0 libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/zealot-server /usr/local/bin/zealot-server

RUN mkdir -p /data/public

ENV DATABASE=sqlite
ENV DB_FILENAME=/data/zealot.db
ENV MEDIA_SOURCE=FILESYSTEM
ENV MEDIA_PATH=/data/public
ENV PORT=8456

EXPOSE 8456

CMD ["sh", "-c", "mkdir -p /data/public && exec /usr/local/bin/zealot-server"]
