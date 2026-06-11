# Utility AI — scoring the moves the game already enumerates

A design for replacing the current random `StupidAi` move-picker with a
**utility AI**: every phase scores each legal move with a set of weighted
*considerations*, and the AI picks the highest-scoring one. The weights live in
a per-player **Personality**, so the same scoring code produces visibly
different playstyles (aggressive, expansionist, builder, trader, turtle…).

We **roll our own** rather than depend on the `big-brain` crate: big-brain is a
Bevy version behind (0.17 vs our 0.18) and effectively unmaintained, and its
Thinker/Action/Scorer machinery is built for *continuous* "what should this
entity do right now" AI — agents picking among always-available behaviours. Our
problem is different and easier: the game already hands us a **discrete,
enumerated, legal-move list** (`AvailableMoves`) each phase. We don't need
big-brain's scheduling/arbitration layer; we need a good **scoring function over
a known list**. That's a few hundred lines of plain Rust. We keep big-brain's
*vocabulary* (considerations, response curves, pickers) because it's the right
mental model.

See also: `docs/reinforcement-learning.md` (these scorers are the "R0 strong
heuristic baseline" that doc calls for, and personalities give us the diverse
opponent pool R2 self-play needs), `docs/agent-api-design.md` (the move pipeline
this hooks into).

## 1. Where this plugs in

The decision pipeline already exists and we reuse it wholesale:

1. The game computes `AvailableMoves { moves: HashMap<usize, GameMove> }` for a
   player (`src/civilization/game_moves/`).
2. An observer (`on_add_available_moves`) queues the player; `drain_ai_move_queue`
   fires a `SelectStupidMove { player }` after `ai_move_delay_secs`.
3. A per-phase system (`select_stupid_*` in `stupid_ai_systems.rs`) reads
   `AvailableMoves` and emits the concrete command message.

Today step 3 does `available_moves.values().choose(&mut rng)` — **uniform
random**. The *only* thing utility AI changes is step 3: instead of `choose`, we
**score every move and pick by personality**. Nothing about move enumeration,
queuing, or command emission changes. This keeps the change surgical and lets us
convert one phase at a time, A/B-testing against the random baseline.

## 2. Core concepts

```
Consideration  — a named, normalised [0,1] reading of the game world for one move.
                 e.g. "does this move land on a city square?", "how exposed am I after?"
Evaluator      — a response curve mapping a raw value → [0,1] (linear, power,
                 sigmoid, step). Lets us say "exposure hurts, but mildly until it's bad".
Score          — weighted sum (or product) of considerations for one move.
Personality    — the weight vector + curve choices that turn considerations into a Score.
Picker         — turns the scored list into a choice: argmax (greedy) or
                 softmax-sample (adds variety / avoids robotic determinism).
```

A move's score is, by default, a **weighted sum**:

```
score(move) = Σ_i  personality.weight[i] * evaluator_i(consideration_i(move))
```

