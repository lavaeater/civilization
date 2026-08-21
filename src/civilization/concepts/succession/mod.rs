pub mod ast_ui_systems;
mod succession_components;
mod succession_plugin;
pub mod succession_systems;

pub use ast_ui_systems::{
    AstCell, AstMarker, AstUiRoot, ast_faction_color, spawn_ast_ui, update_ast_markers,
};
pub use succession_components::*;
pub use succession_plugin::SuccessionPlugin;
pub use succession_systems::{advance_succession_markers, determine_winner};
