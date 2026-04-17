# --- Build Stage ---
FROM rust:1.85-slim-bookworm as builder

WORKDIR /app

# Install ALL build dependencies upfront
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    cmake \
    clang \
    libclang-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

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

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/smartattendance_be .
COPY --from=builder /app/config ./config

ENV PORT=8080

CMD ["./smartattendance_be"]