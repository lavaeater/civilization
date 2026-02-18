use crate::civilization::{CivCardName, CivCardType, Credits};
use bevy::asset::{Asset, Handle};
use bevy::platform::collections::HashMap;
use bevy::prelude::{Resource, TypePath};
use enumflags2::BitFlags;
use serde::{Deserialize, Serialize};

#[derive(Resource)]
pub struct CardHandle(pub Handle<AvailableCivCards>);

#[derive(Resource, Asset, Default, Serialize, Deserialize, TypePath, Clone)]
pub struct AvailableCivCards {
    pub cards: Vec<CivCardDefinition>
}

#[derive(Asset, Debug, Serialize, Deserialize, TypePath, Clone)]
pub struct CivCardDefinition {
    pub name: CivCardName,
    pub description: String,
    pub card_type: BitFlags<CivCardType>,
    pub cost: u32,
    pub credits: Vec<Credits>,
    pub prerequisites: Vec<CivCardName>,
}