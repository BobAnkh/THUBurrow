FROM rust:1.61.0-slim-buster AS chef

RUN apt update -qq && \
    apt install --no-install-recommends libssl-dev pkg-config -y && \
    rm -rf /var/lib/apt/lists/* && \
    cargo install cargo-chef
    # git clone https://github.com/rui314/mold.git && \
    # cd mold && \
    # git checkout v1.0.2 && \
    # make -j$(nproc) CXX=clang++ && \
    # sudo make install
WORKDIR /app

FROM chef AS planner

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release -p backend -p task-executor

# FINAL
FROM debian:buster-slim

RUN apt update -qq && \
    apt install --no-install-recommends libssl-dev curl procps -y && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/backend /usr/local/bin/backend
COPY --from=builder /app/target/release/task-executor /usr/local/bin/task-executor
COPY --from=builder /app/Rocket.toml /etc/backend/Rocket.toml
COPY --from=builder /app/conf/log4rs.yml /etc/backend/conf/log4rs-default.yml
COPY --from=builder /app/docker-entrypoint.sh /

RUN  chmod +x /docker-entrypoint.sh

ENV ROCKET_CONFIG /etc/backend/Rocket.toml

ENTRYPOINT [ "/docker-entrypoint.sh" ]
