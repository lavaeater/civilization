# Online play: progress log

Branch `online-play`. Tracks the work planned in
[online-games-plan.md](online-games-plan.md): create/join online games with a code or
link, many games on one host at once.

## Status at a glance

| # | Step | State |
|---|---|---|
| 1 | Lobby panel background | ✅ Done (merged earlier, b80f996) |
| 2 | `adv_civ_server` tweaks: status endpoint, exit on game over, per-game save dir | ✅ Done |
| 3 | `civ_lobby` supervisor: create / status / proxy / reap | ✅ Done, tested locally |
| 4 | Client Create / Join screen, `?game=` links, per-game reconnect tokens | ⏳ Next |
| 5 | Deploy: Docker image target, Caddyfile routes, pingora compose | ⬜ |
| 6 | Phase 2: host "Start now" / fill with AI, kick a seat, "my games" | ⬜ |
| – | Resume running games after a lobby/container restart | ⬜ Open question |

## Step 2: game server tweaks (done)

- **`GET /api/status`** (`adv_civ_server/src/status.rs`, served by `http.rs`):
  `{total_players, human_seats, joined, connected, started, finished, round, phase}`.
  The game thread refreshes a shared snapshot each frame and the HTTP thread reads it.
- **`EXIT_AFTER_GAME_OVER_SECS`**: when set, the process exits that many seconds after
  `GameOver`, so the final scores stay visible for a while. Unset means it keeps running,
  as before.
- **Per-game save directory:** no code change needed. Saves go to `savegame.json` in the
  working directory, so the lobby starts each game in its own directory (`GAMES_DIR/<code>/`,
  which also holds that game's `server.log`).
- `spike_client` gained `--ws <url>` so it can join a game behind the lobby
  (`--token` already existed).

## Step 3: `civ_lobby` (done)

New workspace crate `civ_lobby/` (axum 0.8 + tokio + tokio-tungstenite + reqwest).

| Route | Does |
|---|---|
| `POST /api/games` `{total_players, human_seats}` | Validates 2–9 players and 1..=total humans, checks the game cap and per-IP rate limit, picks a free port pair, generates a code, spawns `adv_civ_server`, waits for its `/api/status`. Returns `{code, api_base, ws_url, join_url?, total_players, human_seats}`. |
| `GET /api/games/<code>` | Settings plus live status, or 404. |
| `ANY /g/<code>/api/*` | Proxied to the game's HTTP API (`/api/join` etc.). |
| `GET /g/<code>/ws` | WebSocket bridged to the game (binary frames copied both ways). |
| `GET /api/health` | Lobby liveness. |

Each child process gets:
- `SEATS` and `NUM_PLAYERS` from the create form.
- `PORT` / `HTTP_PORT` from a pool: game slot `i` gets `GAME_PORT_BASE + 2i` and `+1`.
- `NETCODE_KEY=random`, so a token for one game is useless in another.
- `PUBLIC_ADDR` (shared by all games) and `PUBLIC_WS=<PUBLIC_WS_BASE>/g/<code>/ws`.
- `EXIT_AFTER_GAME_OVER_SECS`, and `CLIENT_DIR=/nonexistent` (the lobby never serves the
  client through a game process).

The reaper runs every 30 s and removes a game when:
- its process exited (game over, or a crash);
- it never started within `UNSTARTED_TTL_HOURS` (default 24);
- it started but had no human connected for `IDLE_TTL_HOURS` (default 168).

On SIGTERM or Ctrl-C the lobby stops every game.

Configuration (env): `LOBBY_PORT` (8080), `GAME_BIN` (`adv_civ_server` next to the
lobby binary), `GAMES_DIR` (`games`), `GAME_PORT_BASE` (6000), `MAX_GAMES` (10),
`MAX_CREATES_PER_HOUR` (6 per client IP, using `X-Forwarded-For` when present),
`PUBLIC_ADDR`, `PUBLIC_WS_BASE` (e.g. `wss://civ.kidvhs.com`), `PUBLIC_URL` (for
`join_url`), `UNSTARTED_TTL_HOURS`, `IDLE_TTL_HOURS`, `EXIT_AFTER_GAME_OVER_SECS` (900).

**Verified locally (2026-11-10):**
- Started the lobby and created two games (3 and 4 players, 1 human each). An invalid
  create (12 players) got a 400, and an unknown code got a 404.
- Joined each game through `/g/<code>/api/join`, then connected `spike_client --auto` over
  the lobby's `/g/<code>/ws` bridge.
- Both games ran at the same time to rounds 8–9, with status showing `connected: 1`,
  `started: true` and the live phase.
- On SIGTERM the lobby stopped both game processes. Each game left its `savegame.json`
  and `server.log` in its own directory.

Unit tests: code generation and normalisation (no path tricks), settings validation.

### Known limits
- **Restarts lose games.** The registry is in memory. The game server can't resume a
  networked game from its save yet: seats and their reconnect tokens aren't in the save,
  and boot always goes through the lobby state. A lobby or container restart therefore
  ends every running game. Fixing it means saving seat bindings and adding a "resume from
  `savegame.json` on boot" path to `adv_civ_server`.
- **No host controls yet.** A game starts when every human seat is claimed (phase 2:
  "Start now").

## Step 4: client (next)

Plan: "Play Online" opens a Create / Join screen.
- **Create:** players, humans and name → `POST /api/games` → show the code and a copyable
  link → join.
- **Join:** code and name. A `?game=<code>` link prefills the code.
- After either, set `api_url = <origin>/g/<code>`; the join response already carries the
  right `ws_url`.
- Reconnect tokens are stored per game code.

## Log

- **2026-11-10:** Steps 2 and 3 implemented and verified locally (see above).
