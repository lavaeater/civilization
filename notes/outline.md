# Outline

## Intro

This document contains specs, ideas, links and todo-lists pertaining to this project, the game of Advanced Civilization.

## The Next Steps

What is left to do. Below are the identified areas:

1. Rules - Civilization Cards and Effects 
2. Rules - Ships - construction, maintenance, boarding, travel over water and open water. 
3. Rules - AST Progress at end of rounds 
4. Rules - Winning the game 
5. Improved AI - a scoring-based AI with weighted priorities to simulate strategies and personalities. Also consider memories and the prisoner's dilemma, i.e., for trading (the only area where this applies), make sure different players employ different strategies for deception when it comes to extra cards. 
6. Expand trade so they can contain more than three cards - putting emphasis on trust between players. 
7. Multiplayer support 
8. Network play 
9. Multi-platform play

## Rules

All rules are found in the ./rules folder.

### 1. Rules - Civilization Cards and Effects

**What is done:**
- All 24 civilization cards are defined in `assets/definitions/civilization.cards.ron` with card types, costs, and credit systems.
- Card acquisition UI and purchasing mechanics are implemented (commodity card payment).
- Calamity modifier effects are fully implemented in `src/civilization/concepts/resolve_calamities/modifiers.rs`:
  - Engineering (Volcano/Earthquake, Flood), Pottery (Famine), Mysticism/Deism/Enlightenment (Superstition), Music/Drama and Poetry/Democracy (Civil War), Law/Democracy (Civil Disorder), Theology/Philosophy (Iconoclasm and Heresy), Medicine (Epidemic), Military (Barbarian Hordes).

**TODO:**
- [ ] Agriculture: increase population limit by 1 in areas occupied solely by that player
- [ ] Road Building: allow tokens to move through one land area into a second in the same phase
- [ ] Metalworking: in conflicts, player always removes tokens after all others
- [ ] Mining: increase trade card value of Iron/Bronze/Silver/Gems/Gold by one when acquiring civ cards
- [ ] Architecture: allow using tokens from the treasury to assist building one city per turn
- [ ] Coinage: implement taxation rate variation (1–3 tokens per city)
- [ ] Monotheism: convert occupants of an adjacent area to the player's units
- [ ] Literacy: verify no missing effect beyond credits (currently treated as credits-only)
- [ ] Astronomy: prerequisite for ships crossing open seas — defer until ships are implemented
- [ ] Cloth Making: ships may move an extra area (five instead of four) — defer until ships are implemented
- [ ] AI: teach the AI to purchase and benefit from civ cards

---

### 2. Rules - Ships

**What is done:**
- Map data already includes `sea_connections` on areas (`NeedsConnections`), so the topology is present.
- `ShipConstruction` is stubbed out as a commented-out `GameActivity` variant in `src/lib.rs`.

**TODO:**
- [ ] Add `Ship` component and ship token stock per player
- [ ] Implement ship construction phase (`ShipConstruction` game activity): rules for when/where ships can be built and their cost
- [ ] Implement ship maintenance: ships not in use revert / sink if not supported
- [ ] Implement naval movement: ships move up to 4 sea areas (5 with Cloth Making); use `sea_connections`
- [ ] Implement boarding: tokens embark onto ships in their area; ships carry tokens across sea areas
- [ ] Implement open sea crossing (requires Astronomy civ card)
- [ ] Implement combat/piracy interaction with ships
- [ ] Add ship sprites and UI

---

## Rule Analysis

Findings from reading all the rules documents and comparing with game code.

---

### 3. Rules - AST Progress

**What is done:**
- Nothing. No AST (Advancement of Science and Technology / Age Scale Track) system exists.

**TODO:**
- [ ] Define `AstPosition` component per player (track their current position on the AST)
- [ ] Implement end-of-round AST advancement: each player advances one step unless blocked
- [ ] Implement advancement prerequisites (city count / population / civ card thresholds per AST step)
- [ ] Gate the `AcquireCivilizationCards` phase so only players at the correct AST level can purchase certain cards
- [ ] Display AST positions in the UI

