FROM rust:1.92-bookworm AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY . .

ENV SQLX_OFFLINE=true
RUN cargo build --release --locked

FROM debian:trixie-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/teamtalk-telegram-sender-rs /app/teamtalk-telegram-sender-rs
COPY config.toml.example /app/config.toml.example

RUN useradd -r -u 10001 -g users appuser
USER appuser

VOLUME ["/app/data"]

ENTRYPOINT ["/app/teamtalk-telegram-sender-rs"]