We deliberately start with weighted-sum (easy to reason about and tune). Some
considerations act as **vetoes/multipliers** (e.g. "this move loses a city for
nothing" → multiply by ~0) — those are applied as a final clamp/multiply so a
single fatal factor can kill an otherwise attractive move, the way big-brain's
`Product` measure does.

## 3. The Personality

A component carried by every AI player, alongside `StupidAi`:

```rust
#[derive(Component, Clone, Reflect)]
pub struct Personality {
    pub playstyle: Playstyle,        // for display/logging
    pub picker: Picker,              // Greedy | Softmax { temperature }
    pub weights: Weights,            // the tunable knobs below
}

#[derive(Clone, Reflect)]
pub struct Weights {
    // expansion / economy
    pub growth: f32,            // value of feeding population & expanding
    pub city_income: f32,       // value of building/holding cities (tax + cards)
    // map control / aggression
    pub aggression: f32,        // value of attacking enemies / weakening neighbours
    pub defense: f32,           // penalty for leaving own areas/cities exposed
    pub expansion: f32,         // value of grabbing empty/contested territory
    // economy of cards
    pub trade_drive: f32,       // eagerness to propose/accept trades
    pub calamity_aversion: f32, // weight on dumping calamity-bearing cards / hoarding
    pub tech_focus: f32,        // value of civ-card credits & AST progress
    pub risk: f32,              // 0 = cautious (keep reserves), 1 = all-in
}
```

`Playstyle` is an enum of **named presets** that fill in `Weights`. Presets are
the product surface — a few of them give the whole table distinct characters:

| Playstyle    | Character | Knobs that dominate |
|--------------|-----------|---------------------|
| `Balanced`   | the reasonable default | everything ~0.5 |
| `Warlord`    | attacks, contests cities, low reserves | `aggression`, `risk` high; `defense` low |
| `Expansionist` | grabs land fast, thin everywhere | `expansion`, `growth` high; `defense` low |
| `Builder`    | cities + upkeep, hard to dislodge | `city_income`, `defense` high; `aggression` low |
| `Merchant`   | farms trade cards & civ tech | `trade_drive`, `tech_focus` high |
| `Turtle`     | minimal footprint, never overextends | `defense`, `calamity_aversion` high; `risk` low |

Assignment: in `setup_players` each AI player gets a personality — round-robin
over the presets (so an 8-player table shows all the archetypes), with an option
in `DebugOptions` to force one for testing. Humans/agent-controlled players get
none.

## 4. Per-phase scoring

Each phase has its own `score_<phase>(move, world, personality) -> f32`. The
considerations below are the **interesting drivers**; movement, trade, and
civ-card acquisition are the rich ones, as the brief calls out.

### Population Expansion  *(today auto-resolved for most players)*
Mostly a value question of *where* to put growth when it's manual. Considerations
per candidate area:
- **`supports_city`** — area `max_population` ≥ 6 (can host/feed a city): high.
- **`already_mine`** — reinforcing an area I hold (denser, safer) vs spreading.
- **`crowding`** — heading toward surplus removal next phase is wasteful → penalty
  via `growth`/`risk`.
- **`contested`** — area also bordered by enemies: `expansion`/`aggression` like
  it, `defense` dislikes it.

### Ship building / Ship ferry
- **`opens_reach`** — does the ferry/ship reach areas I can't reach by land
  (especially city-capable ones)? `expansion` weights this.
- **`overextension`** — moving across water thins the source; `defense` penalty
  scaled by how exposed the source becomes.

### Movement — *the flagship*
For each `Movement`/`AttackArea`/`AttackCity`/`EndMovement`/`ShipFerry`:
- **`target_supports_city`** — target area `max_population` and emptiness: moving
  toward city-capable, unowned land scores high (`expansion`, `city_income`).
- **`target_is_city_square` already mine** — staying put on a city square I hold
  is *good defense* — so `EndMovement` / not-moving-out-of-a-city scores via
  `defense`. Conversely abandoning a city square is heavily penalised.
- **`source_left_exposed`** — after the move, how many enemy-adjacent tokens
  remain at the source vs enemies who could hit it? Leaving 1 token next to a
  strong neighbour is bad (`defense`).
- **`weakens_neighbour` / attack EV** — for attacks: my tokens vs defender's,
  expected outcome (`conflict` is attrition: equal numbers trade off). `aggression`
  loves a favourable attack; `risk` governs taking even/unfavourable ones. A clearly
  losing attack gets a near-zero **veto multiplier**.
- **`takes_city`** — `AttackCity` that I can actually win is a top-tier move for
  almost everyone (cities are the win engine), scaled hard by `aggression`/`city_income`.
- **`stay_for_construction`** — if staying enables a city build this turn
  (enough pop on a city square), don't wander off: boosts `EndMovement`.

The combination "value of moving = access to city-capable land **and** not
exposing what I hold" is exactly the brief's intuition, expressed as
`expansion`+`city_income` pulling one way and `defense` the other, with
`aggression`/`risk` deciding contests.

### City Construction
- **`can_afford_upkeep`** — will I still feed this city after building (city needs
  pop support)? If not → strong penalty (`defense`/`city_income`): a city you
  can't support is a liability.
- **`board_value`** — building on a high-`max_population`, defensible area scores
  higher.
- **`tempo`** — `Builder` wants every legal city now; `Warlord` may prefer to keep
  tokens mobile (lower `city_income`, higher `aggression`).
- `EndCityConstruction` scored as the baseline "build nothing" alternative.

### City Elimination (forced support check)
When over-supported, choose *which* city to give up: keep the most defensible /
highest-income one, drop the exposed or low-value one. Considerations:
`exposure`, `board_value`, plus `risk` (cautious players shed aggressively to
stay safe).

### Trade — *rich, imperfect information*
**Implementation note (discovered while wiring M5):** AI trading does **not** run
through the `SelectStupidMove` → `select_stupid_trade_move` pipeline. That stub
operates on the dead `TradeMove`/`TradeOffer` enum path and never actuates a card
exchange. The *live* AI trade path is a set of countdown-driven systems in
`concepts/trade/trade_systems.rs` — `ai_create_trade_offers`,
`ai_accept_trade_offers`, `ai_settle_trades`, `ai_stop_trading_when_ready` —
operating on the working `OpenTradeOffer` model (the same one the UI and agent API
use). So utility scoring for trade is injected as **personality knobs into those
systems** (see `stupid_ai/scoring/trade.rs`), not as a `score_move` over the move
list. The conceptual drivers below are unchanged:
- **Propose**: score candidate offers by *net commodity value gained* (move up the
  card-value ladder toward completing sets) weighted by `trade_drive`; never give
  away a top commodity (`is_top_commodity`) unless `trade_drive` is very high.
- **Accept/decline**: accept if the offer improves my set value by a
  personality-scaled threshold; `Merchant` accepts thinner margins.
- **Calamity handling**: offloading a calamity-bearing card is worth extra to a
  high `calamity_aversion` player (dump risk onto others) — but the rules hide
  whether a card carries a calamity from the receiver, so this is a *bluff*
  surface we score but flag for later refinement.
- Trade is genuinely hidden-information; we keep the first version **simple and
  honest** (value-of-set heuristic) and note it as the most promising place to
  later layer learning (per the RL doc).

### Civilization Card Acquisition — *the win engine*
- **`credit_synergy`** — cards give credits in colours that discount future cards;
  score a purchase by how much it cheapens the player's *intended* tech line, not
  just its face benefit (`tech_focus`).
- **`ast_progress`** — cards/credits that advance AST position are weighted by
  `tech_focus`; reaching FINISH is the win, so this is near-terminal value.
- **`affordability` / reserve** — spend down to a reserve governed by `risk`;
  cautious players keep trade cards for next round, `risk`-takers empty the bank.
- **`calamity_immunity`** — some civ cards mitigate calamities; `calamity_aversion`
  weights those up.
- `DoneAcquiringCards` is the "buy nothing more" baseline.

## 5. Module layout

New, self-contained, inside the existing `stupid_ai` module so nothing else moves:

```
src/stupid_ai/
  personality.rs   — Personality, Playstyle, Weights, presets, Picker, Evaluator curves
  scoring/
    mod.rs         — shared helpers: Consideration, weighted_sum, pick(scored, picker)
    movement.rs    — score_movement(...)
    city.rs        — score_city_construction(...), score_city_elimination(...)
    expansion.rs   — score_population_expansion(...), score_ship(...)
    trade.rs       — score_trade(...)
    civ_cards.rs   — score_civ_card(...)
```

The `select_stupid_*` systems shrink to: gather candidate moves → call
`score_*` for each (with the player's `Personality`) → `pick()` → emit the same
command they emit today. The scoring functions are **pure** given their query
inputs, which makes them unit-testable without a running app (feed in synthetic
component data, assert the ranking).

## 6. Picker & determinism

- **Greedy** (argmax) is the default for strength.
- **Softmax** (sample ∝ `exp(score/temperature)`) adds non-robotic variety and is
  what we use when we want a personality to feel "loose"; temperature is a
  personality knob. It also doubles as **exploration** if we later log
  `(state, move)` for imitation learning.
- Ties broken randomly (keep an `rng`), so identical scores don't always pick the
  lowest entity id.

## 7. Implementation plan (commit after each milestone)

- **M1 — Personality scaffolding.** ✅ `personality.rs` with `Personality`,
  `Playstyle`, `Weights`, presets, `Picker`. Assigned round-robin in
  `setup_players` (`DebugOptions.force_playstyle` override). Type registered.
- **M2 — Movement scoring.** ✅ `scoring/mod.rs` + `scoring/movement.rs`;
  `select_stupid_movement` now scores the move list. Unit tests on the ranking
  (Warlord takes a winnable city, Turtle vetoes a lost cause, Expansionist grabs
  empty city land).
- **M3 — Expansion & city construction/elimination.** ✅ `scoring/expansion.rs`
  + `scoring/city.rs`; the three `select_stupid_*` systems converted. (Ship
  ferries are scored inside movement; there is no separate ship-build *move*.)
- **M4 — Civ-card acquisition.** ✅ `scoring/civ_cards.rs`;
  `select_stupid_civ_card_move` scores each option vs wealth + existing credits.
- **M5 — Trade.** ✅ `scoring/trade.rs`; personality knobs injected into the live
  `ai_*` `OpenTradeOffer` systems (see the implementation note in §4).
- **M6 — Tuning pass & self-play sanity.** ⏳ Run `AGENT_FACTIONS`/headless
  self-play, watch that distinct playstyles emerge and games terminate; tune
  weights/curves. (Needs a live run — can't launch the GUI from the tool harness.)

## 8. Testing & tuning

- **Unit**: pure `score_*` over synthetic worlds — assert e.g. a winning
  `AttackCity` outranks a wandering move for a `Warlord`, and a `Builder` keeps its
  city square. Lives in `tests/` with the existing app-builder helpers where a
  world is needed; pure where not.
- **Behavioural**: a self-play harness logging per-player AST progress and a few
  counters (attacks made, cities held, cards bought) per playstyle — the
  archetypes should separate (Warlord: most attacks; Builder: most cities;
  Merchant: most cards). This is also the baseline the RL doc measures learned
  agents against.

## 9. Non-goals (for now)

- No learning/weight optimisation — weights are hand-tuned. (Later: the RL doc's
  R1/R2 can *learn* these weights or replace the linear score with a net, keeping
  this exact "score the legal-move list" interface.)
- No lookahead/search — scoring is one-ply. Cities/attacks get crude
  expected-value terms but no game-tree search (that's the RL doc's optional R3).
- Trade bluffing stays shallow; flagged as the prime candidate for later depth.
