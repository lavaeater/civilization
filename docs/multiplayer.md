# Multiplayer

## Status (June 2026)

Phases 1–2 done, phases 3–4 largely done — see the `web-and-mobile` branch history:

- ✅ Workspace: `adv_civ_protocol` (wire types) + `adv_civ_server` (headless bin + `spike_client` CLI smoke bot)
- ✅ Full real game headless in Docker (`CivLogicPlugins` split; 133 MB image; compose + Caddy in `deploy/`)
- ✅ Move protocol for all phases (interactive trade propose/accept still server-rejected — own milestone)
- ✅ Hidden info: per-seat `YourHand`; public civ cards / hand sizes in `GameStateView`
- ✅ AI fills unclaimed seats; `SEATS=0` = AI self-play; networked seats are `IsHuman + AgentControlled`
- ✅ HTTP join API minting real netcode `ConnectToken`s (`POST /api/join`); `NETCODE_KEY` env
- ✅ In-game network client: `GameState::Online`, menu button, side-panel UI (lobby/phase/moves/hand)
- ✅ Compiles to wasm; `trunk build --release` produces the web bundle (~33 MB wasm)
- ✅ Join-link flow end to end: client fetches its ConnectToken from `/api/join` (ureq native,
  gloo fetch wasm); browser reads `?name=&api=&ws=`; server's `PUBLIC_WS` advertises the socket
  URL (client no longer assumes same-origin `/ws`); Caddy serves the app + `/api/*` + `/ws`
  (websocket upgrade verified through TLS)
- ✅ Single-binary path: `adv_civ_server` serves the wasm client (`CLIENT_DIR`, default `dist`)
  on its HTTP port alongside `/api/*`, so one command hosts client + API + WebSocket — good for
  LAN play with no Caddy. See [`docs/running-multiplayer.md`](running-multiplayer.md).
- ✅ Map view: real board image + per-area labels driven by `GameStateView`
- ✅ Reconnection: session-token-hardened reseat (see below) + full state sync
  (phase/board/hand/moves) on (re)join
- ✅ PWA manifest — installable on mobile (phase 0)
- ✅ Click-the-map move selection: click a highlighted area to act; movement is
  source→target two-click with green/yellow highlight dots (side-panel buttons remain
  a fallback). Decision logic is a pure `resolve_map_click`, unit-tested.
- ✅ Interactive trade over the network (see below) — real `OpenTradeOffer`
  model, not the dead `NetTradeMove`/`TradeMove` pair
- ⬜ Ship placement endpoint — still needs `ShipConstructionState` turned from
  a singleton into a per-player resource first, see docs/roadmap.md
- ⬜ AI takeover after disconnect grace — explicitly deferred: a disconnected
  seat should just wait for its player to reconnect (aided by the token below),
  not get auto-piloted. If we ever want an AI to step in, that should be a
  conscious host decision, not an automatic timeout.
- ⬜ Mobile native (Android via existing mobile crate, then iOS)

### Session tokens (26-08-29)

Reseating used to match purely on the display name a client typed, so anyone
who knew (or guessed) a seated player's name could steal their seat by
joining with the same name. Fixed with a per-seat reconnect secret:

- `POST /api/join` accepts an optional `"token"` field and **always** returns
  a `"reconnect_token"` in its response — freshly minted on a client's first
  join, or simply the token it already sent back to it unchanged.
- The web client saves this in `localStorage` (`adv_civ_reconnect_token`) and
  sends it on every future join. The native client (dev/manual testing, no
  browser storage) reads/logs it via a `RECONNECT_TOKEN` env var instead —
  not the target UX, just parity for testing.
- Server-side, a seat that has never had a token bound accepts a plain name
  match (first-ever join to that identity, or an old client) — this keeps the
  "share a link, type your name" flow working. Once a token *is* bound,
  reclaiming that identity requires it; a name-only or wrong-token join falls
  through to a genuinely free seat instead of stealing the named one. See
  `find_seat_for_join` in `adv_civ_server/src/game.rs` (5 unit tests).
