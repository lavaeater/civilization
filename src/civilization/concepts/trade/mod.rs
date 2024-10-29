pub(crate) mod resources;
pub(crate) mod systems;
pub(crate) mod triggers;
pub(crate) mod events;
pub(crate) mod plugins;

pub(crate) mod prelude {
    use super::resources::*;
    use super::systems::*;
    use super::triggers::*;
    use super::events::*;
    use super::plugins::*;
}