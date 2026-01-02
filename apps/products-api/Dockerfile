# Build stage
FROM rust:1.83-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY libs ./libs
COPY apps/products-api ./apps/products-api

# Build the application
RUN cargo build --release -p products_api

# Runtime stage
FROM alpine:3.20

# Install runtime dependencies
RUN apk add --no-cache ca-certificates libgcc

WORKDIR /app

# Copy the binary
COPY --from=builder /app/target/release/products_api /app/products_api

# Set environment variables
ENV APP_PORT=3003
ENV GRPC_PORT=50051
ENV RUST_LOG=info

EXPOSE 3003 50051

CMD ["/app/products_api"]
