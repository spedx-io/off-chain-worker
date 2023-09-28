# Use an official Rust runtime as a parent image
FROM rust:1.55 as builder

# Set the working directory in the container
WORKDIR /usr/src/app

# Copy the current directory contents into the container
COPY . .

# Build the Rust app
RUN cargo build --release

# Use a minimal alpine image to reduce the final image size
FROM alpine:latest

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/your_binary_name /usr/local/bin

# Run the binary
CMD ["off-chain-worker"]
