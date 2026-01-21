FROM rust:1.78-slim AS builder

WORKDIR /app

COPY simulator/Cargo.toml simulator/Cargo.lock ./
COPY simulator/src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/virality-sim /usr/local/bin/virality-sim
COPY config ./config
COPY webapp/dist ./webapp/dist

ENV RUST_LOG=info

EXPOSE 8787

CMD ["virality-sim", "serve", "--host", "0.0.0.0", "--port", "8787", "--web-root", "/app/webapp/dist"]
