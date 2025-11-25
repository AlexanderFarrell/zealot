# 1. Build stage

FROM golang:1.24-alpine AS build

RUN apk add --no-cache git ca-certificates tzdata

RUN mkdir /src
WORKDIR /src

# Copy go mod and then download dependencies
COPY go.mod go.sum ./
RUN go mod download

# Copy the rest of the source
COPY . .

# Build
RUN CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o /out/zealotd

# 2. Runtime stage
FROM alpine:3.20

RUN apk add --no-cache ca-certificates tzdata

RUN adduser -D -g '' zealot
USER zealot

WORKDIR /app

COPY --from=build /out/zealotd /usr/local/bin/zealotd

ENV ZEALOT_ENV=prod \
    PORT=8082

EXPOSE 8082

# HEALTHCHECK --interval

CMD ["zealotd"]