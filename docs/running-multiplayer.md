# Running the multiplayer server

How to run the Advanced Civilization multiplayer server and get a browser client
in front of players. Three ways, simplest first:

1. **[One binary on a LAN](#a-one-binary-lan-recommended-for-testing)** — the game
   server serves the web client, the join API and the WebSocket itself. Best for
   testing with friends on the same network or across two computers.
2. **[Dev loop](#b-dev-loop-with-hot-reload)** — `trunk serve` for the client with
   hot reload, plus the server. Best while changing client code.
3. **[Docker + Caddy](#c-docker-compose--caddy-tls)** — TLS, one URL, Let's Encrypt.
   Best for a real deployment on a public domain.

For the production home-server plan (behind `pingora-docker` at `civ.kidvhs.com`),
see [`civilization-deploy.md`](../civilization-deploy.md). For the architecture and
protocol design, see [`docs/multiplayer.md`](multiplayer.md).

---

## What the server actually is

`adv_civ_server` is a **headless** build of the full game (real map, all phases, AI
opponents — no window, no rendering). It listens on two TCP ports:

| Port (env)        | Default | Serves                                                        |
|-------------------|---------|--------------------------------------------------------------|
| `PORT`            | `5111`  | The game **WebSocket** (lightyear/netcode).                  |
| `HTTP_PORT`       | `5112`  | The **HTTP** side: `POST /api/join`, `GET /api/health`, and — if a client build is present — the **static web client**. |

Empty seats are filled by the AI, so a game always has a full table. `SEATS=0` is a
pure AI self-play server (nothing to join).

### How a player joins

1. Browser loads the web client from the HTTP port (or from Caddy).
2. The client `POST`s a name to `/api/join`. The server mints a short-lived
   (60 s) netcode **ConnectToken** and replies with it plus the WebSocket URL to
   dial (`ws_url`, controlled by `PUBLIC_WS`).
3. The client opens the WebSocket with that token; the server matches the token's
   client-id to the reserved seat and the player is in the lobby.
4. When every human seat is claimed, the game starts.

### Environment variables

| Variable          | Default              | Meaning                                                                 |
|-------------------|----------------------|-------------------------------------------------------------------------|
| `SEATS`           | `2`                  | Human seats. `0` = AI-only self-play.                                    |
| `NUM_PLAYERS`     | `5`                  | Total players including AI (clamped to 1–9).                             |
| `PORT`            | `5111`               | WebSocket port.                                                         |
| `HTTP_PORT`       | `5112`               | HTTP API + static client port.                                          |
| `NETCODE_KEY`     | *(dev key)*          | `random` (new key each boot), 64 hex chars (fixed key), or unset = all-zero dev key. Use `random` for anything beyond localhost. |
| `PUBLIC_ADDR`     | `127.0.0.1:$PORT`    | Address the ConnectToken is minted for. Set to the address clients dial for the game socket. Must resolve (DNS or IP). |
| `PUBLIC_WS`       | `ws://$PUBLIC_ADDR`  | WebSocket URL advertised to clients in the join response. `ws://host:5111` bare, or `wss://domain/ws` behind Caddy. |
| `CLIENT_DIR`      | `dist`               | Directory of the web client to serve. Missing = HTTP API only.          |
| `BEVY_ASSET_ROOT` | *(exe dir)*          | Must point at the repo root (which contains `assets/`) when running the binary directly. |

The web client also reads URL query params, which override the defaults:
`?name=Alice`, `?api=http://host:5112` (join API base), `?ws=ws://host:5111`
(WebSocket URL). Normally you only need `?name=`.

---

## Prerequisites

```bash
# Rust toolchain + the wasm target for the web client
rustup target add wasm32-unknown-unknown

# trunk builds the wasm client (https://trunkrs.dev)
cargo install --locked trunk

# For path C only:
#   docker + docker compose
```

Run all commands from the repository root.

---

## A. One binary, LAN (recommended for testing)

The server serves everything. Good for "run it on my machine, others open a link".

**1. Build the web client** (produces `dist/`, ~33 MB wasm):

```bash
trunk build --release
```

**2. Run the server with `./run-server.sh`.** This bash launcher builds the
binary (with `--build`), sets `BEVY_ASSET_ROOT` to the repo root for you, and
fills in sensible localhost defaults — so a same-machine test is just:

```bash
./run-server.sh --build      # drop --build on later runs; the binary is cached
```

For **LAN / cross-computer** play, set `HOST` to the host machine's IP; the
script derives `PUBLIC_ADDR` and `PUBLIC_WS` from it:

```bash
HOST=192.168.1.50 ./run-server.sh
```

Any variable can be overridden the same way (`SEATS=3 NETCODE_KEY=random
./run-server.sh`). See `./run-server.sh --help` for the full list.

On boot it prints a summary and then logs `Serving web client from dist on port
5112` and `WebSocket game server listening on 0.0.0.0:5111 (advertised to
clients as …)`. If you see `Path not found: …/assets/…`, `BEVY_ASSET_ROOT`
didn't reach the process — use the script, which sets it reliably.

> ⚠️ **Shell syntax (fish/zsh users):** the `VAR=value command` prefix and
> backslash line-continuations are a **bash** feature. In **fish** a pasted
> multi-line `VAR=… \` block silently runs with the vars unset (you'll get the
> `Path not found` asset errors). That's the whole reason `run-server.sh` exists
> — its shebang re-enters bash regardless of your interactive shell. If you'd
> rather not use the script, either run the raw command through bash
> (`bash -c '…'`) or, in fish, export first with `set -x` (one var per line):
>
> ```fish
> set -x BEVY_ASSET_ROOT $PWD
> ./target/release/adv_civ_server
> ```

**3. Players open the link** in a browser — for a same-machine test (two browser
windows), that's just `localhost`:

```
http://localhost:5112/?name=Alice
http://localhost:5112/?name=Bob
```

Over a LAN, use the host's IP instead (and see the Ports note below):

```
http://192.168.1.50:5112/?name=Alice
http://192.168.1.50:5112/?name=Bob
```

Each picks a seat; when all `SEATS` humans have joined, the game starts and the AI
takes the rest.

> **Ports:** the two computers must be able to reach `5112` **and** `5111` on the
> host. Open both in the host firewall (`sudo ufw allow 5111,5112/tcp` on Ubuntu).
> `PUBLIC_ADDR`/`PUBLIC_WS` must use an address the *other* machine can reach — not
> `127.0.0.1`.

**Quick smoke test from the command line** (no browser needed) — mint a token and
connect the headless CLI client:

```bash
TOKEN=$(curl -s -X POST localhost:5112/api/join -d '{"name":"Tester"}' \
        | python3 -c "import json,sys; print(json.load(sys.stdin)['connect_token'])")
cargo run --release -p adv_civ_server --bin spike_client -- Tester --auto --token "$TOKEN"
```

It should print `✓ Seated as … (Egypt)` and the phases scrolling by.

---

## B. Dev loop with hot reload

While iterating on client code, let `trunk serve` rebuild the client on save
(served on `:8080`) and run the server separately.

```bash
# terminal 1 — the server (no need for it to serve the client here)
cargo build --release -p adv_civ_server
BEVY_ASSET_ROOT="$PWD" SEATS=1 NUM_PLAYERS=4 NETCODE_KEY=random \
  PUBLIC_ADDR=127.0.0.1:5111 PUBLIC_WS=ws://127.0.0.1:5111 \
  ./target/release/adv_civ_server

# terminal 2 — the client with hot reload
trunk serve
```

Because the client is now on a *different* origin (`:8080`) from the API (`:5112`),
point it at the API and socket explicitly:

```
http://localhost:8080/?name=Alice&api=http://localhost:5112&ws=ws://localhost:5111
```

(The `/api/join` endpoint sends permissive CORS headers, so the cross-origin call
from `:8080` works.)

The **native desktop client** can also connect, driven by env vars — handy for a
quick check without a browser:

```bash
AUTO_ONLINE=1 JOIN_URL=http://localhost:5112 PLAYER_NAME=Alice cargo run --release
```

---

## C. Docker Compose + Caddy (TLS)

For a real deployment on a domain. Caddy terminates TLS, serves the client from
`./dist`, and path-routes `/api/*` → server:5112, `/ws` → server:5111.

```bash
# 1. Build the web client into ./dist (Caddy mounts it read-only)
trunk build --release

# 2. Bring up the stack. CIV_DOMAIN=localhost uses Caddy's internal CA;
#    a real domain gets automatic Let's Encrypt certificates.
CIV_DOMAIN=localhost docker compose up --build
```

Then open `https://localhost/?name=Alice` (accept the internal-CA warning for
`localhost`). For a public domain:

```bash
CIV_DOMAIN=civ.example.com SEATS=3 NETCODE_KEY=random docker compose up --build -d
```

Players just open `https://civ.example.com/?name=Alice` — same origin, so no `api`
or `ws` params needed. The compose file already sets `PUBLIC_WS=wss://$CIV_DOMAIN/ws`.

Notes:
- The game image is the headless server only (`Dockerfile`, `--profile dist`); the
  client `dist/` is served by the Caddy container, not baked into the image.
- Server-side saves persist in the `saves` Docker volume across restarts.
- `docker compose down` stops it; add `-v` to also wipe volumes.

---

## Troubleshooting

- **`No web client at dist (CLIENT_DIR) — HTTP API only`** — you didn't run
  `trunk build --release`, or you're running the binary from a directory without a
  `dist/`. Build the client, or set `CLIENT_DIR=/path/to/dist`.
- **`Path not found: .../assets/...`** — `BEVY_ASSET_ROOT` didn't reach the
  process, so it fell back to the exe dir (`target/release/assets`, which doesn't
  exist). Usually a **fish/zsh** shell eating the `VAR=value` prefix — use
  `./run-server.sh`, which sets it reliably, or export it first in fish with
  `set -x BEVY_ASSET_ROOT $PWD`.
- **`405 Method Not Allowed` on `POST /api/join`** — the client is posting to the
  wrong port. That response comes from `trunk serve` (:8080), which has no API.
  Load the client from the server's HTTP port (`http://HOST:5112/…`), or if you're
  deliberately using `trunk serve`, pass `?api=http://HOST:5112&ws=ws://HOST:5111`
  (see path B). Either way the `adv_civ_server` process must actually be running.
- **Browser loads but never connects / "invalid ConnectToken"** — the token
  expired (60 s) or `PUBLIC_WS` points somewhere the browser can't reach. Check the
  join response: `curl -s -X POST http://HOST:5112/api/join -d '{"name":"x"}'` and
  confirm `ws_url` is an address the client machine can open.
- **Another machine can't connect** — `PUBLIC_ADDR`/`PUBLIC_WS` are probably still
  `127.0.0.1`; use the host's LAN IP, and open ports `5111`+`5112` in the firewall.
- **`all seats taken`** — every human seat is filled. Raise `SEATS`, or restart the
  server (each process hosts one game).
