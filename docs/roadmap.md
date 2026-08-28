# Current RoadMap

_Updated 26-08-28_

## Things that we want done 

The goal of this document is to collate what features exist in docs and roadmaps and which ones still need implementation. Go through this and cross-reference the files referenced and the code to figure out status on them and then, in this document, at the end, create a new todo-list of things to do in a suggested "good order" to do it in. 

**Status pass done 26-08-28** — cross-referenced every doc below against the code on `better-ai`. Several items each doc lists as "follow-up" are actually already implemented; corrections are noted inline. The synthesized, ordered todo-list is at the very end of this file.

### Agent API Design

See _agent-api-design.md_

Status: **mostly done.** Transport, read, apply (population/movement/ships/attacks/city
construction), multiplayer (agent-controlled players), and the dedicated `/trade`
endpoints (T1–T4: offer/accept/settle/stop) are all implemented and marked `[x]` in
the doc itself. Civ-card purchase is **also already applyable** — see correction below.
Open: richer `/state` (save-game snapshot), `/wait` long-poll/turn token (A4).

## Follow-ups (from agent-api-design.md — status corrected)

- ~~Implement Trade resolution (offer/accept/settle) end-to-end, then expose it.~~
  **Done.** `/trade/offer`, `/trade/accept`, `/trade/settle`, `/trade/stop` are all
  implemented (`src/agent_api/agent_api_systems.rs`), driving the real `OpenTradeOffer`
  model (not the dead `TradeMove` stub).
- ~~Civ-card purchase: compute cost + payment so `AcquireCard` is applyable.~~ **Done.**
  `CivCardDef::calculate_cost` (`assets_resources.rs:83`) + `ResolvedMove::AcquireCivCard`
  handling in `agent_api_systems.rs:460` compute cost from credits and apply payment.
  Remaining gap: `AcquireCards` (batch) isn't wired, only single `AcquireCard`.
- Auth/port config if this is ever exposed beyond localhost. **Still open** — server is
  `127.0.0.1` only, no auth. Not urgent while it's a local dev tool.

### AST Design

See _ast-design.md_

Status: **done.** All milestones M1–M4 are implemented and marked `[x]` in the doc.

#### 7. Known follow-ups (from ast-design.md — status corrected)

- ~~Full victory scoring (rule 35) and a real game-over state — only the FINISH
  *trigger* (rule 34.1A) is logged today.~~ **Done, and the doc/roadmap note is stale.**
  `succession_systems.rs` now has a `GameActivity::GameOver` state transition and a
  scoring/standings pass (`GameResult { standings }`) triggered on reaching FINISH.
  Worth a pass to double check the rule-35 scoring formula is complete/correct and that
  there's a player-facing victory screen, but the mechanism exists.
- Per-civ Late-Iron point values / age-length overrides mined from the "ASTCalc" sheet,
  dropped into `AstTrack::overrides`. **Still open** — `AstTrack` has the override hook
  but no per-civ data has been mined in. Low priority (cosmetic/accuracy, not blocking).
- Marker positioning currently assumes the standard 17-cell geometry; revisit if per-civ
  overrides change the cell count. **Still open**, blocked on the item above.

### Enhanced Input

See _enhanced_input.md_

Status: **HUD shortcuts done, phase-level keyboard support not started.** Steps 1–4
(action types, `HudContext`, lifecycle, F1–F4 toggle observers) are implemented and
marked `[x]`. Steps 5–6 are genuinely not started — no `TradeContext`/`MovementContext`/
etc. exist in code yet; every phase is still mouse-only.

- [ ] Step 5 — Phase contexts (incremental, one at a time)
- [ ] Step 6 — Universal Confirm / Cancel

### Multiplayer

See _multiplayer.md_ and _running-multiplayer.md_

#### Notes on Multiplayer

That document is pretty large, so I didn't scan it for what is done and not done, what I do know is that an Android / iOS app is super not prioritized. The game is very much a desktop / web game, i.e. being played on a computer. 

Second, my goal is that the game will have a running server using my pingora setup (~/projects/rust/dioxus/pingora-docker), so it will simply have an entry in that docker-compose file, and an entry in the services.json file so that pushing a tag to the repo rebuilds it. I have control of the machine that the server is running on so that is not a problem, all of the things running there I have control over - which also means that we could implement a server-side save-game strategy - either by saving to files or even a postgres database if needed. 

My ideal MVP scenario for online multiplayer is that we have one player that is admin and can access the game to create a multiplayer game. A multiplayer game gets a magic link, navigating to that link connects a player to the game, lets them select faction and player name. Reconnecting is as simple as going to that magic link again, I guess. Easy stuff. And then the human players can start playing.

