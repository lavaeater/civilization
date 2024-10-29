pub mod census_components;
pub mod city_construction_components;
pub mod city_support_components;
pub mod conflict_components;
pub mod game_moves_components;
pub mod general_components;
pub mod movement_components;
pub mod population_expansion_components;

pub mod prelude {
    pub use crate::civilization::components::census_components::*;
    pub use crate::civilization::components::city_construction_components::*;
    pub use crate::civilization::components::city_support_components::*;
    pub use crate::civilization::components::conflict_components::*;
    pub use crate::civilization::components::game_moves_components::*;
    pub use crate::civilization::components::general_components::population::*;
    pub use crate::civilization::components::general_components::*;
    pub use crate::civilization::components::movement_components::*;
    pub use crate::civilization::components::population_expansion_components::*;
    pub use crate::civilization::concepts::trade::components::trade_card_components::*;
}