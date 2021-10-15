FROM rustlang/rust:nightly-alpine AS chef

RUN apk add --no-cach musl-dev && \
    cargo install cargo-chef
WORKDIR /app

FROM chef AS planner

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin backend

# FINAL

FROM alpine:latest

COPY --from=builder /app/target/release/backend /usr/local/bin/backend
ENTRYPOINT [ "backend" ]
