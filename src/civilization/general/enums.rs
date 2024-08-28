use bevy::prelude::Reflect;

#[derive(Debug, Reflect, Copy, Clone, Eq, PartialEq)]
pub enum GameFaction {
    Egypt,
    Crete,
}