---

### 4. Rules - Winning the Game

**What is done:**
- Nothing. No win condition or game-over state exists.

**TODO:**
- [ ] Define victory condition: first player to reach the final AST step wins (depends on AST implementation above)
- [ ] Add a `GameOver` state or event
- [ ] Implement a victory screen / end-game UI
- [ ] Handle tie-breaking rules if multiple players reach the final step in the same round

---

### 5. Improved AI

**What is done:**
- `src/stupid_ai/` contains a functioning but simple AI.
- It picks moves by basic heuristics and random selection for all phases: population expansion, movement, city construction, and trade.
- Trade AI makes rudimentary decisions based on card overlap with other players, distinguishing top vs. worst commodities.

**TODO:**
- [ ] Design a board-state evaluation function (score based on: AST position, city count, population, civ card holdings, treasury size)
- [ ] Weight priorities by personality archetype (aggressive, economic, cultural, etc.) assigned at game start
- [ ] Trade: give each AI player a trust score per opponent, updated based on trade history (prisoner's dilemma)
- [ ] Trade: allow AI to deliberately conceal or misrepresent hidden cards based on personality/trust
- [ ] Trade: AI should seek trades that advance its civ-card purchasing strategy
- [ ] AI should make civ card purchase decisions (currently not implemented)
- [ ] AI should make ship construction and movement decisions once ships are implemented

---

### 6. Expand Trade (more than 3 cards)

**What is done:**
- Trade offers already support more than 3 cards per side structurally — there is a minimum of 3 (2 guaranteed + at least 1 hidden) but no enforced maximum.
- The negotiation and acceptance flow exists.

**TODO:**
- [ ] Verify and test that trades with 4+ cards per side work end-to-end (UI, validation, resolution)
- [ ] Add UI affordances that make offering/receiving many-card trades legible (card count indicators, expand/collapse panels)
- [ ] Implement trust signaling: show trade history between two players so they can make informed decisions about large trades
- [ ] Consider a "reputation" display per player visible to all, updated after each settled trade

---

### 7. Multiplayer Support

**What is done:**
- Nothing specific to multiplayer. The `IsHuman` component distinguishes human from AI players but only drives UI decisions.

**TODO:**
- [ ] Define the multiplayer model: hotseat (multiple humans, same machine) vs. networked (separate machines)
- [ ] For hotseat: route human UI turns to the correct player based on whose turn it is; hide other players' hands
- [ ] For networked: see item 8 below

---

### 8. Network Play

**What is done:**
- Nothing. No networking dependencies or infrastructure exist.

**TODO:**
- [ ] Choose a networking library (e.g. `lightyear` for Bevy, or a custom client/server via `tokio`/`quinn`)
- [ ] Design authority model: server-authoritative with clients sending move intents
- [ ] Serialize game state and moves (much of this may reuse the existing `save_game` serde/ron setup)
- [ ] Implement lobby / game discovery
- [ ] Handle reconnection and desync detection

---

### 9. Multi-Platform Play

**What is done:**
- Desktop (Linux, Windows, macOS) fully supported via standard Bevy.
- iOS and Android build infrastructure exists in `mobile/` (Xcode project for iOS, `cargo-apk` for Android).

**TODO:**
- [ ] Resolve version mismatch: `mobile/Cargo.toml` pins Bevy `0.17.2` while the main crate uses `0.18.0` — update mobile to `0.18.0`
- [ ] Test and fix touch input for mobile: pan, tap-to-select, tap-to-confirm flows
- [ ] Adapt UI layout for small screens (responsive sizing, larger tap targets)
- [ ] Web (WASM) build: `trunk serve` is already configured — test and fix any WASM-specific issues
- [ ] CI: add build checks for `wasm32-unknown-unknown`, iOS, and Android targets
