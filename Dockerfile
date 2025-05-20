FROM rust:1.82-slim-bookworm AS builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs file to build dependencies
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# Build dependencies (this step is cached)
RUN cargo build --release

# Remove the dummy file
RUN rm src/main.rs

# Copy the actual source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libsqlite3-0 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/flight_api /app/flight_api

# Set the environment variable for Railway
ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000

# Expose the port
EXPOSE 8000

# Run the application
CMD ["./flight_api"]