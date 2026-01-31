# Build stage
FROM rust:1.75-slim-bookworm as builder

WORKDIR /app

# Install OpenSSL development libraries
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy main to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Copy actual source code
COPY src ./src
COPY abi ./abi

# Build for release
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install CA certificates and OpenSSL runtime
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/rollup-proof-status /app/rollup-proof-status

# Copy ABI files
COPY abi ./abi

EXPOSE 8080

CMD ["/app/rollup-proof-status"]
