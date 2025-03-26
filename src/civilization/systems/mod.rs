pub mod city_construction_systems;
pub mod city_support_systems;
pub mod conflict_systems;
pub mod general_systems;
pub mod movement_systems;
pub mod remove_surplus_systems;

pub mod prelude {
    pub use crate::civilization::concepts::trade::systems::*;
    pub use crate::civilization::systems::city_construction_systems::*;
    pub use crate::civilization::systems::city_support_systems::*;
    pub use crate::civilization::systems::conflict_systems::*;
    pub use crate::civilization::game_moves::systems::*;
    pub use crate::civilization::systems::general_systems::*;
    pub use crate::civilization::systems::movement_systems::*;
    pub use crate::civilization::systems::remove_surplus_systems::*;
}