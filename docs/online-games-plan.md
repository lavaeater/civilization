# Online games: create / join, many games at once

_Plan, 26-11-08. Nothing here is implemented yet._

## Goal

From the web client's "Play Online":

1. **Create game**: pick total players (2–9), how many are human (1..total), your name.
   You get a short **code** (`brisk-otter-42`) and a **link**
   (`https://civ.kidvhs.com/?game=brisk-otter-42`) to send to the others.
2. **Join game**: open the link (or type the code), enter a name, and you're in that game's
   lobby.
3. Several such games run on `civ.kidvhs.com` at once, each with its own settings.

Today none of that is possible. `SEATS` / `NUM_PLAYERS` are env vars on one long-running
`adv_civ_server` container, so the live site hosts exactly one game with fixed settings.

## Why one process can't host many games (today)

`GameState` and `GameActivity` are **app-global Bevy states**, and the whole phase machine
hangs off them (`OnEnter(GameActivity::X)` everywhere). One Bevy `World` = one game. This
was already flagged in `multiplayer.md` ("Multiple games per server — the honest caveat").
Making game state per-entity would touch every phase plugin, and that's not worth it for
this.

## Recommendation: a lobby supervisor that runs one game process per game

Keep `adv_civ_server` exactly as it is (one game per process, configured by env), and add a
small **supervisor** in front of it that creates, routes to and reaps those processes.

```
browser ──► pingora ──► civilization-web (Caddy: static client)
                     └► civ-lobby :8080  ── /api/games…           (create / status)
                           │            ── /g/<code>/api/*  ─proxy─► child HTTP_PORT
                           │            ── /g/<code>/ws     ─proxy─► child PORT (WS)
                           └─ spawns:  adv_civ_server  (SEATS, NUM_PLAYERS, PORT,
                                       HTTP_PORT, NETCODE_KEY=random, SAVE_DIR) × N
```

Why this shape:
- **No game-engine refactor.** The game server already reads everything it needs from env.
- **Crash isolation.** One game's panic kills only that game, not every game on the site.
- **The client barely changes.** It already takes `?api=` and `?ws=` overrides
  (`NetworkSettings` in `src/network_client.rs`). A game is just `api = /g/<code>`,
  `ws = wss://domain/g/<code>/ws`.
- **Cheap.** A turn-based headless game idles at near-zero CPU. Memory per process (map,
  card defs, ECS) is the real limit. Measure it, but a small VPS should hold dozens.

### Netcode: one key per game

Every child gets `NETCODE_KEY=random`, so a ConnectToken minted by game A is rejected by
game B even though all games share the same public address (`civ.kidvhs.com:443`).
`PUBLIC_ADDR` stays the same for every child. `PUBLIC_WS` becomes
`wss://civ.kidvhs.com/g/<code>/ws`. The IPv4/IPv6 `internal_addresses()` fix in
`mint_token` already handles the bind-vs-public mismatch.

### The supervisor (`civ_lobby`, new workspace crate)

