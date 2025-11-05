# Use the official Rust image as a base
FROM xblackxsnakex/tau2c:rust-musl-builder_latest AS builder

# Create a new empty shell project
WORKDIR /usr/src/signet

# Copy the Cargo.toml files and the source code
COPY Cargo.toml Cargo.lock ./
COPY agent/Cargo.toml ./agent/
COPY broker/Cargo.toml ./broker/

RUN mkdir agent/src broker/src && \
    echo "fn main() {}" > agent/src/main.rs && \
    echo "fn main() {}" > broker/src/main.rs && \
    cargo build --release -p broker && \
    rm -rf target/release && \
    rm -rf agent/src broker/src

COPY agent/src/main.rs ./agent/src/
COPY broker/src/main.rs ./broker/src/

# Build the broker
RUN cargo build --release -p broker --target=x86_64-unknown-linux-musl

# Use a smaller, final image
FROM alpine:latest

# Copy the built broker binary from the builder stage
COPY --from=builder /usr/src/signet/target/x86_64-unknown-linux-musl/release/broker /usr/local/bin/broker
COPY --from=builder /lib/ld-musl-x86_64.so.1 /lib/


# Set the command to run the broker
CMD ["broker"]
