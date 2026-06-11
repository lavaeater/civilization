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
- [ ] Coinage: human player UI to choose rate (1 or 3 tokens/city) before taxes collected — AI sets rate via `ai_set_coinage_rate`; human always uses default 2 (19.2)
- [ ] Revolt visual: replace the revolted city's sprite with the beneficiary's city token and update `CityTokenStock` for both players

---

### 1. Rules - Civilization Cards and Effects

**What is done:**
- All 24 civilization cards defined in `assets/definitions/civilization.cards.ron` with costs, types, and credit tables.
- Card acquisition UI and purchasing mechanics implemented (commodity card + treasury payment).
- Prerequisites enforced (Engineering → RoadBuilding/Mining, Law → Democracy/Philosophy, Enlightenment → Monotheism/Theology).
- Credits from previously held cards applied to purchases.
- Calamity modifier effects implemented for: Engineering (Volcano/Earthquake, Flood), Pottery (Famine), Mysticism/Deism/Enlightenment (Superstition), Music/Drama and Poetry/Democracy (Civil War victim selection), Law/Democracy (Civil Disorder), Theology/Philosophy (Iconoclasm — see bug note below), Medicine (Epidemic primary), Military (Barbarian Hordes).

**Card effects implemented:**
- [x] Agriculture: +1 expansion in solely-occupied areas (`max_expansion_for_player_with_agriculture` in `population.rs`; `game_moves_systems.rs` uses it) — *+1 token substituted when cities are reduced still TODO (26.11b)*
- [x] Road Building: 2-hop land movement through empty friendly areas (`game_moves_systems.rs`)
- [x] Metalworking: non-MW players removed first in conflicts (`handle_with_metalworking` in `conflict_functions.rs`)
- [x] Architecture: city construction threshold −1 (5/11 vs 6/12); one saved token goes to treasury (`build_city` + `game_moves_systems.rs`)
- [x] Mining: best commodity stack +1 face value = +count² bonus to total buying power (`total_stack_value_with_mining`)
- [x] Monotheism: post-calamity elimination of up to 2 adjacent enemy tokens; auto-selected for AI; human UI not yet interactive (32.94)
- [x] Theology: immune to Monotheism conversions (32.952)
- [x] Cloth Making: ships get +1 hop (2-hop ferry moves generated in `game_moves_systems.rs`)
- [x] Astronomy: ships may enter `OpenSea`-marked areas (`game_moves_systems.rs`)
- [x] Engineering: +3 effective tokens in city conflicts (`conflict_triggers.rs`)
- [x] Coinage: `CoinageTaxRate` component; AI auto-sets rate; rate respected in `collect_taxes`

**TODO — card effects not yet implemented:**
- [ ] Agriculture: +1 token substituted when a city is reduced (26.11b) — the expansion bonus is done, this part is not
- [ ] Coinage: human player UI to choose rate (1 or 3) before taxes collected each turn (19.2) — currently AI-only
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
- [x] **Iconoclasm secondary victim** protections (30.819):
  - Theology holder: cannot be named as secondary victim ✓ (`advance_iconoclasm_heresy`)
  - Philosophy holder: cannot lose more than 1 city as secondary ✓ (`advance_iconoclasm_heresy`)
- [ ] **Epidemic secondary victim**: at least 1 token must remain in each affected area (30.612) — not enforced
- [ ] **Epidemic**: cities account for a maximum of 4 unit points (not 5) in Epidemic loss calculation (30.612) — not implemented
- [ ] **Famine**: Grain cards used for Pottery reduction must be placed face up and cannot be used to acquire civ cards that turn (30.312) — not implemented

