# Headless multiplayer game server (see docs/multiplayer.md).
#
#   docker build -t adv-civ-server .
#   docker run -p 5111:5111 -p 5112:5112 -e SEATS=2 -e NUM_PLAYERS=5 adv-civ-server
#
# Production runs behind the Caddy front in docker-compose.yml, which
# terminates TLS; the server itself speaks plain ws:// + http://.

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

FROM debian:bookworm-slim
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
