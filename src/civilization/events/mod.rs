pub mod census_events;
pub mod city_construction_events;
pub mod city_support_events;
pub mod game_moves_events;
pub mod general_events;
pub mod movement_events;
pub mod population_expansion_events;
pub mod trade_events;

pub mod prelude {
    pub use crate::civilization::events::census_events::*;
    pub use crate::civilization::events::city_construction_events::*;
    pub use crate::civilization::events::city_support_events::*;
    pub use crate::civilization::events::game_moves_events::*;
    pub use crate::civilization::events::general_events::*;
    pub use crate::civilization::events::movement_events::*;
    pub use crate::civilization::events::population_expansion_events::*;
    pub use crate::civilization::events::trade_events::*;
}