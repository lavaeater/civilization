#!/usr/bin/env bash
#
# Launch the Advanced Civilization multiplayer server (headless game + web
# client + join API + WebSocket, all in one process).
#
# Runs through bash on purpose: bash honours the `VAR=value command` prefix and
# reliably exports env vars, which fish's inline-assignment syntax does not do
# across pasted multi-line commands. Invoke it from fish/zsh/anything — the
# shebang re-enters bash — or explicitly with `bash run-server.sh`.
#
# Every setting has a localhost default and can be overridden by exporting the
# matching variable first, e.g.:
#   SEATS=3 NETCODE_KEY=random ./run-server.sh
#   HOST=192.168.1.50 ./run-server.sh          # LAN play (derives PUBLIC_ADDR/WS)
#
# Flags:
#   --build | -b   (re)build the release server binary before running
#   --help  | -h   show this help
#
set -euo pipefail

# Repo root = the directory this script lives in, so it works from anywhere and
# BEVY_ASSET_ROOT always points at the tree that contains assets/.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

BIN="target/release/adv_civ_server"

case "${1:-}" in
    -h|--help)
        # Print the contiguous header comment block (lines after the shebang,
        # up to the first non-comment line), stripped of the leading "# ".
        awk 'NR>1 && /^#/ {sub(/^# ?/, ""); print; next} NR>1 {exit}' "${BASH_SOURCE[0]}"
        exit 0
        ;;
    -b|--build)
        cargo build --release -p adv_civ_server
        shift
        ;;
esac

if [[ ! -x "$BIN" ]]; then
    echo "run-server.sh: $BIN not found. Build it first with:" >&2
    echo "run-server.sh:   ./run-server.sh --build" >&2
    exit 1
fi

# Host that clients dial for the game socket. Default localhost; set HOST to the
# machine's LAN IP for cross-computer play.
HOST="${HOST:-127.0.0.1}"

export BEVY_ASSET_ROOT="${BEVY_ASSET_ROOT:-$ROOT}"
export SEATS="${SEATS:-2}"
export NUM_PLAYERS="${NUM_PLAYERS:-5}"
export PORT="${PORT:-5111}"
export HTTP_PORT="${HTTP_PORT:-5112}"
export NETCODE_KEY="${NETCODE_KEY:-random}"
export PUBLIC_ADDR="${PUBLIC_ADDR:-$HOST:$PORT}"
export PUBLIC_WS="${PUBLIC_WS:-ws://$HOST:$PORT}"
export CLIENT_DIR="${CLIENT_DIR:-dist}"

echo "==> adv_civ_server"
echo "    asset root  : $BEVY_ASSET_ROOT"
echo "    seats       : $SEATS human of $NUM_PLAYERS players"
echo "    open client : http://$HOST:$HTTP_PORT/?name=YourName"
echo "    websocket   : $PUBLIC_WS"
echo

exec "./$BIN" "$@"
