pub mod conflict_functions;
pub mod general_functions;

pub mod prelude {
    pub use crate::civilization::functions::conflict_functions::*;
    pub use crate::civilization::functions::general_functions::*;
}