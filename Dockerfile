# Use the official Rust image as a base
FROM xblackxsnakex/tau2c:rust-musl-builder_latest AS builder

# Create a new empty shell project
WORKDIR /usr/src/signet

# Copy the Cargo.toml files and the source code
COPY Cargo.toml Cargo.lock ./
COPY agent/Cargo.toml ./agent/
COPY broker/Cargo.toml ./broker/

COPY signet/ ./signet/

COPY agent/src/main.rs ./agent/src/
COPY broker/src/main.rs ./broker/src/

# Build the broker
RUN cargo build --release -p broker --target=x86_64-unknown-linux-musl

# Use a smaller, final image
FROM alpine:latest

# Copy the built broker binary from the builder stage
COPY --from=builder /usr/src/signet/target/x86_64-unknown-linux-musl/release/broker /usr/local/bin/broker
COPY --from=builder /lib/ld-musl-x86_64.so.1 /lib/

COPY ca/ca.crt /ca/

# Set the command to run the broker
CMD ["broker"]
