# Headless multiplayer game server + web client (see docs/multiplayer.md and
# civilization-deploy.md). Two build targets from one Dockerfile:
#
#   docker build --target server -t civilization-server .
#   docker run -p 5111:5111 -p 5112:5112 -e SEATS=2 -e NUM_PLAYERS=5 civilization-server
#
#   docker build --target web -t civilization-web .
#   docker run -p 80:80 civilization-web
#
# `server` is the default target (matches the previous single-target
# Dockerfile's `docker build -t adv-civ-server .` still working unchanged).
#
# Production (pingora-docker, civilization-deploy.md): pingora terminates TLS
# for the public domain and forwards plain HTTP to `web`, an internal Caddy
# that path-routes `/api/*` and `/ws` to `server` and serves the static wasm
# client for everything else — see deploy/Caddyfile.internal.
#
# The standalone `docker-compose.yml` in this repo runs a different Caddy
# (deploy/Caddyfile) that *does* terminate TLS itself, for a deployment not
# sitting behind pingora — that path is unaffected by this file.

FROM rust:1-bookworm AS builder
WORKDIR /build

# bevy's Linux build dependencies (audio via cpal/alsa, udev via winit/gilrs)
# build-essential: some rust:1 image variants ship without binutils' ld.
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential pkg-config libasound2-dev libudev-dev \
    libwayland-dev libxkbcommon-dev \
    && rm -rf /var/lib/apt/lists/*

COPY . .
# The dist profile is the opt-level-3 distribution build (the release
# profile is tuned for wasm size instead — see the workspace Cargo.toml).
RUN cargo build --profile dist -p adv_civ_server --bin adv_civ_server

# ── web-builder: trunk build of the wasm client ──────────────────────────────
# Separate stage from `builder` (not reused) so the `server` target's image
# doesn't pay for wasm toolchain installation it never needs — Docker only
# builds the stages a given --target actually depends on.
FROM rust:1-bookworm AS web-builder
WORKDIR /build

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential pkg-config libx11-dev libasound2-dev libudev-dev \
    && rm -rf /var/lib/apt/lists/*
RUN rustup target add wasm32-unknown-unknown
# Pinned to match .github/workflows/deploy-page.yaml's `trunk-action` version
# range closely enough; `--locked` keeps the exact resolved deps reproducible.
RUN cargo install --locked trunk

COPY . .
# .dockerignore excludes .cargo/ entirely (its Linux entry forces the mold
# linker, which isn't installed here) — but that also drops the
# [target.wasm32-unknown-unknown] rustflags .cargo/config.toml sets on the
# host, which getrandom needs to build for wasm at all. Recreate just that
# one flag here rather than un-ignoring the whole file.
RUN mkdir -p .cargo && printf '[target.wasm32-unknown-unknown]\nrustflags = ["--cfg", "getrandom_backend=\\"wasm_js\\""]\n' > .cargo/config.toml
RUN trunk build --release

# ── web: static client + internal (non-TLS) path router ────────────────────
FROM caddy:2 AS web
COPY --from=web-builder /build/dist /srv
COPY deploy/Caddyfile.internal /etc/caddy/Caddyfile
EXPOSE 80

# ── server: the headless game, unchanged from the original single-stage build.
# Last stage = Docker's default --target, matching the original file's plain
# `docker build -t adv-civ-server .` still working unchanged.
FROM debian:bookworm-slim AS server
WORKDIR /app

# Runtime linkage only: the headless server never opens an audio device,
# but the binary links libasound via bevy_kira_audio's dependency chain.
RUN apt-get update && apt-get install -y --no-install-recommends \
    libasound2 libudev1 libwayland-client0 libxkbcommon0 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/dist/adv_civ_server /app/adv_civ_server
COPY assets/definitions /app/assets/definitions
COPY assets/maps/civilization.map.ron /app/assets/maps/civilization.map.ron

ENV BEVY_ASSET_ROOT=/app
# Game WebSocket / HTTP join API
EXPOSE 5111 5112

# Server-side saves land here; mount a volume to survive restarts.
VOLUME /app/saves

CMD ["/app/adv_civ_server"]
