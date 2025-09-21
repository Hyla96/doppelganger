# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app
COPY Cargo.toml ./
COPY src/ ./src/

RUN cargo build --release --bin v1

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/v1 ./v1

EXPOSE 3001

CMD ["./v1"]