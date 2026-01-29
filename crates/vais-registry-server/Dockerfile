# Build stage
FROM rust:1.82-bookworm AS builder

WORKDIR /app
COPY . .

RUN cargo build --release --bin vais-registry-server

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/vais-registry-server /app/vais-registry-server
COPY --from=builder /app/crates/vais-registry-server/static /app/static

RUN mkdir -p /app/data/packages

ENV VAIS_REGISTRY_HOST=0.0.0.0
ENV VAIS_REGISTRY_PORT=3000
ENV VAIS_REGISTRY_DB=/app/data/registry.db
ENV VAIS_REGISTRY_STORAGE=/app/data/packages
ENV VAIS_REGISTRY_CORS_ALL=true
ENV VAIS_REGISTRY_LOGGING=true

EXPOSE 3000

VOLUME ["/app/data"]

CMD ["/app/vais-registry-server"]
