use crate::civilization::CivCardName;
use bevy::platform::collections::HashSet;
use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Serialize, Deserialize)]
pub struct PlayerCivilizationCards {
    pub cards: HashSet<CivCardName>,
}

#[derive(Component)]
pub struct PlayerAcquiringCivilizationCards;

#[derive(Component, Default)]
pub struct CivTradeUi;