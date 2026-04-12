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

---

### Taxation (Phase 1)

**What is done:**
- `CollectTaxes` phase added to `GameActivity` enum and wired into the phase sequence (after `AcquireCivilizationCards`, before `PopulationExpansion`).
- Phase skips automatically if no cities exist on the board (first turn of the game).
- Each player transfers 2 tokens from stock to treasury per city (19.1).
- Shortfall detection: players who cannot pay in full have excess cities marked as revolting (19.31).
- Democracy holders pay what they can but never revolt (19.34).
- Revolt resolution: player with most unit points in stock (tokens=1, cities=5) takes over revolting cities; if no one can, city is eliminated (19.32–19.33).
- Unit tests cover full payment, shortfall revolt count, Democracy immunity, revolt beneficiary selection, and Coinage rate calculations.

**TODO:**
- [ ] Coinage: allow players holding Coinage to set rate to 1 or 3 tokens/city before taxes are collected (19.2) — currently hard-coded to 2
- [ ] Revolt visual: replace the revolted city's sprite with the beneficiary's city token and update `CityTokenStock` for both players

---

### 1. Rules - Civilization Cards and Effects

**What is done:**
- All 24 civilization cards defined in `assets/definitions/civilization.cards.ron` with costs, types, and credit tables.
- Card acquisition UI and purchasing mechanics implemented (commodity card + treasury payment).
- Prerequisites enforced (Engineering → RoadBuilding/Mining, Law → Democracy/Philosophy, Enlightenment → Monotheism/Theology).
- Credits from previously held cards applied to purchases.
- Calamity modifier effects implemented for: Engineering (Volcano/Earthquake, Flood), Pottery (Famine), Mysticism/Deism/Enlightenment (Superstition), Music/Drama and Poetry/Democracy (Civil War victim selection), Law/Democracy (Civil Disorder), Theology/Philosophy (Iconoclasm — see bug note below), Medicine (Epidemic primary), Military (Barbarian Hordes).

**TODO — card effects not yet implemented:**
- [ ] Agriculture: population limit +1 in areas solely occupied by this player's tokens; also +1 token substituted when cities are reduced (26.11)
- [ ] Road Building: allow tokens to pass through one land area into a second in the same movement phase (23.31); first area must not contain enemy units, Barbarians, or Pirate cities; cannot be used to then board a ship
- [ ] Metalworking: in conflicts, the holder removes tokens after all non-Metalworking players (24.24); Metalworking vs Metalworking is normal
- [ ] Architecture: holder may use treasury tokens to help build one city per turn; at least half of tokens must be on-board; cannot be used in areas with enemy units or Barbarians (25.3)
- [ ] Coinage: vary taxation rate to 1 or 3 tokens/city per turn (19.2, 32.42)
- [ ] Mining: increases value of one set of Iron/Bronze/Silver/Gems/Gold by one card when acquiring civ cards or evaluating hand for AST/victory — once per turn, may not exceed card maximum (28.53, 32.26)
- [ ] Monotheism: at end of calamity phase, convert one adjacent land area's units to own units; cannot target Monotheism/Theology holders, Barbarians, or Pirate cities (32.94)
- [ ] Theology: not affected by Monotheism (32.952)
- [ ] Cloth Making and Astronomy: defer until ships are implemented
- [ ] Credits may not be used in the same turn they are acquired (31.53) — verify this is enforced
- [ ] AI: teach the AI to select and benefit from civilization cards

**TODO — calamity modifier bugs/gaps:**
- [x] **Iconoclasm and Heresy** now correctly affects `cities_to_reduce` (default 4 cities), with all modifiers:
  - Theology: −3 cities (30.814) ✓
  - Philosophy: −1 city (30.813) ✓
  - Law: −1 city (30.812) ✓
  - Monotheism: +1 city (30.815) ✓
  - Road Building: +1 city (30.816) ✓
  - `advance_iconoclasm_heresy` refactored to use `ReduceCity` pattern
- [x] **Civil Disorder** now uses "all but 3 cities" default with all modifiers cumulative (30.715):
  - Music: −1 city (30.712) ✓
  - Drama and Poetry: −1 city (30.712) ✓
  - Law: −1 city (30.712) ✓
  - Democracy: −1 city (30.712) ✓ (was wrongly setting to 0)
  - Military: +1 city (30.713) ✓
  - Road Building: +1 city (30.714) ✓
- [x] **Epidemic** Road Building primary modifier added (+5 unit points, 30.614); Medicine now subtracts 8 (not halves):
  - Road Building: +5 unit points for primary victim ✓
  - Medicine for secondary victim: −5 unit points (30.613) — still TODO (needs secondary resolution)
- [x] **Slave Revolt** modifiers now correctly applied (30.423); base changed from 2 to token-based (15 tokens):
  - Mining: +5 tokens cannot support ✓
  - Enlightenment: −5 tokens cannot support ✓
  - Both Mining + Enlightenment: effects cancel ✓
  - `advance_slave_revolt` now queries `PlayerAreas.total_population()` and derives city count
