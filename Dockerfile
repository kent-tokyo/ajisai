# Pinned minimal server image for headless JSON-RPC operation.
FROM rust:1.97.0-bookworm AS builder
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY spec ./spec
RUN cargo build --release --locked --bin ajisai-server

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install --no-install-recommends -y ca-certificates libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 10001 --shell /usr/sbin/nologin ajisai
USER ajisai
WORKDIR /workspace
COPY --from=builder /src/target/release/ajisai-server /usr/local/bin/ajisai-server
ENTRYPOINT ["/usr/local/bin/ajisai-server"]
