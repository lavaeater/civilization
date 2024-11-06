pub mod city_construction_plugin;
pub mod city_support_plugin;
pub mod conflict_plugin;
pub mod general_plugin;
pub mod movement_plugin;
pub mod remove_surplus_plugin;

pub mod prelude {
    pub use crate::civilization::concepts::trade::plugins::*;
    pub use crate::civilization::plugins::city_construction_plugin::*;
    pub use crate::civilization::plugins::city_support_plugin::*;
    pub use crate::civilization::plugins::conflict_plugin::*;
    pub use crate::civilization::game_moves::plugins::*;
    pub use crate::civilization::plugins::movement_plugin::*;
    pub use crate::civilization::plugins::remove_surplus_plugin::*;
}