- [ ] **Iconoclasm secondary victim** protections (30.819):
  - Philosophy holder: cannot lose more than 1 city as secondary — not implemented
  - Theology holder: cannot be named as secondary victim at all — not implemented
- [ ] **Epidemic secondary victim**: at least 1 token must remain in each affected area (30.612) — verify
- [ ] **Epidemic**: cities account for a maximum of 4 unit points (not 5) in Epidemic loss calculation (30.612) — verify
- [ ] **Famine**: Grain cards used for Pottery reduction must be placed face up and cannot be used to acquire civ cards that turn (30.312) — not implemented

**TODO — calamity resolution logic (TODOs in `resolve_calamities_systems.rs`):**
- [ ] **Civil War**: actual interactive unit selection for victim (chooses 15 + bonuses) — currently just logged
- [ ] **Civil War**: beneficiary interactive unit selection (20 points) — currently just logged
- [ ] **Civil War**: faction transfer — replace victim's units with beneficiary's tokens — not implemented
- [ ] **Civil War**: Philosophy override — first faction = 15 units chosen by beneficiary (30.4124) — not implemented
- [ ] **Civil War**: Military penalty — remove 5 unit points from each faction after selection (30.414) — not implemented
- [ ] **Treachery**: transfer city to the trading player (replace with their city token); currently just destroys the city (30.221)
- [ ] **Treachery** (not traded case): reduce own city, no other player benefits (30.222) — verify this path is correct
- [ ] **Piracy**: identify coastal cities (requires `Coastal` area marker or tag) — not implemented
- [ ] **Piracy**: replace two of primary victim's coastal cities with Pirate city tokens (30.911)
- [ ] **Piracy**: replace one coastal city each of two secondary victims (30.912)
- [ ] **Barbarian Hordes**: placement logic — choose start area causing greatest damage to primary victim (30.5211)
- [ ] **Barbarian Hordes**: continued movement — surplus Barbarians move to adjacent area causing greatest damage; repeat until no surplus (30.5231–30.5232)
- [ ] **Barbarian Hordes**: Crete may not be primary victim (30.527)
- [ ] **Flood**: primary victim loses max 17 unit points from flood plain (30.511)
- [ ] **Flood**: secondary victims — 10 unit points from same flood plain; primary victim allocates among others (30.512)
- [ ] **Flood**: if no units on any flood plain, eliminate one coastal city; if no coastal cities, no effect (30.514)
- [ ] **Flood**: white city sites are vulnerable; black city sites are safe (30.511)
- [ ] **Famine secondary**: interactive selection for primary victim to allocate 20 unit points (≤8 per player) — marked TODO in code
- [ ] **Iconoclasm secondary**: primary victim must order 2 cities from other players to be reduced (30.818) — marked TODO in code
- [ ] **Epidemic secondary**: primary victim must allocate 25 unit points (≤10 per player) — marked TODO in code

**TODO — Conflict consequences (missing from conflict phase):**
- [ ] When a player eliminates another's city by direct attack: draw one of victim's trade cards at random (24.51)
- [ ] Pillage: attacker may transfer up to 3 tokens from stock to treasury per city eliminated (24.52)
- [ ] Engineering: attacker needs only 6 tokens (city replaced by 5); defending Engineering city needs 8 to attack (replaced by 7); both Engineering = cancel (24.35) — verify this is in the conflict code

---

### 2. Rules - Ships

**What is done:**
- `SeaPassage` component added to area entities; `connect_areas` now wires sea connections alongside land connections.
- `OpenSea` marker component available for deep-water areas that require Astronomy.
- `Ship` component (owner entity), `ShipStock` (4 ships per player, initially in stock), and `PlayerShips` (area → ships on board) components implemented.
- `ShipConstruction` `GameActivity` variant added and wired into the phase sequence after `Census`, before `Movement`.
- `ShipsPlugin` registered; ship entities created for each player during `setup_players`.
- `enter_ship_construction` system handles maintenance (rule 22.3) and basic AI building (rules 22.1–22.4) in a single OnEnter pass.
- Simple 20×16 pixel ship sprite created (`assets/textures/ship.png`).

**TODO:**
- [ ] Military holders build ships after non-Military holders (22.11) — currently unordered
- [ ] Construction cost: allow levy (tokens from the area) in addition to treasury; currently only treasury or stock, not split between them per-area
- [ ] Ship movement during Movement phase: ships move up to 4 water areas along `SeaPassage` connections (23.52)
- [ ] Tokens embarking onto ships — only tokens not yet moved overland (23.51); up to 5 per ship
- [ ] Tokens must disembark before end of Movement phase (23.56); one-ship-per-token rule
- [ ] Open sea enforcement: ships may not cross to `OpenSea`-marked areas without Astronomy (23.52)
- [ ] Cloth Making: ship range +1 area (23.53)
- [ ] Astronomy: ships may enter open sea areas (23.54)
- [ ] Greece dual-coastline rule: ships enter/leave from same side (23.57)
- [ ] Human UI for ship construction (current implementation auto-builds for all players)
- [ ] Human UI for ship movement and embarkation/disembarkation

