//! Shared UI stacking tiers, bottom to top.
//!
//! World-space rendering (the map, tokens, ships, cities) and gizmos (movement
//! arrows, city-building/ship-placement previews) are a separate render pass
//! that Bevy always composites *beneath* `bevy_ui` -- these constants can't
//! and don't need to reorder anything against that pass, only against each
//! other. Every independently-spawned top-level UI root (each panel here is
//! its own tree with no shared parent) is a sibling for `ZIndex` purposes, so
//! a plain (non-global) `ZIndex` using these tiers is enough to order all of
//! them consistently.
//!
//! - [`Z_AREA_INDICATOR`]: per-area population/capacity overlays, positioned
//!   over the board itself.
//! - [`Z_ACTION_UI`]: the current phase's interactive controls (movement,
//!   ship construction, city construction) -- "what can I do right now."
//! - [`Z_PANEL`]: always-on status panels (AST, the Player Info/Trade
//!   Cards/Game State/Game Activity HUD strip).
//! - [`Z_DIALOG`]: modal human-decision UI that should never be obscured by
//!   anything above (calamity secondary-victim/faction-choice selection
//!   panels, civilization card purchase). Trade's offer/settlement modals use
//!   `GlobalZIndex` instead, which already always wins regardless of tier.

pub const Z_AREA_INDICATOR: i32 = 1;
pub const Z_ACTION_UI: i32 = 10;
pub const Z_PANEL: i32 = 20;
pub const Z_DIALOG: i32 = 50;
