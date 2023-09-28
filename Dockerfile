# Use an official Rust runtime as a parent image
FROM rust:1.57 as builder

# Set the working directory in the image to /usr/src/app
WORKDIR /usr/src/app

# Copy the current directory contents into the container at /usr/src/app
COPY . .

# Build the Rust app
RUN cargo build --release

# Use a minimal alpine image to reduce the final image size
FROM alpine:latest

# Copy the binary from the builder
COPY --from=builder /usr/src/app/target/release/your_binary_name /usr/local/bin/

# Run your binary
CMD ["your_binary_name"]