#### Status correction

This is much further along than the note above assumed — the described MVP is
basically **already built** on `better-ai` (the `web-and-mobile` work merged in):
`adv_civ_server` + `adv_civ_protocol` exist as workspace members. Per `multiplayer.md`'s
own status table:

- ✅ Headless server, Docker image, move protocol for all phases, hidden info, AI fills
  empty seats, HTTP join API minting netcode `ConnectToken`s — **this is the magic-link
  flow**: `POST /api/join` with a name → token → WebSocket connect → reseated on
  reconnect by name match. Wasm client build, PWA manifest, click-the-map move
  selection, single-binary LAN mode, and a Docker+Caddy TLS deployment path all exist
  (`docs/running-multiplayer.md`).
- ⬜ Interactive trade over the network (server currently rejects it), ship-placement
  endpoint, session tokens (currently name-matched reseat, no auth), AI takeover after
  disconnect grace, mobile native (Android/iOS — confirmed not prioritized).
- Not yet done: the pingora `docker-compose`/`services.json` wiring described above —
  the app is deployable, it's just not plugged into your existing pingora host yet.

### Utility AI Design

See _utility-ai-design.md_

There are some things not yet implemented and / or tested in there, we should analyze them. My thoughts on the AI is to make the AI very "pointed" towards different ways of winning the game. For instance, conquest is a viable strategy, but so is trade... so perhaps one cannot simply reduce a strategy in this type of game to simple playstyles... but I wouldn't rule it out, right? 

What I have noticed is that the player in general is not keen enough on building cities. Regardless of playstyle, without cities, a player is doomed. The goal of every player is to advance on the AST, reach the end and then score the highest using the games scoring protocols. Getting there fast can perhaps debilitate other players to not score highly, I am not sure. But without cities, advancement on the AST is impossible - which concretely means that an AI must check their ability to advance every now and then. As a human player I am keenly aware of this at all times - but as a note on UI in general, when moving ahead on the AST, this could be noted.

#### Status correction

M1–M5 are all done and marked `[x]` in the doc: personality scaffolding, movement
scoring, expansion/city construction/elimination scoring, civ-card scoring, and trade
personality knobs are all implemented under `src/stupid_ai/` (`personality.rs`,
`scoring/{movement,city,expansion,trade,civ_cards}.rs`). **Only M6 — the tuning pass —
is open** (`⏳`, needs a live/headless self-play run to watch playstyles separate and
tune weights). This is exactly where the "AI doesn't build enough cities" complaint
lives: `city.rs` scores city construction, but nobody has run the self-play harness to
verify/tune the `city_income`/`defense` weights against real games yet.

| Playstyle    | Character | Knobs that dominate |
|--------------|-----------|---------------------|
| `Balanced`   | the reasonable default | everything ~0.5 |
| `Warlord`    | attacks, contests cities, low reserves | `aggression`, `risk` high; `defense` low |
| `Expansionist` | grabs land fast, thin everywhere | `expansion`, `growth` high; `defense` low |
| `Builder`    | cities + upkeep, hard to dislodge | `city_income`, `defense` high; `aggression` low |
| `Merchant`   | farms trade cards & civ tech | `trade_drive`, `tech_focus` high |
| `Turtle`     | minimal footprint, never overextends | `defense`, `calamity_aversion` high; `risk` low |

### New Notes on Gameplay and Transitions

The game is pretty near feature completion. I am not entirely sure all calamities work as they should (Barbarian Hordes is one, I think we must work more on), but this can be explored and tests can be written.

#### Status correction

Barbarian Hordes is **not a stub** — it has a full cascading-damage implementation
(`resolve_calamities_systems.rs`: `BarbarianHordesPhase`, `BarbarianHordesState`,
`barbarian_damage_score`, tie-break rule 30.525, `MAX_CASCADE_ITERATIONS` guard,
`advance_barbarian_hordes`). Whether it's *correct* per rules is unverified — no
targeted test found for it — so "write tests for it" is still the right next step, just
not "implement it from scratch."

On the phase-summary wishlist below: none of it exists yet. There's no round/phase
summary pane, no trade/conflict/card-draw log. Camera-follow exists but only for two
calamity-related selections (`focus_camera_on_calamity_selection`,
`focus_camera_on_unit_loss_selection` in `resolve_calamities_ui_systems.rs`) — it isn't
wired to movement, conflict, or city construction, which matches the "there is lag to
it" observation. The civ-card hover/click tooltip with dynamic discount-highlighting
(item 12) has no code at all yet.

