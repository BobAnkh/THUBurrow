FROM rustlang/rust:nightly AS chef

RUN apt update -qq && apt install libssl-dev \
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

FROM debian:buster

COPY --from=builder /app/target/release/backend /usr/local/bin/backend
COPY --from=builder /app/conf/log4rs.yml /etc/backend/conf/log4rs.yml
ENTRYPOINT [ "backend" ]
