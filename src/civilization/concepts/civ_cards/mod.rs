mod plugin;
mod enums;
mod messages;
mod assets_resources;
mod systems;
mod components;

pub use assets_resources::*;
pub use components::*;
pub use enums::*;
pub use messages::*;
pub use plugin::*;
pub use systems::*;

use bevy::asset::Asset;
use bevy::prelude::{Component, Resource, TypePath};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

pub fn mathematics() -> CivCardDefinition {
    CivCardDefinition {
        name: CivCardName::Mathematics,
        description: "Founded by Pythagoras in the 6th century BC. ".to_string(),
        card_type: CivCardType::Arts | CivCardType::Sciences,
        cost: 230,
        credits: vec![
            Credits::ToType(CivCardType::Arts, 5),
            Credits::ToType(CivCardType::Sciences, 20),
            Credits::ToSpecificCard(CivCardName::Law, 5),
            Credits::ToSpecificCard(CivCardName::Enlightenment, 10),
            Credits::ToSpecificCard(CivCardName::Philosophy, 25),
            Credits::ToSpecificCard(CivCardName::Theology, 25),
        ],
        prerequisites: Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_mathematics() {
        let maths = vec![mathematics()];
        if let Ok(maths_ron) = ron::to_string(&maths) {
            println!("Mathematics: {maths_ron}");
            assert_eq!(maths[0].name, CivCardName::Mathematics);
        }
    }
    
    #[test]
    fn load_cards() {
        let ron_str = std::fs::read_to_string("assets/definitions/civilization.cards.ron")
            .expect("Failed to read civilization.cards.ron");
        let cards: Vec<CivCardDefinition> =
            ron::from_str(&ron_str).expect("Failed to deserialize RON");
        
        
        assert_eq!(cards.len(), 24);
        
        let resource = AvailableCivCards { cards };
        let resource_str = ron::to_string(&resource).expect("Failed to serialize RON");
        println!("Resource: {resource_str}");
    }
}
