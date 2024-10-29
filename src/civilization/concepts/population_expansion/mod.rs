pub(crate) mod components;
pub(crate) mod events;
pub(crate) mod plugins;
pub(crate) mod systems;
pub(crate) mod triggers;

pub(crate) mod prelude {
    use super::components::*;
    use super::events::*;
    use super::plugins::*;
    use super::systems::*;
    use super::triggers::*;
}