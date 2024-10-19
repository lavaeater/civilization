pub mod conflict_triggers;
pub mod game_moves_triggers;
pub mod general_triggers;
pub mod population_expansion_triggers;
pub mod trade_triggers;

pub mod prelude {
    pub use crate::civilization::triggers::conflict_triggers::*;
    pub use crate::civilization::triggers::game_moves_triggers::*;
    pub use crate::civilization::triggers::general_triggers::*;
    pub use crate::civilization::triggers::population_expansion_triggers::*;
    pub use crate::civilization::triggers::trade_triggers::*;
}