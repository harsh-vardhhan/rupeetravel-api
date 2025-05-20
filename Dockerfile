FROM rust:1.85-slim as builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the entire project
COPY . .

# Build the application
RUN cargo build --release

# Runtime image - using the same base image as the builder to ensure glibc compatibility
FROM rust:1.85-slim

# Install runtime dependencies only
RUN apt-get update && apt-get install -y \
    libsqlite3-0 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/flight_api /app/flight_api

# Create an empty database file if it doesn't exist
RUN touch /app/flight_api.db && chmod 666 /app/flight_api.db

# Set the environment variables for Rocket
ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000
ENV ROCKET_LOG_LEVEL=normal
ENV DATABASE_URL=sqlite3.railway.internal

# Expose the port
EXPOSE 8000

# Run the application
CMD ["./flight_api"]