- This is **session-token hardening for a friends-and-family game**, not
  real security — the token travels in plaintext over the join HTTP request
  and lives in `localStorage`, which is fine against a curious namesake but
  not a determined attacker. Good enough for "invite friends," not for a
  public server with strangers.

### Trade over the network (26-08-29)

Three new dedicated messages, not `SubmitMove` picks — the guaranteed-card
selections are free-form input, not a choice from an enumerated move list,
same reasoning as the agent API's dedicated `/trade/*` endpoints, which this
mirrors closely (right down to reusing the same `OpenTradeOffer` model and
methods — `is_valid`, `accept`, `settle_creator`/`settle_acceptor`):

- `ProposeTradeOffer { offering_guaranteed, offering_hidden_count,
  wanting_guaranteed, wanting_hidden_count, target }` → spawns a validated
  `OpenTradeOffer`, silently dropped server-side if it fails `is_valid()`
  (rule 28.3: exactly 2 guaranteed cards, ≥3 total, per side).
- `AcceptTradeOffer { offer }`.
- `SettleTradeOffer { offer, cards }` — either party's actual cards; the
  hidden slots may be anything, honest bluff included.
- Server broadcasts `TradeOffersView` (every open offer, hidden slots as
  counts only — never identities) to everyone whenever any offer changes,
  and pushes it to a (re)joining client alongside the existing phase/board/
  hand sync. Mirrors `GET /trade`'s shape in the agent API.

**Client UI is deliberately simplified**, not a placeholder: both guaranteed
slots on a side are the same card (2 of it) rather than two different named
cards, picked by cycling through the 18 commodity types with a button rather
than a proper picker widget; settlement always uses your entire real hand for
both guaranteed and hidden slots rather than letting you choose exactly which
cards to commit. This is a real, usable trade flow — not everything the rules
allow, but everything needed to actually trade. See docs/roadmap.md's design
sketch for where a richer UI (and the negotiation/counter-offer model) could
go from here.

**Not interactively verified**: no working websocket test client exists in
this environment (`spike_client` has a pre-existing, unrelated
`lightyear::link::Link::new` build break) to drive an actual multi-client
trade round trip. Verified instead by: full compile across native, wasm, and
the headless server; the full test suite (357 lib + 132 integration tests);
and a live headless self-play run confirming the new systems run every frame
with zero connected clients without panicking or erroring. The server-side
logic reuses the exact `OpenTradeOffer` methods the agent API and local human
UI already exercise, so its correctness rests on tests that already exist
for those, not new ones. Please run an actual two-client trade before relying
on this.

Original exploration follows.

---

