# Use Rust official image as builder
FROM rust:1.84-slim as builder

# Create a new empty shell project
WORKDIR /usr/src/app
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install wkhtmltopdf and its dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    wkhtmltopdf \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary from builder
COPY --from=builder /usr/src/app/target/release/rust-md-to-pdf /usr/local/bin/rust-md-to-pdf

# Expose the port the app runs on
EXPOSE 8080

# Run the binary
CMD ["rust-md-to-pdf"] 