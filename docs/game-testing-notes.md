# Testing Notes

## AST Design

If one looks at the ast.xslx, one would find that all factions have their own quirks on the ast. Som factions enter Early Bronze Age earlier than others and in the end game, some factions require a certain amount of points in the form of purchased civilization cards to advance.

**Not yet done.** This is a real, tracked gap (`AstTrack::overrides` in `docs/ast-design.md` §7 was scoped for exactly this -- per-civ Late-Iron point/age asymmetry mined from the `ASTCalc` sheet -- but never implemented). Bigger than a UI fix: needs mining `docs/ast.xslx`'s `ASTCalc` sheet for the actual per-faction numbers, then extending `AstTrack` and `advance_succession_markers`. Flagging rather than picking it up silently -- say the word and I'll scope it properly.

~~All factions are on one row, as well, they need to be spaced out a bit, I think.~~
**Fixed 2026-08-17** (`4805aee`): at game start every faction sits on space 0; the old vertical stacking stepped markers by 9px against a 15px marker, so up to 9 stacked factions overlapped into an unreadable blob. Markers sharing a space now arrange into a non-overlapping 2-column grid.

# Already done notes

## Save/Load

**Fixed 2026-08-18** (`7cd7ce1`, `450ffe1`, `8856a3f`): loading a save silently dropped several components that a fresh game gets via `setup_players` -- `ShipStock`/`PlayerShips` (broke the Player Info HUD and *all* move generation, human included), `Personality` (broke every AI move-selection system), and `PlayerCivilizationCards` (civ card bonuses silently stopped applying after a reload). The `Personality` gap is what caused the actual reported hang: an AI player whose turn came up after loading would never move again, no crash or error, just silent forever. Reproduced against the real save file that hung and confirmed fixed. Civ cards now round-trip through save/load (`owned_civ_cards` on `SavedPlayer`, backward-compatible with older save files via `#[serde(default)]`).

## Ships

~~Ships are supposed to be able to move independently, I think. So even if empty, they should be part of the movement phase.~~
**Fixed 2026-08-17** (`50f4563`): ship-move generation and `execute_ship_ferry` were both implicitly gated on the owning player having unmoved tokens in the ship's area. Ships now get a move (and can execute it) purely from being present in an area, independent of tokens (rule 23.1).

## UI Notes

It is crowded and cluttered - but also very much a work in progress.

* ~~Opacity - panels should change opacity to be less see-through, being able to see through them adds nothing, really, just adds clutter.~~
  **Fixed 2026-08-17** (`39da8e2`): the shared `LavaTheme` default background/border alpha was 0.25; raised to 0.9.
* ~~There is a "current action"-panel in the bottom right, I think it should move up into the action-area instead. So if a token can be moved from some area to another, it should be located as close as possible to that token without covering any of the targets for movement. That makes it more "free floating" and leaving the edges of the screen for player status.~~
  **Fixed 2026-08-17** (`e1aa601`): the movement controls panel now anchors near screen-center, riding along with `pan_camera_to_current_source`'s existing re-centering on the selected source area. This is a first-pass heuristic (no explicit target-avoidance), worth eyeballing further in play.
* ~~AST bottom left - perfect placement, but that means that all other panels should be moved to the top of the screen, really.~~
* ~~The collapse-button should then be placed at the bottom for the panels that are at the top now, and those would be Player Info, Trade Cards, Game State and lastly, Game Activity~~
  **Both fixed 2026-08-17** (`fe4e794`): the four-card HUD strip moved from bottom-left to the top of the screen; each card's collapse toggle now renders at its bottom edge via `with_collapsible_toggle_at_bottom`.
* ~~Area indicators and the actual UI interfere with each other. We need to check z-index for everything, so the map should be on the bottom, then tokens/boats/cities, then area capacity indicators etc. Then gizmos for transport, city building and whatnot. Then we get the action UI and after that we get the UI-panels such as AST, game state etc. Lastly, any dialogs, trade UI, civ card purchase UI etc, on the absolute top. Also, all the panels, like the AST etc, are still very see-through.~~
  **Fixed 2026-08-18** (`b81acbc`): z-index was ad-hoc -- the AST panel (5) actually sat *below* the action-UI panels (10), the HUD strip had no z-index at all, and the civ card purchase UI had neither a z-index nor any absolute positioning (rendered in plain document flow, easily obscured). Introduced four named tiers in `src/civilization/ui/z_layers.rs` -- `Z_AREA_INDICATOR (1) < Z_ACTION_UI (10) < Z_PANEL (20) < Z_DIALOG (50)` -- applied consistently everywhere a top-level UI root spawns; civ card purchase also got a proper full-screen centered root for the first time. Also bumped a handful of panels still stuck at the old 0.7 alpha (AST, area indicators, movement controls, HUD strip cards) up to 0.9. One thing this *can't* fix: the map/tokens/ships/cities and gizmos are a separate Bevy render pass that always composites beneath all UI regardless of z-index -- that ordering (world < gizmos < UI) is already correct by construction, not something a z-index tier can adjust.
* ~~We must hide all the area status markers (area number, population/capacity and so on) unless the area is involved in some sort of player action. So, for instance, during movement, we should show all the markers that are relevant, ie, the area that movement is going FROM and then all the areas that we can move TO. All the others should be hidden.~~
  **Fixed 2026-08-18** (`cc2e01e`): markers now only show for areas present in the human player's current `AvailableMoves` -- during Movement that's specifically the currently focused source plus its reachable targets (mirroring `draw_movement_arrows`, which already only draws arrows for that same source, not every source the player could still move from). Population Expansion/City Construction show whatever area each available move references; Trade and civ card acquisition have no board-area concept at all, so their markers stay hidden throughout.