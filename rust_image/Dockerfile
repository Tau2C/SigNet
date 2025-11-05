# Start from a minimal Debian base
FROM debian:bullseye-slim

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    ca-certificates \
    build-essential \
    pkg-config \
    libssl-dev \
    musl-tools \
    git \
    && rm -rf /var/lib/apt/lists/*

# Set environment variables for Rust
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

# Install Rust via rustup
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --no-modify-path

# Add MUSL target (for static linking)
RUN rustup target add x86_64-unknown-linux-musl

# Show installed targets for confirmation
RUN rustup target list --installed && rustc --version && cargo --version

# Set workdir for builds
WORKDIR /usr/src/app
