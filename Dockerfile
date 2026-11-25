FROM rust:1-slim-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/ovh-vps-watcher /usr/local/bin/ovh-vps-watcher
USER nobody
CMD ["ovh-vps-watcher"]
