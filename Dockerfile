# Use the official Rust image as a base
FROM rust:1.78 as builder

# Create a new empty shell project
WORKDIR /usr/src/signet

# Copy the Cargo.toml files and the source code
COPY Cargo.toml Cargo.lock ./
COPY agent/Cargo.toml ./agent/
COPY broker/Cargo.toml ./broker/
COPY agent/src/main.rs ./agent/src/
COPY broker/src/main.rs ./broker/src/

# Build the broker
RUN cargo build --release -p broker

# Use a smaller, final image
FROM debian:bullseye-slim

# Copy the built broker binary from the builder stage
COPY --from=builder /usr/src/signet/target/release/broker /usr/local/bin/broker

# Set the command to run the broker
CMD ["broker"]
