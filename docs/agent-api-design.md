# Agent API — Design & Plan

Lets an external agent (e.g. Claude via `curl`) play the game by querying state
and submitting moves over a tiny local HTTP server embedded in the running game.

## Why HTTP (not WebSockets)

- The game is **turn-based** — request/response fits naturally; no need for a
  persistent server-push channel.
- An agent driving via tools acts in **discrete calls** (curl/file reads), not a
  held socket + event loop.
- `tiny_http` is **synchronous**, so a Bevy system can poll it non-blocking each
  frame — no tokio/async dragged into the frame loop.

## How it plugs in

The game already has the whole decision pipeline:

- Each player gets an `AvailableMoves { moves: HashMap<usize, GameMove> }` component
  when it is their turn in a phase (`game_moves/`).
- The `StupidAi` reacts to `On<Add, AvailableMoves>`, queues a choice, and a system
  translates the chosen `GameMove` into a **command message**
  (`ExpandPopulationManuallyCommand`, `MoveTokenFromAreaToAreaCommand`, …).
- The (commented-out) `console/make_a_move_command.rs` shows the generic
  "apply move at index" translation — the Agent API revives that, fed by HTTP.

The external agent **drives the existing `IsHuman` player** — we are a remote
human, reusing the proven "the game waits for this player" path. The AI trigger
ignores non-`StupidAi` players, so it won't fight us.

## Endpoints (MVP — Population Expansion)

- `GET /state` — current `GameActivity`, whether it's the human's turn, and the
  player's areas with population context.
- `GET /moves` — the human player's `AvailableMoves` as `{index, kind, …}`.
- `POST /move` `{"index": k, "number": n?}` — translate `AvailableMoves[k]` to its
  command message and apply it.

Server: `127.0.0.1:7878`. If the port can't bind, the API logs a warning and the
game runs normally without it.

## Milestones

- [x] **A1 — Transport + read.** Embed `tiny_http`, poll from a Bevy system
  (always-on), implement `GET /state` and `GET /moves` for the human player.
  Verified live: `GET /state` at the menu returns `{"human_player":false,"phase":"NotPlaying"}`.
- [x] **A2 — Apply (Population Expansion).** `POST /move` writing
  `ExpandPopulationManuallyCommand`; covered by an in-crate test
  (snapshot → `/moves` → apply emits the command). Live end-to-end play pending a
  game session.
- [~] **A3 — Generalize.** Movement / Ship ferry / Attacks / City construction /
  EndMovement / EndCityConstruction / civ-card *Done* are applyable. **Trade and
  buying civ cards are deferred** (see below).
- [x] **A5 — Multiplayer.** The API drives *all* agent-controlled players, not just
  one. A player is agent-controlled when it is `IsHuman` without `StupidAi`; the
  `AGENT_FACTIONS` env var marks which factions at game start.
- [~] **A4 — Ergonomics.** Thin client script `scripts/agent_autoplay.py` drives a
  full self-play game (conservative reference strategy). `GET /wait?faction=&timeout_ms=`
  is now implemented: it holds the HTTP request open (no busy-polling) until it's
  that player's turn or `timeout_ms` elapses (default 30s, capped at 60s), returning
  the same shape as a player's `/state` entry plus `timed_out`. Still open: a richer
  `/state` (save_game-shaped snapshot).
  - Note: `AcquireCards` (a *batch* civ-card purchase) exists as a `GameMove`
    variant and is handled on the multiplayer network path (`adv_civ_server/src/net.rs`),
    but move-generation never produces it — `AvailableMoves` only ever offers
    single-card `AcquireCard` moves, recalculated after each purchase. An agent
    already buys several cards per turn by calling `POST /move` with `AcquireCard`
    repeatedly before `DoneAcquiringCards`; wiring `AcquireCards` into the agent API
    would be dead code with today's move generation, so it's left alone.

## Running a full self-play game

    AGENT_FACTIONS=all cargo run        # launch with every player agent-controlled
    python3 scripts/agent_autoplay.py   # drive them all; watch the board + A.S.T.

The script builds cities, satisfies city support, ends movement, skips civ-card
buying, and opts out of trading — enough to march a game forward. Swap in the
`/trade/offer`+`/accept`+`/settle` calls for real trading.

## Multiplayer

- `AGENT_FACTIONS=all` → every non-human player is agent-controlled (full agent
  self-play). `AGENT_FACTIONS=Egypt,Babylon` → just those factions. Unset = none
  (only the configured human, if any, is agent-drivable).
- Endpoints are faction-aware:
  - `GET /players` — all controlled players and whose turn it is.
  - `GET /state` — phase + a `players` array (each with areas + `your_turn`).
  - `GET /moves?faction=Egypt` — that player's moves. Omit `faction` and the API
    picks the single player who currently has moves (handy in sequential phases).
  - `POST /move {faction?, index, number?}` — apply for that player (or the active
    one if `faction` is omitted).

## Trade (in progress) — two trade systems

The codebase has **two parallel trade models**:
- `TradeMove` / `TradeOffer` / `AvailableMoves` — move-generation is wired
  (`recalculate_trade_moves_for_player`) but **execution is a dead stub** (the
  `button_action` arms are empty). Do **not** use this path.
- `OpenTradeOffer` + the `ai_*` systems (`ai_create_trade_offers`,
  `ai_accept_trade_offers`, `ai_settle_trades`, `ai_stop_trading_when_ready`) +
  the human UI handlers (`handle_publish_offer`, `handle_accept_offer_button`,
  `handle_confirm_settlement`) — the **actual working trade**: offers carry exactly
  2 guaranteed cards + a hidden count each way; accept; then both settle by
  choosing actual cards (the hidden ones can bluff).

So agent trade is a **dedicated `/trade` interface over `OpenTradeOffer`**, mirroring
those handlers — not the generic move translator. Agent-controlled players are
tagged `AgentControlled` (plus `IsHuman`) so AI trade systems skip them
(`Without<IsHuman>`) and we drive them via the API.

Trade increments:
- [x] **T1** — `GET /trade?faction=` (offers + my hand + can_accept) and
  `POST /trade/stop {faction?}` (drop `CanTrade`, so a full-agent game can clear the
  phase).
- [x] **T2** — `POST /trade/accept {faction?,id}` → `OpenTradeOffer::accept`.
- [x] **T3** — `POST /trade/offer {faction?,offering_guaranteed,offering_hidden,
  wanting_guaranteed,wanting_hidden,target?}` → spawns a validated `OpenTradeOffer`.
- [x] **T4** — `POST /trade/settle {faction?,id,cards}` → `settle_creator` /
  `settle_acceptor`; the existing `finalize_settled_open_offers` does the exchange.

Card names are the `TradeCard` display names (e.g. `Ochre`, `Iron`). Offer ids are
the stringified entity bits returned by `GET /trade`. A full trade is: creator
`POST /trade/offer` → acceptor `GET /trade` (see `can_accept`) → `POST /trade/accept`
→ both `POST /trade/settle` with their actual cards → cards exchanged automatically.

Buying civ cards is still deferred (needs cost/payment computation); only
`DoneAcquiringCards` is wired so that phase doesn't stall.

## Follow-ups

- Implement Trade resolution (offer/accept/settle) end-to-end, then expose it.
- Civ-card purchase: compute cost + payment so `AcquireCard` is applyable.
- Auth/port config if this is ever exposed beyond localhost.