**TODO — calamity resolution logic:**
- [ ] **Civil War**: human player interactive unit/city selection (victim selects 15+ pts, beneficiary selects 20+ pts) — currently auto-resolved for all players
- [ ] **Civil War**: Philosophy override — victim selects 15 units chosen by beneficiary instead (30.4124)
- [ ] **Civil War**: Military penalty — remove 5 unit points from each faction after selection (30.414)
- [x] **Civil War**: faction transfer — tokens re-owned via `Token::new(beneficiary)`, cities via `TransferCityTo` (`advance_civil_war` `TransferFaction` phase)
- [x] **Civil War**: Philosophy victim protection (−5 pts) and Military beneficiary bonus (+5 pts) ✓
- [x] **Treachery**: transfer city to trading player via `TransferCityTo`; non-traded case reduces own city (30.221–222) ✓
- [x] **Treachery**: human player UI — picks which city to hand over ✓ (`resolve_calamities_ui_systems.rs`)
- [x] **Piracy**: targets coastal cities first (`SeaPassage`-marked areas) (30.911) ✓
- [ ] **Piracy**: replace one coastal city each of two secondary victims (30.912) — not implemented
- [x] **Human calamity selection UI**: Superstition, Slave Revolt, Civil Disorder, Treachery, Iconoclasm & Heresy all pause for human input ✓
- [ ] **Barbarian Hordes**: placement logic — choose start area causing greatest damage (30.5211)
- [ ] **Barbarian Hordes**: continued movement — surplus Barbarians move to adjacent area with greatest damage; repeat until no surplus (30.5231–30.5232)
- [ ] **Barbarian Hordes**: Crete may not be primary victim (30.527)
- [ ] **Flood**: primary victim loses max 17 unit points from flood plain (30.511) — currently no unit-point cap
- [ ] **Flood**: secondary victims — 10 unit points from same flood plain, allocated by primary victim (30.512)
- [ ] **Flood**: if no units on any flood plain, eliminate one coastal city; if none, no effect (30.514)
- [ ] **Flood**: `CityFlood` component exists but is not consulted — white/black city site safety not enforced (30.511)
- [ ] **Famine secondary**: primary victim allocates 20 unit points (≤8 per player) — currently auto-distributed
- [ ] **Epidemic secondary**: primary victim allocates 25 unit points (≤10 per player) — currently auto-distributed

**TODO — Conflict consequences (missing from conflict phase):**
- [ ] When a player eliminates another's city by direct attack: draw one of victim's trade cards at random (24.51)
- [ ] Pillage: attacker may transfer up to 3 tokens from stock to treasury per city eliminated (24.52)
- [ ] Engineering exact city-conflict thresholds (24.35): attacker with Engineering needs only 6 tokens (vs 7); defending Engineering city requires 8 to attack (vs 7); both Engineering = cancel — current impl uses +3 effective tokens which approximates but may not be exact

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
- [x] Ship movement during Movement phase: `ShipFerryCommand` event, `execute_ship_ferry` system, `GameMove::ShipFerry` move generation, AI handling (23.52)
- [x] Tokens embarking onto ships — only tokens not yet moved overland via `TokenHasMoved` filter (23.51); up to 5 per ship
- [x] Open sea enforcement: ships may not enter `OpenSea`-marked areas without Astronomy (`game_moves_systems.rs`)
- [x] Cloth Making: ship range +1 area — 2-hop ferry moves generated (`game_moves_systems.rs`)
- [x] Astronomy: ships may enter open sea areas (`game_moves_systems.rs`)
- [ ] Tokens must disembark before end of Movement phase (23.56); one-ship-per-token rule
- [ ] Greece dual-coastline rule: ships enter/leave from same side (23.57)
- [ ] Human UI for ship construction (current implementation auto-builds for all players)
- [ ] Human UI for ship movement and embarkation/disembarkation

---

## Rule Analysis

Findings from reading all the rules documents and comparing with game code.

---

### 3. Rules - AST Progress

**What is done:**
- `AstPosition { space: u32 }` component on each player; starts at 1.
- `MoveSuccessionMarkers` `GameActivity` variant wired between `AcquireCivilizationCards` and `CollectTaxes`.
- `SuccessionPlugin` + `advance_succession_markers` system: advances marker +1 if epoch requirements met, retreats −1 if no cities (not below space 1), stays if frozen.
- Epoch boundaries: Stone Age 1–3, Early Bronze 4–6 (≥2 cities), Late Bronze 7–9 (≥3 cities + ≥3 card groups), Early Iron 10–12 (≥4 cities + all 5 groups + ≥9 cards), Late Iron 13+ (≥5 cities).
- Dual-color cards counted for multiple groups (bitflag union).
- Save/load: `ast_space` field in `SavedPlayer` (backward-compatible default = 1).
- 6 unit tests covering all advancement/retreat/freeze cases.

**TODO:**
- [ ] Per-faction starting positions (different factions begin at different spaces on the actual board)
- [ ] Late Iron space-specific card value thresholds (the card total value requirement varies per space)
- [ ] Add `AstMarker` entity per player on the board/UI — display current position visually

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
