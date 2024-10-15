pub mod population_expansion_plugin;
pub mod population_expansion_systems;
pub mod population_expansion_components;
pub mod population_expansion_events;
pub mod population_expansion_triggers;

pub mod prelude {
    pub use super::population_expansion_components::*;
    pub use super::population_expansion_events::*;
    pub use super::population_expansion_plugin::*;
    pub use super::population_expansion_systems::*;
    pub use super::population_expansion_triggers::*;
}