Exploration: server-hosted multiplayer using [lightyear](https://github.com/cBournhonesque/lightyear),
with the server running in a Docker container and human players joining — in priority order —
from the **web**, then Android, then iOS. Games support 0..=max human players; empty seats are
filled by our AI (`stupid_ai` / utility AI), so a 0-human game is just the server playing itself.

## TL;DR of the recommendation

- **lightyear 0.26.x** (matches our Bevy 0.18) in client/server mode, netcode.io auth.
- **Headless server**: the existing game logic running under `MinimalPlugins` in a Docker
  container — the AI and the integration tests prove the rules engine already runs without UI.
- **Transport: WebSocket (WSS) first**, WebTransport later. We're turn-based; TCP head-of-line
  blocking is irrelevant to us, and WSS sits behind a bog-standard reverse proxy with
  Let's Encrypt. WebTransport requires the game server itself to terminate QUIC/TLS on a UDP
  port, which complicates the Docker story for zero gameplay benefit at our pace.
- **Protocol = moves, not component replication.** The server sends each client a view of the
  game state plus their `AvailableMoves`; the client sends back a chosen move. This is exactly
  the interface `stupid_ai` and the agent API already consume.
- **Invite links**: server has a small HTTP API. Creating a game returns
  `https://host/join/<game-id>`; opening it serves the wasm client, the friend types a name,
  the HTTP API hands them a netcode `ConnectToken`, lightyear connects them into the lobby.
  No accounts, no passwords, for now.
- **One game per server process** in the first pass (see "Multiple games" below — our global
  `GameState`/`GameActivity` states make multi-game-per-world a real refactor).

## Why lightyear, and which parts of it we actually use

Lightyear ([repo](https://github.com/cBournhonesque/lightyear),
[book](https://cbournhonesque.github.io/lightyear/book/)) is the de-facto Bevy networking
library: client/server architecture, transport-agnostic (UDP, WebTransport, WebSocket, Steam),
wasm-compatible, with releases tracking Bevy versions (0.26.x ↔ Bevy 0.18).

Its headline features — client-side prediction, rollback, snapshot interpolation, lag
compensation — exist for fast-paced action games. **We need almost none of that.** Advanced
Civilization is turn-based; 200 ms of latency on "build city in Thrace" is imperceptible.

What we *do* use:

| lightyear feature | Why we want it |
|---|---|
| `ClientPlugins` / `ServerPlugins` | Connection lifecycle, handshake, timeouts, disconnect detection inside Bevy's ECS |
| netcode.io auth (`ConnectToken`) | Secure-enough join flow without rolling our own handshake |
| WebSocket + WebTransport transports, wasm support | The whole point: browser clients |
| Reliable ordered channels (Messages) | Move submission and state broadcast |
| Replication + entity mapping | *Optional* — see "State sync" below; we may use it for spectator-friendly state, but messages suffice |
| Host-server / local mode | Single-player keeps working without a network: client and "server" in one app |

The alternative considered: `bevy_replicon` (+ `bevy_replicon_renet`). Comparable maturity,
arguably simpler replication model, but lightyear has first-class WebTransport/WebSocket wasm
transports and a netcode auth story out of the box, which is exactly our bottleneck. Sticking
with lightyear as instructed.

## Architecture

```
                         Docker container
                ┌──────────────────────────────────┐
 Browser ──WSS──┤ reverse proxy (Caddy)            │
 Browser ──WSS──┤   ├── /            → wasm client │
                │   ├── /api/*       → HTTP API    │
 (later)        │   └── /ws          → lightyear   │
 Android ─WSS───┤                       WebSocket  │
                │ adv_civ_server (headless Bevy)   │
                │   ├── CivilizationPlugin (rules) │
                │   ├── stupid_ai (empty seats)    │
                │   ├── lightyear ServerPlugins    │
                │   └── HTTP API (lobby + tokens)  │
                └──────────────────────────────────┘
```

### Crate layout

Move to a small workspace (we already half have one with `lava_ui_builder` and `mobile`):

```
adv_civ/            existing lib: game rules, components, concepts, game_moves, AI
adv_civ_protocol/   shared: network messages, channel definitions, serializable state types
adv_civ_server/     bin: MinimalPlugins + CivilizationPlugin + lightyear server + HTTP API
(root bin)          existing client, grows lightyear ClientPlugins behind a feature/menu choice
```

`adv_civ_protocol` is the contract both sides compile against — lightyear requires identical
message/channel registration on client and server. Keep it free of Bevy render/UI deps so the
server build stays slim.

### Headless server feasibility

This is the part the codebase is already good at. The integration tests in `tests/` run the
full rules engine with app-builder helpers and no window; `stupid_ai` plays entire games
unattended. The server binary is essentially "the test harness, forever":

- `MinimalPlugins` + asset-free variants of the setup (the map/area definitions load via
  `bevy_common_assets`/RON — fine headless; sprites/audio plugins simply aren't added).
- Audit `CivilizationPlugin` for accidental UI/render dependencies and split anything found
  into a `CivUiPlugin`. The phase logic, triggers, and `game_moves` should already be clean —
  the tests wouldn't pass otherwise.
- `GamePaused` and save_game work as-is server-side; **server-side saves give us free
  crash-recovery and "resume game later" for multiplayer**.

## State sync: messages, not replication (mostly)

Two options exist; the codebase strongly favors the first.

### Option A — move-based protocol (recommended)

The game already has the perfect seam: each phase computes `AvailableMoves`
(`src/civilization/game_moves/game_moves_components.rs`) — a `HashMap<usize, GameMove>` —
for the active player, and both the AI and the agent API play by *picking an index*. The
network protocol is the same conversation:

```
Server → Client:  GameStateView   (serialized view of public state + your private hand)
Server → Client:  YourMoves       (the AvailableMoves map, with stable ids — see below)
Client → Server:  SubmitMove      (game_id-scoped: chosen move index / trade parameters)
Server → Client:  MoveRejected | StateDelta | PhaseChanged ...
```

Consequences:

- **Server-authoritative by construction.** Clients can't cheat: they can only choose from
  moves the server offered. Validation already exists — it's the same path the AI uses.
- **No game logic on the client at all.** The web client becomes a renderer + move picker.
  Smaller wasm binary, no risk of client/server rules divergence.
- **Entity ids must be translated.** `GameMove` variants carry `Entity` (`MovementMove.source`,
  `BuildCityMove.target`, …). `Entity` is world-local and meaningless across the wire. The
  protocol crate needs stable identifiers — area names (areas already have names for the map),
  player index, card names (`CivCardName`, `TradeCard` already exist as data types). A
  `NetworkGameMove` mirror of `GameMove` with `AreaId`/`PlayerId`/card names, plus a mapping
  layer server-side. This is the single biggest piece of protocol work.
- **Hidden information is trivial**: trade cards in hand are only ever sent to their owner;
  calamity cards stay secret until resolved. With component replication we'd need per-client
  interest management rules for this; with messages it's just "compose the view per client."
- **Trade phase is the hard case.** Trading is interactive and concurrent (offers,
  counteroffers between any players, ~2600 lines in `trade_systems.rs`). It still fits the
  message model — offers/accepts are already events — but expect the protocol for the trade
  phase to be as large as all other phases combined.

### Option B — lightyear component replication

Mark game-state components (`Population`, cities, token holdings, …) with `Replicate` and let
lightyear sync them; client UI reads components exactly like the single-player UI does today.
Attractive because the existing UI systems keep working almost unchanged, and lightyear's
entity mapping solves the `Entity` translation problem for us. Costs: per-client visibility
rules for hidden info (lightyear "Rooms"/interest management), replicating our many bespoke
components requires them all to be (de)serializable and registered, and the client still needs
the move-picking messages anyway.

**Pragmatic hybrid**: messages for moves and private info (Option A core), and optionally
replicate the *public board state* components so the existing map UI keeps reading components
it already understands. Decide after a spike; start the spike with pure Option A on a single
phase (PopulationExpansion) since it's the simplest.

## Transport: WebSocket first, WebTransport later

Both are supported by lightyear on native **and** wasm. As of March 2026 WebTransport is
Baseline (Safari 26.4 finally shipped it), so browser support is no longer the deciding factor.
Operations are:

| | WebSocket (WSS) | WebTransport |
|---|---|---|
| Browser support | Universal, ancient | Baseline since 2026-03 (Safari 26.4+) — older installed Safaris excluded |
| TLS | Terminated at reverse proxy; Caddy auto-provisions Let's Encrypt | Game server terminates QUIC/TLS itself; needs cert material mounted into the container, UDP/443 exposed |
| Docker/networking | One TCP port behind any proxy/LB; works on hotel/corp networks | UDP often blocked on restrictive networks; can't sit behind a plain HTTP proxy |
| Gameplay benefit | Head-of-line blocking — irrelevant for turn-based | Unordered/unreliable streams — we don't need them |

For a turn-based board game, WSS wins on every axis we care about. Lightyear is
transport-agnostic, so adding WebTransport later (for the native mobile clients, or just to
flex) is a config change plus cert plumbing, not a redesign. Note: lightyear's wasm examples
use self-signed certs with `serverCertificateHashes` (14-day validity) — that's a dev-loop
trick, not something to ship; production WebTransport wants a real cert either way.

## Auth and the invite-link flow

Lightyear's netcode layer requires clients to present a `ConnectToken`, minted with a
`private_key` + `protocol_id` known to the server. Lightyear deliberately does *not* prescribe
how tokens reach clients — "use a secure backend" — which slots perfectly into the invite-link
design. We already run an embedded HTTP server for the agent API (`tiny_http`,
`src/agent_api/`), so an HTTP sidecar in the game server is established practice here
(consider graduating to `axum` for the public-facing one).

Flow — no accounts, friction ≈ zero:

```
1. Host: POST /api/games {name: "Tommie", human_seats: 3, ai_seats: 2}
     → { game_id: "brisk-otter-42", join_url: "https://civ.example/join/brisk-otter-42" }
   (Or via a "Create online game" button in the web client, which calls the same API.)

2. Host sends join_url to friends over whatever (Signal, Discord, carrier pigeon).

3. Friend opens join_url → Caddy serves the wasm client with the game id in the path.
   Client shows one input: "Your name".

4. Client: POST /api/games/brisk-otter-42/join {name: "Greger"}
     → { connect_token: <base64 netcode token>, session_token: <uuid> }
   The ConnectToken is short-lived (~30 s) and single-use; netcode handles replay protection.
   The session_token is OURS, stored in localStorage — see reconnection below.

5. Client hands the ConnectToken to lightyear → WSS connect → server maps the netcode
   client_id to the seat reserved in step 4 → player appears in the lobby.

6. Lobby (a pre-`GameState::Playing` screen): shows joined players, lets the host fill empty
   seats with AI and pick factions, host presses Start → server drives `PrepareGame` onward.
```

Security posture, stated honestly: a join link is a bearer capability. Anyone holding the URL
can take a seat. For "invite some friends" that is the *feature*, not a bug. Mitigations that
cost nothing: unguessable game ids, seat cap, link expiry, host kick. Real accounts (OAuth,
email) are explicitly out of scope until we care about strangers, rankings, or persistence
across devices.

**Reconnection matters more than authentication** for us: browser tabs close, phones sleep,
and a civ game lasts hours. The `session_token` from step 4 lets a returning client call
`/join` again and be re-attached to the *same seat* with a fresh ConnectToken. Server-side,
a disconnected human seat pauses input for that player (or after a grace period, offers the
host an "AI takeover" — `stupid_ai` taking the seat is code we already have).

## Docker

Multi-stage build, nothing exotic:

```dockerfile
# build stage: rust:1.8x — cargo build --release -p adv_civ_server
#              + trunk build --release (wasm client) → dist/
# runtime stage: debian:bookworm-slim (or distroless)
#   /app/adv_civ_server          headless server binary
#   /app/dist/                   wasm client static files
#   /app/assets/                 map/card RON definitions
```

- Caddy can run as a second container (compose) or the server's HTTP API can serve `dist/`
  itself and Caddy stays a pure TLS front. Compose with two services is the cleanest.
- Ports: 443/tcp only (Caddy). Internally: game server HTTP+WS on e.g. 8080. No UDP until
  WebTransport day, which adds 443/udp straight to the game server.
- State: mount a volume for server-side saves (`save_game` already serializes the full game
  to RON) → container restarts resume games.
- The release profile (`opt-level = "s"`) is tuned for wasm size; the server wants the `dist`
  profile (opt-level 3) instead.

### Multiple games per server — the honest caveat

`GameState` and `GameActivity` are **app-global Bevy states**, and the whole phase machine
hangs off them. One Bevy `World` therefore hosts exactly one game. Options:

1. **Process-per-game (first pass).** A tiny lobby/spawner service (or just the host's POST
   handler) launches one `adv_civ_server` process per game on an internal port; Caddy routes
   `/<game-id>/ws` to it. Dumb, robust, crash-isolated, trivially resumable from saves. A civ
   game's server load is near-zero between moves, so even a small VPS hosts many processes.
2. **Refactor states to per-game entities** (`GameActivity` as a component on a "game" entity,
   systems keyed by game). Correct long-term answer for density, big invasive refactor — every
   `OnEnter(GameActivity::X)` schedule and state-scoped system changes. Not first-pass.

Recommendation: ship 1, design the protocol so nothing assumes single-game (every message
carries/implies a game id), revisit 2 only if hosting cost ever matters.

## Web client

- Existing `trunk serve` pipeline already builds the game for wasm; the multiplayer client is
  the same build with lightyear `ClientPlugins` (websocket + netcode features, wasm-compatible)
  and a network-backed implementation of the move-selection UI.
- The client keeps `GameState`/`GameActivity` *locally* for UI flow, driven by server
  `PhaseChanged` messages instead of local phase systems — the phase plugins' *systems* don't
  run on the client in network mode, only the UI layers.
- Single-player must not regress: lightyear's host-server (local) mode runs client and server
  in one app with no socket, so "local game" and "online game" share one code path. This is
  the strongest argument for doing the Option A refactor even before shipping multiplayer.

## Mobile

- **Phase 0 (free)**: the web client *is* the mobile client. WSS works in every mobile
  browser; the join link opens straight into the game. Ship a PWA manifest so it installs to
  the home screen. Most of the "mobile work" is responsive UI, which the web version needs
  anyway.
- **Phase 1, Android**: existing `cargo apk` / `mobile` crate builds the same Bevy app
  natively; lightyear's native WebSocket (or later WebTransport/UDP) transport, same protocol
  crate, same join flow (deep-link `https://civ.example/join/...` into the app via Android App
  Links).
- **Phase 2, iOS**: same story via the existing `mobile.xcodeproj`; Universal Links for the
  invite URL. Apple review and signing make this last, as specified.

## Phased plan

1. **Spike (1 phase, 2 players, localhost):** workspace split, `adv_civ_protocol` with
   `NetworkGameMove` for PopulationExpansion only, headless server bin, native client over
   localhost WebSocket. Proves the move-protocol seam and the Entity↔stable-id mapping.
2. **Full protocol:** all phases incl. trade; per-client state views; hidden information.
   AI fills unclaimed seats (already works server-side by construction).
3. **Web + invite flow:** wasm client over WSS, HTTP API (create/join/token), lobby screen,
   session tokens + reconnection, host-start.
4. **Docker:** compose (Caddy + server), volume for saves, process-per-game spawner.
5. **Hardening:** disconnect grace / AI takeover, link expiry, server-side save/resume, load
   test with N AI games.
6. **Mobile:** PWA polish → Android native → iOS.

## Open questions

- How much of the current UI reads game components directly vs. could consume a
  `GameStateView` message? Determines how tempting the Option B hybrid is.
- Trade phase protocol design — concurrent offers between humans need timeouts/UX that the
  hot-seat version never needed.
- Spectators? Cheap with Option A (send them the public view, no moves) — worth keeping in
  mind while designing messages, not building yet.
- Does `bevy_kira_audio`/asset loading need feature-gating to keep the server image free of
  audio/render crates? (Likely yes: a `client` cargo feature on `adv_civ`.)
- lightyear 0.26.x API churn: the crate refactored into subcrates recently; pin exact versions
  and budget for upgrade friction at each Bevy bump.

## References

- lightyear repo: <https://github.com/cBournhonesque/lightyear> (0.26.x ↔ Bevy 0.18)
- lightyear book — connection/auth: <https://cbournhonesque.github.io/lightyear/book/concepts/connection/title.html>
- lightyear examples (incl. `lobby`, `simple_setup`, wasm/WebTransport cert notes):
  <https://github.com/cBournhonesque/lightyear/tree/main/examples>
- netcode.io spec (the auth model behind ConnectToken):
  <https://github.com/mas-bandwidth/netcode/blob/main/STANDARD.md>
- WebTransport baseline status: <https://caniuse.com/webtransport>
