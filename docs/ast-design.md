# A.S.T. (Archaeological Succession Table) — Design & Plan

Tracking the players' progress through the ages. Rules reference:
`docs/rules/14.movement-of-markers-on-the-ast.md` (rule 33). Source data mined
from `docs/ast.xslx`, sheet **"AST"**.

## 1. What the A.S.T. is

A linear track of **17 spaces, indexed 0–16**:

| Index | 0 | 1–4 | 5–7 | 8–10 | 11–13 | 14–16 |
|-------|---|-----|-----|------|-------|-------|
| Label | START | Stone Age | Early Bronze | Late Bronze | Early Iron | Late Iron |
| Entry req. | — | — | 2 cities | 3 cities, 3 card groups | 4 cities, 9 cards, 5 groups | 5 cities, point value |
| Finish | | | | | | 16 = FINISH |

At the end of each turn every player's marker moves **one space right** unless
gated by epoch-entry requirements (frozen, rule 33.3) or pushed **left** when the
player holds no cities (rule 33.4, except in the Stone Age which has no city
requirement).

### Spreadsheet mapping (for reference)

- Sheet "AST", row 8 (`POS->`): `Q=0` (START), gap at `R`, `S=1 … AH=16`, `AI`=FINISH.
- Header rows 6–7 give the generic age bands and their criteria (above).
- Rows 9–20: one row per civilization. The `'X'` cells and the 1200–1900 numbers
  are one *saved game's* state (current marker + Late-Iron calendar/score labels),
  **not** static config. Track-cell backgrounds are a uniform gold; the per-civ
  age-length asymmetry is therefore authored by us (see §4), not extracted.

## 2. Authentic civilization colours (from the sheet)

Only the 9 factions present in `GameFaction` are listed; sheet ARGB → linear srgb.

| Faction | Sheet ARGB | Notes |
|---------|-----------|-------|
| Africa  | `FF993300` | brown |
| Iberia  | `FFFF0000` | red |
| Illyria | `FFFFFF00` | yellow |
| Thrace  | `FF008000` | green |
| Crete   | `FF99CC00` | lime |
| Asia    | `FFFF9900` | orange |
| Assyria | `FF3366FF` | blue |
| Babylon | `FFCCFFFF` | pale cyan |
| Egypt   | `FFFFFFCC` | cream |

(Sheet also defines Arabia `FF969696`, Persia `FFCC99FF`, Indus `FFFFCC99` — not
in the game yet, recorded here for completeness.)

## 3. Data model

Split static definition (resource) from dynamic marker (component):

- **`AstTrack`** — `Resource`. The static board: a `Vec<AstSpace>` of 17 entries,
  each with `index`, `epoch`, and the entry requirement. Capable of per-civ
  overrides (`HashMap<GameFaction, Vec<AstSpace>>`) so we can later encode age
  lengths that differ by civ; initially every civ uses the standard layout.
- **`AstSpace`** — `index: u8`, `epoch: AstEpoch`, `is_finish: bool`.
- **`AstEpoch`** — existing enum; fix `for_space` to the 0–16 indexing above and
  keep `min_cities` / `min_card_groups` / `min_card_count`. Late-Iron point value
  handled by the track/space, not the epoch.
- **`AstPosition`** — existing per-player `Component`; re-baseline so `space` is
  **0-indexed** (0 = START) to match the sheet and `AstTrack`.

## 4. Per-civ asymmetry (future-friendly hook)

`AstTrack` carries optional per-faction overrides. Milestone M1 ships the standard
band layout for all; a later pass can mine the "ASTCalc" sheet / board scans for
real per-civ Late-Iron point values and age lengths and drop them into the
override map without touching consumers.

## 5. UI (lava_ui_builder)

A horizontal A.S.T. panel:
- 17 cells left→right, age bands grouped and labelled (Stone/Bronze/Iron) with the
  entry criteria as sub-labels, mirroring the sheet header.
- Each player's marker rendered as a faction-coloured chip sitting on its space;
  multiple markers on one space stack.
- Rebuilt/repositioned when `AstPosition` changes (after `MoveSuccessionMarkers`).
- Lives in the `Playing` state; toggleable.

## 6. Milestones

- [x] **M1 — Data model & resource.** `AstTrack`/`AstSpace`, fix `AstEpoch::for_space`
  to 0–16, re-baseline `AstPosition` to 0-indexed, init resource in `SuccessionPlugin`,
  seed `AstPosition` for players at game start. Unit tests for epoch boundaries.
- [x] **M2 — Advancement logic.** Rewrite `advance_succession_markers` against
  `AstTrack`: one-step advance, epoch-entry gating, freeze (33.3), backward move
  (33.4), clamp at FINISH. Tests for advance / freeze / retreat / finish.
- [x] **M3 — AST UI.** lava_ui_builder panel: track cells, age bands + criteria,
  faction-coloured markers positioned per `AstPosition`, live updates.
- [x] **M4 — Integration & polish.** Faction colour helper (`ast_faction_color`),
  FINISH/game-end log hook (rule 34.1A), F8 visibility toggle, UI-construction tests.

Commit at the end of each milestone.

## 7. Known follow-ups (not in scope here)

- Full victory scoring (rule 35) and a real game-over state — only the FINISH
  *trigger* (rule 34.1A) is logged today.
- Per-civ Late-Iron point values / age-length overrides mined from the "ASTCalc"
  sheet, dropped into `AstTrack::overrides`.
- Marker positioning currently assumes the standard 17-cell geometry; revisit if
  per-civ overrides change the cell count.
</content>