What would be nice is a slicker transitioning of the game, in general. I assume we could simply iterate over things, but I have some thoughts already - after each phase in the game, we should be given a summary of what happened, because this is open information in the game.

#### Phases and Summaries

The current phase should be shown clearly in the pane for it.

1. Taxation - what every player paid in taxes, very simple, and if they used coinage to increase or decrease taxation (selected before taxation occurs) - and of course, if any city revolts occured. 
2. Population Expansion - our tokens aren't exactly perfectly positioned on the board. If there are multiple tokens, we should indicate this. We have some indicators surrounding areas, but some visual indicator that there are a **lot** of tokens in an area is helpful. I think there is an algorithm that shifts tokens slightly if they are more than one, but I think this information is thrown away in some circumstances - and also, tokens of different civilizations should not stack together, they should form independent piles.
3. Census is not necessary to make a big deal of, it updates the list in the game, done.
4. Ship construction - which player built a ship, where. Did they maintain a ship? Did they scrap a ship?
5. Movement - when the player is done, a replay could potentially be made showing every move on the board, otherwise... this one would become a lot of information without value quickly.
6. Conflict - great to have a list of conflicts that occured, number of tokens on each list and outcome (i.e. end number of tokens, since both players can co-exist after a battle).
7. Who and where are cities built?
8. No need to list surplus removal - but perhaps if someone loses a city it could be interesting information?
9. Who draws how many cards? 
10. After all trade is said and done, a summary of trades made could be done - only revealing open cards, of course.
11. Calamity Resolution, this one is great - who got hit by what where and did they have cards to mitigate it?
12. Summary of who acquired what cards, simple. - Also, a note here, hovering or click a civ card should show it's entire card text / description and credits - and hey - when selected, all other non-purchased cards should be highlighted with their new price **with the selected card** - making it easier to take discounts into account when purchasing.
13. Movement on AST - super important. If a player cannot move, he must be made aware of it. If some else moves, that should be highlighted as well.

Clearly, this can be done in so many different ways. In the Sid Meier Games, for instance, if the player gets attacked, we zoom in on that attack. This happens here as well, but there is lag to it. I could live with having the camera pan to areas of interest if something happens, that is a fine idea, but it has to **work**. If we do some kind of addition to the Game Info / Round Info pane, we could have some slight delay of events to slow the game down a bit so the player can absorb the info. So, say we are doing movement and the player is 3rd, then we list all the previous players moves in that pane... OR show them as they happen on the board... Suggestions welcome, as always we do it easy and clearly first, then improve later.

---

## Prioritized Todo List

Ordered so each item is cheap-and-standalone or unblocks/derisks the ones after it.
Several "follow-ups" from the source docs turned out to already be done (see the
corrections above) and are **not** repeated here.

