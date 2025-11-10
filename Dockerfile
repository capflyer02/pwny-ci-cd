# syntax=docker/dockerfile:1

############################
# 1) Build stage
############################
FROM rust:1-slim AS builder

WORKDIR /app

# Copy manifest and source
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build release binary
RUN cargo build --release

############################
# 2) Runtime stage
############################
FROM debian:stable-slim AS runtime

# Create non-root user
RUN useradd -m appuser

WORKDIR /app

# Copy compiled binary from builder
# If your binary isn't named pwny-ci-cd, change this:
COPY --from=builder /app/target/release/pwny-ci-cd /usr/local/bin/pwny-ci-cd

USER appuser

# Expose the port the app listens on
EXPOSE 8080

# Run the web server
ENTRYPOINT ["/usr/local/bin/pwny-ci-cd"]

