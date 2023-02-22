FROM rust:slim-bullseye AS builder
RUN apt-get update -y && export DEBIAN_FRONTEND=noninteractive && apt-get install -y lld clang
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release

FROM debian:bullseye-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/zero2prod .
COPY --from=builder /app/settings ./settings
ENV APP_ENV=prod
ENTRYPOINT ["./zero2prod"]
