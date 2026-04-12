# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Run the game (native)
cargo run

# Run tests
cargo test --no-fail-fast
cargo nextest run --failure-output final --no-fail-fast

# Run a single test
cargo test <test_name>
cargo nextest run <test_name>

# Lint
cargo clippy
cargo clippy --all-targets

# Web build (requires trunk + wasm32-unknown-unknown target)
trunk serve

# Android (requires cargo-apk)
cargo apk run -p mobile

# Nix environment (non-NixOS Linux)
nix develop --impure
# On nixgl: gl cargo run
```

Bacon is configured for watch-mode development (`.config/bacon.toml`).

## Architecture

This is a Bevy 0.18 ECS game implementing the Advanced Civilization board game. The game is structured as a turn-based activity loop driven by two Bevy states:

- `GameState` — top-level: `Loading`, `Menu`, `Playing`, `Sandbox`
- `GameActivity` — sub-state of `Playing`, represents the current game phase

### Game phases (in order)

`PrepareGame` → `StartGame` → `PopulationExpansion` → `Census` → `Movement` → `Conflict` → `CityConstruction` → `RemoveSurplusPopulation` → `CheckCitySupportAfterRemoveSurplusPopulation` → `AcquireTradeCards` → `Trade` → `ResolveCalamities` → `CheckCitySupportAfterResolveCalamities` → `AcquireCivilizationCards`

Each phase is a separate module under `src/civilization/concepts/`, containing its own Bevy plugin, systems, components, and events.

### Key structural patterns

- **Plugin composition**: `CivilizationPlugin` (`src/civilization/plugins/`) aggregates all concept plugins. Each game phase is its own plugin.
- **ECS events as commands**: Game actions go through events (in `src/civilization/events/`) rather than direct system calls. Player moves are managed via `src/civilization/game_moves/`.
- **Components**: Core game entities (tokens, areas, cities, populations) live in `src/civilization/components/`.
- **Triggers/Observers**: `src/civilization/triggers/` contains Bevy observers that react to component changes.
- **Pausing**: The `GamePaused` resource suspends game systems while keeping `GameActivity` substate intact (so UI state survives pauses).

### Notable modules

- `src/civilization/concepts/trade/` — largest and most complex module (~2600 lines in trade_systems.rs); handles commodity card trading between players
- `src/civilization/concepts/save_game/` — serialization of full game state via `serde`/`ron`
- `src/civilization/concepts/resolve_calamities/` — calamity event resolution (volcano, earthquake, civil war, etc.)
- `src/civilization/concepts/civ_cards/` — civilization card purchasing (partially implemented; see `civ_cards.md`)
- `src/stupid_ai/` — AI player that drives non-human factions
- `src/lava_ui_builder/` (local crate) — custom Bevy UI builder library used throughout

### Testing

Tests live in `tests/` (integration-style, using Bevy app builder helpers from `tests/mod.rs` and `src/test_utils.rs`). The `test-utils` feature flag exposes internal helpers. Nextest config is at `.config/nextest.toml`.
