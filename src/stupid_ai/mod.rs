pub mod stupid_ai_components;
pub mod stupid_ai_events;
pub mod stupid_ai_plugin;
pub mod stupid_ai_systems;
pub mod stupid_ai_triggers;

pub mod prelude {
    pub use super::stupid_ai_components::*;
    pub use super::stupid_ai_events::*;
    pub use super::stupid_ai_plugin::*;
    pub use super::stupid_ai_systems::*;
    pub use super::stupid_ai_triggers::*;
}
