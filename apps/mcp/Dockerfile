FROM rust:1.89-bookworm AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY . .

RUN cargo build --release -p zealot-mcp

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/zealot-mcp /usr/local/bin/zealot-mcp

ENV ZEALOT_URL=http://localhost:7377
ENV MCP_PORT=3100

EXPOSE 3100

CMD ["zealot-mcp", "--mode", "http"]
