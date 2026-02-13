use bevy::prelude::{Entity, Message};
use crate::civilization::TradeCard;

#[derive(Message, Debug)]
pub struct ResolveNextCalamity;

#[derive(Message, Debug)]
pub struct ResolveVolcanoEarthquake {
    pub primary_victim: Entity,
    pub traded_by: Option<Entity>,
}

#[derive(Message, Debug)]
pub struct VolcanoEruption {
    pub volcano_area: Entity,
}

#[derive(Message, Debug)]
pub struct Earthquake {
    pub primary_victim: Entity,
    pub city_to_destroy: Entity,
    pub city_to_reduce: Option<Entity>,
}

#[derive(Message, Debug)]
pub struct CalamityResolved {
    pub player: Entity,
    pub calamity: TradeCard,
}
