pub mod city_construction_events;
pub mod city_support_events;
pub mod general_events;
pub mod movement_events;

pub mod prelude {
    pub use crate::civilization::events::city_construction_events::*;
    pub use crate::civilization::events::city_support_events::*;
    pub use crate::civilization::game_moves::events::*;
    pub use crate::civilization::events::general_events::*;
    pub use crate::civilization::events::movement_events::*;
}