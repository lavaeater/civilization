use bevy::prelude::Reflect;
use std::fmt::Display;

#[derive(
    serde::Deserialize, serde::Serialize, Debug, Reflect, Copy, Clone, Eq, PartialEq, Default, Hash,
)]
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

impl Display for GameFaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}
