# Use a Rust base image for building
FROM rust:latest as builder

# Set the working directory inside the container
WORKDIR /usr/src/myapp

# Copy the dependency manifests
COPY Cargo.toml Cargo.lock ./

# Build the dependencies separately to optimize caching
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy the source code
COPY . .

RUN touch src/main.rs

# Build the application
RUN cargo build --release

# Start a new stage with Debian slim
FROM debian:bookworm-slim

# Set the working directory inside the container
WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the built binary from the builder stage
COPY --from=builder /usr/src/myapp/target/release/nutritionist .

COPY --from=builder /usr/src/myapp/templates templates

RUN ls -al
# Expose any necessary ports
EXPOSE 8080

# Define the command to run the application
CMD ["./nutritionist"]
