# ── Build stage ───────────────────────────────────────────────────────────────
FROM rust:1-slim-bookworm AS builder

# paho-mqtt uses the "bundled" feature which compiles the Paho C library from
# source, so cmake and a C/C++ compiler are required at build time.
RUN apt-get update && apt-get install -y --no-install-recommends \
        cmake \
        make \
        gcc \
        g++ \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests first so that Cargo can download and compile all dependencies
# as a separate Docker layer. This layer is only rebuilt when Cargo.toml /
# Cargo.lock change, making subsequent builds significantly faster.
COPY Cargo.toml Cargo.lock ./

# Build throwaway stubs so that Cargo resolves and compiles all dependencies.
# An empty lib.rs and a minimal main.rs are valid Rust, so the build succeeds
# and the compiled dependency artefacts land in /app/target.
RUN mkdir -p src \
    && echo 'fn main(){}' > src/main.rs \
    && touch src/lib.rs \
    && cargo build --release \
    && rm -rf src

# Copy the real source tree and rebuild. Only the crate itself is recompiled;
# the dependency artefacts produced above are reused from the cache.
COPY src ./src
RUN touch src/main.rs src/lib.rs \
    && cargo build --release

# ── Runtime stage ─────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/vda5050_vehicle_simulator ./

# Embed a default config.toml. Mount your own file over this path at runtime
# (see docker-compose.yml) to customise broker address, vehicle parameters, etc.
COPY config.toml ./

CMD ["./vda5050_vehicle_simulator"]
