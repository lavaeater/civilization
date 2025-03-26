pub mod conflict_triggers;
pub mod general_triggers;

pub mod prelude {
    pub use crate::civilization::triggers::conflict_triggers::*;
    pub use crate::civilization::game_moves::triggers::*;
    pub use crate::civilization::triggers::general_triggers::*;
}