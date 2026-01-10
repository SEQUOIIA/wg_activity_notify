ARG BASE_IMAGE=rust:1.91-bullseye

FROM $BASE_IMAGE as planner
WORKDIR app
RUN cargo install cargo-chef --version 0.1.73
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM $BASE_IMAGE as cacher
WORKDIR app
RUN apt update && apt install -y ca-certificates wget gcc libssl-dev libc6-dev pkg-config
RUN cargo install cargo-chef --version 0.1.73
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM $BASE_IMAGE as builder
RUN apt update && apt install -y ca-certificates wget gcc libssl-dev libc6-dev pkg-config
WORKDIR app
COPY . .
# Copy over the cached dependencies
COPY --from=cacher /app/target target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt update && apt install -y --no-install-recommends wireguard-tools iptables net-tools ca-certificates && apt-get clean && rm -rf /var/lib/apt/lists/*
COPY ./config.yml.example /app/config.yml
COPY --from=builder /app/target/release/wg_activity_notify_daemon /app
WORKDIR /app
CMD ["/bin/sh", "-c", "/app/wg_activity_notify_daemon"]