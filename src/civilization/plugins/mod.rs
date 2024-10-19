pub mod census_plugin;
pub mod city_construction_plugin;
pub mod city_support_plugin;
pub mod conflict_plugin;
pub mod game_moves_plugin;
pub mod general_plugin;
pub mod movement_plugin;
pub mod population_expansion_plugin;
pub mod remove_surplus_plugin;
pub mod trade_plugin;
pub mod trade_card_plugin;

pub mod prelude {
    pub use crate::civilization::plugins::census_plugin::*;
    pub use crate::civilization::plugins::city_construction_plugin::*;
    pub use crate::civilization::plugins::city_support_plugin::*;
    pub use crate::civilization::plugins::conflict_plugin::*;
    pub use crate::civilization::plugins::game_moves_plugin::*;
    pub use crate::civilization::plugins::general_plugin::*;
    pub use crate::civilization::plugins::movement_plugin::*;
    pub use crate::civilization::plugins::population_expansion_plugin::*;
    pub use crate::civilization::plugins::remove_surplus_plugin::*;
    pub use crate::civilization::plugins::trade_plugin::*;
    pub use crate::civilization::plugins::trade_card_plugin::*;
}