FROM golang:1.24-alpine AS build

RUN apk add --no-cache git ca-certificates tzdata

WORKDIR /src

COPY go.mod go.sum ./
RUN go mod download

COPY . .

RUN CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o /out/zealot-mcp .

FROM alpine:3.20

RUN apk add --no-cache ca-certificates tzdata

RUN adduser -D -g '' zealot
USER zealot

WORKDIR /app

COPY --from=build /out/zealot-mcp /usr/local/bin/zealot-mcp

ENV MCP_TRANSPORT=http \
    MCP_LISTEN_ADDR=:8080 \
    MCP_BASE_PATH=/mcp

EXPOSE 8080

CMD ["zealot-mcp"]
