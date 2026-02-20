use crate::civilization::{CivCardName, CivCardType, Credits};
use bevy::asset::{Asset, Handle};
use bevy::platform::collections::HashSet;
use bevy::prelude::{Resource, TypePath};
use enumflags2::{BitFlags};
use serde::{Deserialize, Serialize};

#[derive(Resource)]
pub struct CardHandle(pub Handle<AvailableCivCards>);

#[derive(Resource, Asset, Default, Serialize, Deserialize, TypePath, Clone)]
pub struct AvailableCivCards {
    pub cards: Vec<CivCardDefinition>,
}

impl AvailableCivCards {
    pub fn get_cards(&self, card_type: &BitFlags<CivCardType>) -> Vec<&CivCardDefinition> {
        self.cards
            .iter()
            .filter(|card| card.card_type.contains(*card_type))
            .collect()
    }

    pub fn total_credits(&self, cards: &HashSet<CivCardName>) -> Vec<Credits> {
        self.cards
            .iter()
            .filter(|card| cards.contains(&card.name))
            .flat_map(|card| card.credits.clone())
            .collect()
    }
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

impl CivCardDefinition {
    pub fn calculate_cost(&self, credits: &Vec<Credits>) -> u32 {
        let total_credits = credits
            .iter()
            .map(|credit| match credit {
                Credits::ToType(card_type, credit) => {
                    let card_type = BitFlags::from(*card_type);
                    if self.card_type.intersects(card_type) {
                        *credit
                    } else {
                        0u32
                    }
                }
                Credits::ToAll(credit) => *credit,
                Credits::ToSpecificCard(card_name, credit) => {
                    if self.name == *card_name {
                        *credit
                    } else {
                        0u32
                    }
                }
            })
            .sum::<u32>();
        self.cost - total_credits
    }
}
