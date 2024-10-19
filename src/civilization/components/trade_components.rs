use bevy::prelude::{Component, Entity, Reflect};
use bevy::utils::HashMap;
use crate::civilization::enums::trade_card_enums::Commodity;

#[derive(Debug, Component, Reflect)]
pub struct TradeOffer {
    pub player_a: Option<Entity>,
    pub player_b: Option<Entity>,
    pub player_a_commodities: HashMap<Commodity>
}