---

## Rule Analysis

Findings from reading all the rules documents and comparing with game code.

---

### 3. Rules - AST Progress

**What is done:**
- Nothing. No AST system exists.

**TODO:**
- [ ] Add `AstPosition` component per player (tracks current space)
- [ ] Add `AstMarker` entity per player on the board/UI
- [ ] Implement end-of-round AST advancement: each player advances one space (33.1)
- [ ] Epoch entry requirements (33.2):
  - Early Bronze: 2 cities in play
  - Late Bronze: 3 cities + at least 3 civ card groups (colors)
  - Early Iron: 4 cities + 9 civ cards from all 5 groups
  - Late Iron: 5 cities + civ card value ≥ printed space value
- [ ] Frozen marker: if city count < epoch requirement, marker does not advance (33.3)
- [ ] Backward movement: no cities in play → marker moves back 1 space/turn (except Stone Age, 33.4)
- [ ] Dual-color cards count as 2 groups for epoch entry (31.551)
- [ ] Display AST positions in UI

---

### 4. Rules - Winning the Game

**What is done:**
- Nothing. No win conditions or game-over state exist.

**TODO:**
- [ ] Implement victory point calculation (35.1):
  - Civilization card total face value
  - Commodity card set values (28.51) + face value of individual cards
  - Treasury: 1 point per token
  - AST position: 100 points per space
  - Cities on board: 50 points per city
- [ ] Trigger end-of-game check: at least one player reaches a finish square, OR predetermined time limit reached (34.1)
- [ ] All players must complete the final turn before determining winner (34.2)
- [ ] Tiebreaker: furthest along the A.S.T. wins
- [ ] Add `GameOver` state and victory screen UI

---

### 5. Improved AI

**What is done:**
- `src/stupid_ai/` implements a rule-based AI with random selection for all phases: pop expansion, movement, city construction, and trade.
- Trade AI uses basic heuristics (card overlap, top vs. worst commodities).

**TODO:**
- [ ] Board-state evaluation function: score based on AST position, city count, population, civ card holdings, treasury
- [ ] Personality archetypes (aggressive, economic, cultural) assigned at game start with weighted priorities
- [ ] Trade: per-opponent trust score updated from trade history (prisoner's dilemma)
- [ ] Trade: AI uses deceptive hidden-card strategies based on personality and trust level
- [ ] Trade: AI trades strategically toward its civ card purchasing goals
- [ ] AI civ card purchasing decisions (not yet implemented at all)
- [ ] AI taxation decisions (once taxation is implemented)
- [ ] AI ship construction and movement (once ships are implemented)

---

### 6. Expand Trade (more than 3 cards)

**What is done:**
- Structurally supports more than 3 cards per side (minimum 3 enforced, no hard maximum).
- The trade minimum (3 per side, 2 must be specified) is correctly implemented per rules (28.3).

**TODO:**
- [ ] Verify and test that trades with 4+ cards per side work end-to-end
- [ ] Add UI affordances for many-card trades (card count indicators, expand/collapse)
- [ ] Trade history display: show past trades between two players to inform trust decisions
- [ ] Consider a per-player "reputation" display visible to all, updated after each settled trade

---

### 7. Multiplayer Support

**What is done:**
- `IsHuman` component distinguishes human from AI players, used for UI routing.

**TODO:**
- [ ] Define multiplayer model: hotseat vs. networked (separate from item 8)
- [ ] Hotseat: route UI turns to the correct player; hide other players' trade card hands between turns
- [ ] Hotseat: enforce trade card secrecy — players should not see each other's hands during negotiation

---

### 8. Network Play

**What is done:**
- Nothing. No networking dependencies or infrastructure.

**TODO:**
- [ ] Choose a networking library (e.g., `lightyear` for Bevy, or `tokio`/`quinn` for custom client-server)
- [ ] Design authority model: server-authoritative with clients sending move intents
- [ ] Serialize game state and moves (reuse `save_game` serde/ron infrastructure)
- [ ] Implement lobby / game discovery
- [ ] Handle reconnection and desync detection

---

### 9. Multi-Platform Play

**What is done:**
- Desktop (Linux, Windows, macOS) fully supported.
- iOS and Android build infrastructure exists in `mobile/` (Xcode project, `cargo-apk`).
- Web (WASM) build via `trunk serve` is configured.

**TODO:**
- [ ] Resolve version mismatch: `mobile/Cargo.toml` pins Bevy `0.17.2` while main crate uses `0.18.0` — update mobile
- [ ] Test and fix touch input: pan, tap-to-select, tap-to-confirm
- [ ] Adapt UI for small screens: responsive sizing, larger tap targets
- [ ] Test and fix WASM-specific issues
- [ ] CI: add build checks for `wasm32-unknown-unknown`, iOS, and Android targets
