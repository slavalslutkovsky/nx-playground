# Migration Container
# Builds the Sea-ORM migration binary for running database migrations
# Used in: Docker Compose init service, Kubernetes Jobs, CI/CD pipelines

# =============================================================================
# Chef Stage - Prepare recipe for dependency caching
# =============================================================================
FROM rust:1 AS chef
RUN cargo install cargo-chef
WORKDIR /app

# =============================================================================
# Planner Stage - Generate recipe.json
# =============================================================================
FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY apps/ apps/
COPY libs/ libs/
RUN cargo chef prepare --recipe-path recipe.json

# =============================================================================
# Builder Stage - Build dependencies then source
# =============================================================================
FROM messense/rust-musl-cross:x86_64-musl AS builder

WORKDIR /app
ENV RUST_BACKTRACE=1

# Install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo install cargo-chef --locked

# Cook dependencies (cached layer)
RUN cargo chef cook --release --recipe-path recipe.json --target x86_64-unknown-linux-musl

# Copy source and build
COPY Cargo.toml Cargo.lock ./
COPY apps/ apps/
COPY libs/ libs/

RUN cargo build --release -p migration --target x86_64-unknown-linux-musl

# =============================================================================
# Runtime Stage - Minimal image
# =============================================================================
FROM scratch

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/migration /migration

# Metadata labels
LABEL \
    org.opencontainers.image.title="migration" \
    org.opencontainers.image.source="zerg" \
    org.opencontainers.image.description="Sea-ORM database migration runner" \
    security.non-root="true" \
    security.static-binary="true" \
    security.minimal-size="true" \
    security.no-shell="true" \
    security.base-image="scratch"

ENTRYPOINT ["/migration"]
CMD ["up"]
