pub mod census_systems;
pub mod city_construction_systems;
pub mod city_support_systems;
pub mod conflict_systems;
pub mod game_moves_systems;
pub mod general_systems;
pub mod movement_systems;
pub mod population_expansion_systems;
pub mod remove_surplus_systems;

pub mod prelude {
    pub use crate::civilization::systems::census_systems::*;
    pub use crate::civilization::systems::city_construction_systems::*;
    pub use crate::civilization::systems::city_support_systems::*;
    pub use crate::civilization::systems::conflict_systems::*;
    pub use crate::civilization::systems::game_moves_systems::*;
    pub use crate::civilization::systems::general_systems::*;
    pub use crate::civilization::systems::movement_systems::*;
    pub use crate::civilization::systems::population_expansion_systems::*;
    pub use crate::civilization::systems::remove_surplus_systems::*;
    pub use crate::civilization::concepts::trade::systems::trade_systems::*;
    pub use crate::civilization::concepts::trade::systems::trade_card_setup_systems::*;
    pub use crate::civilization::concepts::trade::systems::trade_card_systems::*;
}