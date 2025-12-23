ARG APP_NAME

# Use Alpine for native musl support (multi-arch image)
FROM rust:alpine AS chef
RUN apk add --no-cache musl-dev
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY apps/ apps/
COPY libs/ libs/
RUN cargo chef prepare --recipe-path recipe.json

# Builder stage - Alpine has native musl
FROM rust:alpine AS builder
ARG APP_NAME

RUN apk add --no-cache musl-dev

WORKDIR /app
ENV RUST_BACKTRACE=1

# Install cargo-chef
RUN cargo install cargo-chef --locked

COPY --from=planner /app/recipe.json recipe.json

# Build dependencies (native musl - no cross-compilation needed)
RUN cargo chef cook --release --recipe-path recipe.json

COPY Cargo.toml Cargo.lock ./
COPY apps/ apps/
COPY libs/ libs/

# Build the application
RUN cargo build --release -p ${APP_NAME} && \
    cp /app/target/release/${APP_NAME} /app/binary

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
