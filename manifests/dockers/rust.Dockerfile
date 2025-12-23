ARG APP_NAME
# TARGETARCH is auto-set by Docker buildx (amd64 or arm64)
ARG TARGETARCH=amd64

# Multi-arch chef image (runs natively on both x86_64 and ARM64)
FROM --platform=$BUILDPLATFORM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY apps/ apps/
COPY libs/ libs/
RUN cargo chef prepare --recipe-path recipe.json

# Multi-arch builder with musl support
FROM --platform=$BUILDPLATFORM rust:1-slim AS builder
ARG APP_NAME
ARG TARGETARCH

WORKDIR /app
ENV RUST_BACKTRACE=1

# Install build dependencies and musl toolchain
RUN apt-get update && apt-get install -y \
    musl-tools \
    musl-dev \
    gcc-aarch64-linux-gnu \
    gcc-x86-64-linux-gnu \
    && rm -rf /var/lib/apt/lists/*

# Add musl targets
RUN rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl

# Install cargo-chef
RUN cargo install cargo-chef --locked

COPY --from=planner /app/recipe.json recipe.json

# Build dependencies based on target architecture
RUN if [ "$TARGETARCH" = "amd64" ]; then \
      cargo chef cook --release --recipe-path recipe.json --target x86_64-unknown-linux-musl; \
    else \
      export CC_aarch64_unknown_linux_musl=aarch64-linux-gnu-gcc && \
      cargo chef cook --release --recipe-path recipe.json --target aarch64-unknown-linux-musl; \
    fi

COPY Cargo.toml Cargo.lock ./
COPY apps/ apps/
COPY libs/ libs/

# Build the application
RUN if [ "$TARGETARCH" = "amd64" ]; then \
      cargo build --release -p ${APP_NAME} --target x86_64-unknown-linux-musl && \
      cp /app/target/x86_64-unknown-linux-musl/release/${APP_NAME} /app/binary; \
    else \
      export CC_aarch64_unknown_linux_musl=aarch64-linux-gnu-gcc && \
      cargo build --release -p ${APP_NAME} --target aarch64-unknown-linux-musl && \
      cp /app/target/aarch64-unknown-linux-musl/release/${APP_NAME} /app/binary; \
    fi

FROM scratch AS rust
ARG APP_NAME

COPY --from=builder /app/binary /app

# Environment
ENV PORT=8080
EXPOSE ${PORT}

# Security and metadata labels
LABEL \
    org.opencontainers.image.title="${APP_NAME}" \
    org.opencontainers.image.source="playground" \
    org.opencontainers.image.description="Minimal Rust application from Nx monorepo" \
    security.non-root="true" \
    security.static-binary="true" \
    security.minimal-size="true" \
    security.no-shell="true" \
    security.distroless="false" \
    security.minimal="true" \
    security.base-image="scratch"

ENTRYPOINT ["/app"]
