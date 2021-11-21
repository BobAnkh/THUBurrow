FROM rustlang/rust:nightly-buster-slim AS chef

RUN apt update -qq && \
    apt install --no-install-recommends libssl-dev pkg-config -y && \
    rm -rf /var/lib/apt/lists/* && \
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
FROM debian:buster-slim

RUN apt update -qq && \
    apt install --no-install-recommends libssl-dev -y && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/backend /usr/local/bin/backend
COPY --from=builder /app/Rocket.toml /etc/backend/Rocket.toml
COPY --from=builder /app/conf/log4rs.yml /etc/backend/conf/log4rs-default.yml

ENV ROCKET_CONFIG /etc/backend/Rocket.toml

ENTRYPOINT [ "backend" ]