1. ~~**Utility AI M6 — tuning pass.**~~ **Done (26-08-28).** Fixed the `adv_civ_server`
   release build first (it didn't compile — `ConfirmCivCardPurchase` was missing the
   `treasury_tokens` field added to the struct without updating the network path;
   networked purchases now pay 0 treasury tokens toward cost, matching the fact the
   wire protocol doesn't expose treasury payment yet). Then ran real headless
   9-player AI-only self-play (`SEATS=0 NUM_PLAYERS=9 BEVY_ASSET_ROOT=$(pwd)
   ./target/release/adv_civ_server`, `adv_civ_server/src/game.rs`'s existing
   `HeadlessGamePlugin`) with added instrumentation (playstyle-at-setup log, a
   city-built log line) to see what's actually happening.
   - **Confirmed root cause of the "AI doesn't build cities" complaint**: found a
     concrete case (a Warlord faction that built zero cities in 27 rounds while
     "frozen" at 0 cities). `score_city_construction`'s `city_income` weight is
     genuinely low for Warlord/Turtle, so `EndCityConstruction` could out-score
     building even when a decent site was available.
   - **Fix**: `score_city_construction` (`src/stupid_ai/scoring/city.rs`) now takes
     an `urgency: f32` (0 or 1) that overrides `city_income` when the player is
     short of the cities needed to clear their *next* A.S.T. epoch gate — computed
     in `select_stupid_city_building` (`src/stupid_ai/stupid_ai_systems.rs`) from
     `PlayerCities`/`AstPosition`/`AstEpoch`. No playstyle can sit at zero cities
     forever just because `city_income` is a low-priority knob for it; site quality
     (capacity/pressure) still decides *which* city under urgency.
   - **Tests**: 3 new unit tests in `city.rs` (`balanced_player_prefers_building_with_no_urgency`,
     `urgency_forces_warlord_to_build_even_though_city_income_is_low`,
     `urgency_does_not_override_site_quality`). Full suite still green (132/132).
   - **Verified in a second self-play run**: the specific Warlord-frozen-at-zero
     case from the first run is fixed (that faction reached 3 cities). Playstyles
     do separate — Expansionist/Balanced factions reliably reach 2–5 cities by
     round ~25.
   - **Residual issue found, not fixed here** (follow-up, not urgent enough to
     block this pass): in the second run, two other factions (a different Warlord,
     a Turtle) still ended with **zero** cities despite the urgency fix. Their
     `[CITY_CONSTRUCTION]` logs show no build ever happened — meaning they likely
     never had a legal `CityConstruction` move to score in the first place (no area
     with enough of their own population concentrated in one place), not a scoring
     problem. That points at `expansion.rs`/`movement.rs` population-concentration
     behaviour, not `city.rs`. Worth a follow-up self-play pass instrumented with
     "why no city-construction move was offered" logging.
2. **Test + verify calamities, starting with Barbarian Hordes.** The implementation
   exists (cascading damage, tie-breaks) but is untested against the rules text — write
   targeted tests in `tests/` before trusting it in longer self-play runs from item 1.
   Also derisks the "game is near feature-complete" assumption generally.
3. **Finish the Agent API's last gaps.** `AcquireCards` (batch purchase), a richer
   `GET /state` (save-game-shaped snapshot), and a `/wait` long-poll/turn token (A4 in
   `agent-api-design.md`). Small, well-scoped, and makes the API — which items 1 and 2
   both lean on for automated play — fully self-sufficient.
4. **Civ-card hover/click tooltip with dynamic discount pricing.** Roadmap item 12: show
   full card text on hover/click, and when a card is selected, re-price every other
   unpurchased card as if that one were bought first. Self-contained UI work in
   `civ_cards` using the existing `calculate_cost`/credits machinery — no new game logic,
   just surfacing what already exists. High value for human players, doesn't block or
   depend on anything else.
5. **Phase/round summary pane — start with the cheap phases.** Build the "Game Info /
   Round Info" summary pane infrastructure once, then fill in the easy, already-logged
   phases first: Taxation, Ship construction, City construction, Card draws, Trade
   settlement, Calamity resolution, Card acquisition, AST movement (items 1, 4, 7, 9,
   10, 11, 12, 13 in the notes above). Defer the two open-ended ones — Population
   Expansion token-stacking visuals and a full Movement replay — to a follow-up pass;
   they're genuinely harder ("becomes a lot of information without value quickly," per
   your own note) and benefit from seeing the simpler summaries land first.
6. **Extend camera-follow beyond calamities.** `focus_camera_on_selection` already
   exists and is wired for two calamity cases; reuse it for conflict, movement, and city
   construction so "the camera pans to where something happened" actually works
   game-wide, not just for calamities. Natural pairing with item 5's summaries.
7. **Population-expansion token stacking (item 2 in the notes).** Fix/verify the
   token-offset algorithm so multiple tokens — and separately, different civs' tokens —
   visibly form independent piles instead of overlapping or losing their offsets.
8. **Enhanced Input Steps 5–6 — phase input contexts.** Wire `Confirm`/`Cancel`/
   `Navigate*` into each phase (Trade, Calamity, Movement, City/Ship build, Civ-card),
   one at a time as the doc suggests. Pure UX polish, not blocking anything else — do it
   once the phase UIs it touches (civ cards, summaries) have settled from items 4–6.
9. **Multiplayer: close the remaining ⬜ items.** Interactive trade over the network
   (currently server-rejected), a ship-placement endpoint, real session tokens (replacing
   name-matched reseat), and AI takeover after a disconnect grace period. The MVP you
   described is otherwise already built — this is what's left before it's robust enough
   for real friends-and-family play.
10. **Wire the deployment into your pingora host.** Add `adv_civ_server` to
    `pingora-docker`'s `docker-compose.yml` and `services.json` so pushing a tag rebuilds
    it — the Docker/Caddy path already exists (`running-multiplayer.md`), this is just
    plugging it into infra you already control. Do this after item 9 so what you deploy
    is actually stable.
11. **AST per-civ Late-Iron overrides.** Mine the "ASTCalc" sheet for real per-civ point
    values/age lengths into `AstTrack::overrides`, then revisit marker-positioning
    geometry if the cell count changes. Cosmetic/accuracy, not blocking — low priority.
12. **Agent API auth/port config.** Only needed once/if the API is exposed beyond
    `127.0.0.1`. No action needed until item 9/10 make that a real scenario.
13. **Mobile native (Android/iOS).** Explicitly deprioritized in your own notes —
    included here only so it's not forgotten, not because it should be scheduled soon.