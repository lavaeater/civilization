pub mod city_construction_components;
pub mod city_support_components;
pub mod conflict_components;
pub mod general_components;
pub mod movement_components;

pub mod prelude {
    pub use crate::civilization::components::city_construction_components::*;
    pub use crate::civilization::components::city_support_components::*;
    pub use crate::civilization::components::conflict_components::*;
    pub use crate::civilization::game_moves::components::*;
    pub use crate::civilization::components::general_components::population::*;
    pub use crate::civilization::components::general_components::*;
    pub use crate::civilization::components::movement_components::*;
}