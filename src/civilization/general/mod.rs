pub mod general_plugin;
pub mod general_systems;
pub mod general_events;
pub mod general_enums;
pub mod general_components;

pub mod prelude {
    pub use super::general_components::*;
    pub use super::general_components::population::*;
    pub use super::general_enums::*;
    pub use super::general_events::*;
    pub use super::general_systems::*;
    pub use super::general_plugin::*;
}