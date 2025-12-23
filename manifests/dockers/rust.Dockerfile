ARG APP_NAME
# TARGETARCH is auto-set by Docker buildx (amd64 or arm64)
ARG TARGETARCH=amd64

FROM rust:1 AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY apps/ apps/
COPY libs/ libs/
RUN cargo chef prepare --recipe-path recipe.json

# Architecture-specific builder images
FROM messense/rust-musl-cross:x86_64-musl AS builder-amd64
FROM messense/rust-musl-cross:aarch64-musl AS builder-arm64

# Select builder based on target architecture
FROM builder-${TARGETARCH} AS builder
ARG APP_NAME
ARG TARGETARCH

WORKDIR /app
ENV RUST_BACKTRACE=1

# Set Rust target based on architecture
ENV RUST_TARGET_amd64=x86_64-unknown-linux-musl
ENV RUST_TARGET_arm64=aarch64-unknown-linux-musl

COPY --from=planner /app/recipe.json recipe.json
RUN cargo install cargo-chef --locked

# Build dependencies
RUN if [ "$TARGETARCH" = "amd64" ]; then \
      cargo chef cook --release --recipe-path recipe.json --target x86_64-unknown-linux-musl; \
    else \
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
