# --- Build Stage ---
FROM rust:1.81-slim-bookworm as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Create dummy project for dependency caching
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/smartattendance_be*

# Copy source code and build
COPY src ./src
COPY config ./config
RUN cargo build --release

# --- Runtime Stage ---
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies (OpenSSL and CA certificates for HTTPS)
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/smartattendance_be .

# Copy config folder (optional but good for local/dev use)
COPY --from=builder /app/config ./config

# Set default port
ENV PORT=8080

# Run the binary
CMD ["./smartattendance_be"]
