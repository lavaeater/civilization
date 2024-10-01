use bevy::prelude::Reflect;

#[derive(serde::Deserialize, serde::Serialize, Debug, Reflect, Copy, Clone, Eq, PartialEq, Default, Hash)]
pub enum GameFaction {
    #[default]
    Egypt,
    Crete,
    Africa,
    Asia,
    Assyria,
    Babylon,
    Illyria,
    Iberia,
    Thrace,
}