A small `axum` + `tokio` binary (axum because it needs to proxy a WebSocket upgrade, and
`tiny_http` can't do that). It runs in the same image as `adv_civ_server`, so it can spawn it.

HTTP API:

| Route | Does |
|---|---|
| `POST /api/games` `{total_players, human_seats, host_name}` | Validate (2 ≤ total ≤ 9, 1 ≤ humans ≤ total), check the `MAX_GAMES` cap, pick a free port pair from a pool, generate a code, spawn the child, wait for its `/api/health`. Returns `{code, join_url}`. |
| `GET /api/games/<code>` | `{total_players, human_seats, joined, started, finished}`: lets the client show "3/4 joined" or "game not found". |
| `ANY /g/<code>/api/*` | Reverse-proxy to the child's `HTTP_PORT` (strip the prefix). The existing `POST /api/join` works unchanged. |
| `GET /g/<code>/ws` | WebSocket-upgrade proxy to the child's `PORT`. |
| `GET /api/health` | Supervisor liveness. |

Codes: `adjective-animal-NN` from small word lists (readable aloud, unguessable enough for
"invite friends"), checked for collisions against the registry.

Lifecycle:
- **Registry** on disk (`/app/saves/games.json`: code → settings, ports, created_at,
  state). When the container restarts, games that had started are re-spawned and resume from
  their save (`save_game` already serializes the whole game; each child gets its own
  `SAVE_DIR=/app/saves/<code>/`).
- **Reaping:** if the child exits on game over, mark it finished. An unstarted game expires
  after ~24 h. A started game with no connected humans for ~7 days gets stopped (its save
  kept for a while).
- **Limits:** `MAX_GAMES` (default ~10) and a per-IP create rate limit. That's all the abuse
  protection a friends-only site needs. As `multiplayer.md` already says, the link is a
  bearer capability by design.

### Changes needed in `adv_civ_server`

Small ones:
- `SAVE_DIR` env var (per-game save path) instead of the fixed save location, plus a
  "resume from save on boot" switch.
- **Exit when the game ends** (and on a supervisor-sent shutdown) so the process can be
  reaped.
- Expose `joined / started / finished` on its `/api/health` (or a new `/api/status`) for the
  supervisor's `GET /api/games/<code>`.
- Optional, phase 2: **host "Start now"**. Today a game starts when every human seat is
  claimed. Add a host-only message to start early and fill unclaimed human seats with AI. The
  first player to join a code is the host (they created it).

### Client changes (`src/menu.rs`, `src/network_client.rs`)

- "Play Online" opens a **Create / Join** screen instead of joining immediately:
  - *Create:* two number pickers (players, humans) + name field → `POST /api/games` → show
    the code and the link with a **Copy link** button, then continue into the lobby.
  - *Join:* code field + name field. Landing on `?game=<code>` skips straight to this, with
    the code prefilled.
- After create/join, set `api_url = <origin>/g/<code>` and the ws URL to
  `wss://<origin>/g/<code>/ws`, then run the existing join flow unchanged.
- Show the code and link in the lobby panel too, so the host can re-send it.
- Store the reconnect token **per game code** in `localStorage`
  (`reconnect_token:<code>`). Today there's one token, and a second game would clobber it.

### Lobby panel has no background (the unreadable yellow text)

`rebuild_online_ui` (`src/network_client.rs`) spawns the right-side `OnlineUiRoot` panel
with no background colour, so the theme's yellow text sits directly on the map. Fix: give
the root node the theme's panel background (or a semi-opaque dark `BackgroundColor`, ~85%
alpha), the same as the in-game HUD panels. This is a one-line fix and should go in first,
independent of everything else.

### Deployment (`pingora-docker`)

- Replace the `civilization-server` service with a `civ-lobby` service (same image,
  different entrypoint). Drop `CIV_SEATS` / `CIV_NUM_PLAYERS`: settings now come from the
  create form. Keep the saves volume.
- `deploy/Caddyfile.internal`: route `/api/games*` and `/g/*` (including the WS upgrade) to
  `civ-lobby:8080`. Keep the static client as is. The old `/api/join` and `/ws` routes go.
- Only the supervisor's port is exposed. Child ports stay inside the container.

## Order of work

1. **Lobby panel background.** One-line fix. Ship it right away.
2. **`adv_civ_server` tweaks:** `SAVE_DIR`, exit on game end, status endpoint. Each one is
   small and testable on its own.
3. **`civ_lobby` supervisor:** create/status/proxy/reap, plus a registry. Test it locally by
   creating two games and joining each with `spike_client`.
4. **Client Create/Join screen** + `?game=` links + per-game reconnect tokens.
5. **Deploy:** compose + Caddyfile changes on pingora.
6. *(Phase 2)* host "Start now" / fill with AI, kick a seat, list "my games" (from the
   reconnect tokens stored locally).

## Alternatives considered

- **Several Bevy `App`s in one process (one thread per game).** It would work, since each
  `App` has its own `World`. But one panic takes down every game, ports and save paths still
  need managing per app, and a crashed game can't be restarted on its own. All the downsides
  of process-per-game and none of the isolation.
- **Docker container per game** (supervisor talks to the Docker socket). Stronger isolation,
  but it mounts the Docker socket into a web-facing service and adds seconds of start-up per
  game. Overkill here.
- **Per-entity game state in one World.** The "correct" long-term answer for density, but
  it's a refactor of every phase plugin. Only worth it if hosting cost ever matters.
