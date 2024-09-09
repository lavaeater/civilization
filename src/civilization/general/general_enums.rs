use bevy::prelude::Reflect;

#[derive(Debug, Reflect, Copy, Clone, Eq, PartialEq, Default)]
pub enum GameFaction {
    #[default]
    Egypt,
    Crete,
}

