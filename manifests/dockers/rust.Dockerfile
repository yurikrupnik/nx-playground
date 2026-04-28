ARG APP_NAME

FROM rust:1 AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY apps/ apps/
COPY libs/ libs/
RUN cargo chef prepare --recipe-path recipe.json

FROM messense/rust-musl-cross:x86_64-musl AS builder
ARG APP_NAME

WORKDIR /app
ENV RUST_BACKTRACE=1

COPY --from=planner /app/recipe.json recipe.json
RUN cargo install cargo-chef --locked
RUN cargo chef cook --release --recipe-path recipe.json --target x86_64-unknown-linux-musl

COPY Cargo.toml Cargo.lock ./
COPY apps/ apps/
COPY libs/ libs/

RUN cargo build --release -p ${APP_NAME} --target x86_64-unknown-linux-musl

FROM scratch AS rust
ARG APP_NAME

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/${APP_NAME} /app

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
