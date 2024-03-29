## -----------------------------------------------------------------------------
## Build
## -----------------------------------------------------------------------------
FROM rust:1.66.1-slim-buster as build-stage

RUN apt update && apt install -y --no-install-recommends \
  pkg-config \
  libssl-dev \
  libcurl4-openssl-dev

WORKDIR "/build"

# Install and build crates
COPY Cargo.* /build/
RUN mkdir /build/src && echo "fn main() {}" > /build/src/main.rs
RUN cargo build --release

# Build app
COPY src/ /build/src/
RUN touch src/main.rs && cargo build --release

## -----------------------------------------------------------------------------
## Package
## -----------------------------------------------------------------------------
FROM debian:buster

# Install runtime dependencies
RUN apt update && apt install -y --no-install-recommends \
  ca-certificates \
  libssl1.1 \
  libcurl4

COPY --from=build-stage "/build/target/release/nats-gatekeeper" "/app/nats-gatekeeper"

WORKDIR "/app"
ENTRYPOINT ["/app/nats-gatekeeper"]
