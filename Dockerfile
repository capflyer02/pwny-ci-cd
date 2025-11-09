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
# CHANGE "pwny-ci-cd" below if your package/binary name is different
COPY --from=builder /app/target/release/pwny-ci-cd /usr/local/bin/pwny-ci-cd

USER appuser

ENTRYPOINT ["/usr/local/bin/pwny-ci-cd"]

EXPOSE 8080

