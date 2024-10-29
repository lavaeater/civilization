pub(crate) mod systems;
pub(crate) mod plugins;
pub(crate) mod components;
pub(crate) mod resources;

pub(crate) mod prelude {
    pub use super::systems::*;
    pub use super::plugins::*;
    pub use super::components::*;
    pub use super::resources::*;
}
