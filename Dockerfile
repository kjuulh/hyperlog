# hyperlog-serve — the gRPC + http server (Postgres-backed). Built per-arch in
# CI (.woodpecker/images.yaml) and fused into a multi-arch manifest. Mirrors the
# forage controller pattern: sccache via memcached (build_arg, skipped locally),
# arch-detected, SQLX_OFFLINE (runtime queries only). protoc is added for the
# tonic_build step in hyperlog-protos.
FROM rust:1.95-slim-trixie AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

ARG SCCACHE_VERSION=0.9.1
ARG SCCACHE_MEMCACHED_ENDPOINT=
ARG TARGETARCH
RUN case "$TARGETARCH" in \
      amd64) SCCACHE_TARGET=x86_64-unknown-linux-musl ;; \
      arm64) SCCACHE_TARGET=aarch64-unknown-linux-musl ;; \
      *) echo "unsupported TARGETARCH: $TARGETARCH" && exit 1 ;; \
    esac && \
    curl -fsSL "https://github.com/mozilla/sccache/releases/download/v${SCCACHE_VERSION}/sccache-v${SCCACHE_VERSION}-${SCCACHE_TARGET}.tar.gz" \
      | tar xz -C /tmp && \
    install -m 0755 "/tmp/sccache-v${SCCACHE_VERSION}-${SCCACHE_TARGET}/sccache" /usr/local/bin/sccache && \
    sccache --version
ENV SCCACHE_MEMCACHED_ENDPOINT=$SCCACHE_MEMCACHED_ENDPOINT

# Migrations are embedded via sqlx::migrate! and queries use runtime
# sqlx::query() (no query! macros), so no DB is needed at build time.
ENV SQLX_OFFLINE=true

COPY . .

# Scope the build to the server binary so the TUI crate's deps don't land in
# the image. sccache only when the endpoint is provided (empty = local build).
RUN if [ -n "$SCCACHE_MEMCACHED_ENDPOINT" ]; then \
      export RUSTC_WRAPPER=sccache; \
      echo "==> sccache enabled (endpoint=$SCCACHE_MEMCACHED_ENDPOINT)"; \
    else \
      echo "==> sccache disabled (no SCCACHE_MEMCACHED_ENDPOINT)"; \
    fi && \
    cargo build --release --bin hyperlog-serve && \
    if [ -n "$SCCACHE_MEMCACHED_ENDPOINT" ]; then sccache --show-stats; fi

FROM debian:trixie-slim AS production

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/hyperlog-serve /usr/local/bin/hyperlog-serve

# Listen on all interfaces in a container. Provide DATABASE_URL and
# HYPERLOG_JWT_SECRET at runtime; migrations run on startup.
ENV EXTERNAL_GRPC_HOST=0.0.0.0:4000 \
    EXTERNAL_HOST=0.0.0.0:3000 \
    INTERNAL_HOST=0.0.0.0:3001
EXPOSE 4000 3000 3001
ENTRYPOINT ["/usr/local/bin/hyperlog-serve"]
