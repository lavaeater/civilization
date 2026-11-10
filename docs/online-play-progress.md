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
| 4 | Client Create / Join screen, `?game=` links, per-game reconnect tokens | ✅ Done, tested natively |
| 5 | Deploy: Docker image target, Caddyfile routes, pingora compose | 🟡 Repo side done; live rollout is yours |
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

## Step 4: client (done)

New module `src/online_setup.rs` and a new `GameState::OnlineSetup`. "Play Online" now
opens a Create / Join screen instead of joining straight away:
- **Your name:** a text field (Bevy 0.19's `EditableText`), prefilled from `?name=` or
  `PLAYER_NAME`.
- **Create a new game:** − / + steppers for players (2–9) and how many of them are human
  (1..players), then "Create game" → `POST /api/games` → join.
- **Join a game:** a code field, then "Join game" → `GET /api/games/<code>` → join, or
  "No game with that code".
- **Joining:** either way the client sets `api_url = <lobby>/g/<code>` and runs the
  existing join flow unchanged. The join response already carries that game's `ws_url`.
- **Invite links:** opening `?game=<code>` (browser) or setting `GAME_CODE` (native) skips
  the main menu straight to this screen with the code filled in.
- **Lobby panel:** shows the game code, the invite link and a **Copy invite link** button
  (Bevy clipboard).
- **Reconnect tokens:** stored per game (`adv_civ_reconnect_token:<code>` in
  `localStorage`), so two games no longer overwrite each other's seat.
- **Old single-game servers:** if the server isn't a lobby (a bare `adv_civ_server`: its
  404 / HTML answer), both buttons fall back to joining its one game directly, the way
  "Play Online" used to.
- **Lobby address:** in the browser it's the page origin (or `?api=`); natively it's
  `LOBBY_URL`, default `http://127.0.0.1:8080`.
- **Dev hook** `AUTO_ONLINE_SETUP=create|join`: presses that button as soon as the screen
  is up, like the existing `AUTO_ONLINE`. It lets the flow run without a mouse.

Also fixed while testing:
- the "Leave game" and move buttons' text overflowed its buttons; smaller font, and the
  move buttons now fit the panel.

**Verified (2026-11-10)**, native client under X11 against a local lobby:
- The screen renders, with the code prefilled from `GAME_CODE`.
- Client 1 pressed Create (4 players, 2 human) and got `honest-tiger-62`. It joined as
  Egypt, and the panel showed the code, the link and the copy button.
- Client 2 joined by code as Crete. Status showed `joined: 2, connected: 2,
  started: true`, round 1 Movement.
- `cargo check --target wasm32-unknown-unknown` is clean. The browser build itself wasn't
  run.
- Unit tests: stepper bounds, how lobby answers are read, code normalisation.

Not done / follow-ups:
- ~~**Font gaps:** "—" and "✓" rendered as empty boxes.~~ Fixed: the full
  `FiraMono-Medium.ttf` now replaces Bevy's built-in default font (a small subset of the
  same typeface), and ✓ ◄ ► 🏆 🏁 became text ("[x]", "<", ">", "Winner:", "End").
  Icons or images can come later.
- **No "my games" list yet** (phase 2): the per-game tokens make it possible.

## Step 5: deploy (repo side done)

- **Dockerfile:**
  - The builder also builds `civ_lobby`.
  - The `server` image contains both binaries.
  - A new last (so default) stage `lobby` runs `/app/civ_lobby` with
    `GAMES_DIR=/app/saves/games`, `GAME_BIN=/app/adv_civ_server` and port 8080.
  - `--target server` still gives the one-fixed-game server.
- **`deploy/Caddyfile.internal`** (baked into the `web` image) and `deploy/Caddyfile`:
  `/api/*` and `/g/*` go to the lobby on port 8080. The old `/ws` route is gone.
- **Standalone `docker-compose.yml`:** the `game` service now builds the `lobby` target
  with `PUBLIC_ADDR`, `PUBLIC_WS_BASE`, `PUBLIC_URL` and `MAX_GAMES`.

**For the live pingora-docker stack** (another repo; not touched). In
`docker-compose.yml`, change the `civilization-server` service to:

```yaml
  civilization-server:
    build:
      context: ./repos/civilization
      dockerfile: Dockerfile
      target: lobby            # was: server
    container_name: civilization-server
    environment:
      - PUBLIC_ADDR=civ.kidvhs.com:443
      - PUBLIC_WS_BASE=wss://civ.kidvhs.com
      - PUBLIC_URL=https://civ.kidvhs.com
      - MAX_GAMES=10
      # SEATS / NUM_PLAYERS / NETCODE_KEY / PUBLIC_WS are no longer used:
      # each game gets its own from the create form and a random key.
    volumes:
      - <host saves dir, unchanged>:/app/saves
    restart: unless-stopped
    networks:
      - proxy-network
```

Then rebuild **both** `civilization-server` and `civilization-web`, because the new Caddy
routes are baked into the web image. Running games are lost on that restart (see Known
limits).

## Log

- **2026-11-10:** Steps 2 and 3 implemented and verified locally (see above).
- **2026-11-10:** Step 4 (client) done and verified natively with two clients; step 5's
  repo-side deploy files updated.
