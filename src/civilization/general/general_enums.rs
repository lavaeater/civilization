use bevy::prelude::Reflect;

#[derive(serde::Deserialize, Debug, Reflect, Copy, Clone, Eq, PartialEq, Default)]
pub enum GameFaction {
    #[default]
    Egypt,
    Crete,
}

