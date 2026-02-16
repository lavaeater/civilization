use bevy::prelude::Component;
use enumflags2::{bitflags, BitFlags};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Write};

#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CivCardType {
    Arts,
    Crafts,
    Sciences,
    Civics,
    Religion,
}

#[derive(Component, Debug, Serialize, Deserialize)]
pub struct CivCardDefinition {
    pub name: CivCardName,
    pub description: String,
    pub card_type: BitFlags<CivCardType>,
    pub cost: u32,
    pub discounts: Vec<Credits>,
    pub prerequisites: Vec<CivCardName>,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CivCardName {
    Pottery = 1,
    ClothMaking = 2,
    Metalworking = 3,
    Agriculture = 4,
    RoadBuilding = 5,
    Mining = 6,
    Engineering = 7,
    Astronomy = 8,
    Coinage = 9,
    Medicine = 10,
    Mathematics = 11,
    DramaAndPoetry = 12,
    Music = 13,
    Architecture = 14,
    Literacy = 15,
    Law = 16,
    Military = 17,
    Philosophy = 18,
    Democracy = 19,
    Mysticism = 20,
    Deism = 21,
    Enlightenment = 22,
    Monotheism = 23,
    Theology = 24,
}

impl Display for CivCardName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CivCardName::Pottery => f.write_str("Pottery"),
            CivCardName::ClothMaking => f.write_str("Cloth Making"),
            CivCardName::Metalworking => f.write_str("Metalworking"),
            CivCardName::Agriculture => f.write_str("Agriculture"),
            CivCardName::RoadBuilding => f.write_str("Road Building"),
            CivCardName::Mining => f.write_str("Mining"),
            CivCardName::Engineering => f.write_str("Engineering"),
            CivCardName::Astronomy => f.write_str("Astronomy"),
            CivCardName::Coinage => f.write_str("Coinage"),
            CivCardName::Medicine => f.write_str("Medicine"),
            CivCardName::Mathematics => f.write_str("Mathematics"),
            CivCardName::DramaAndPoetry => f.write_str("Drama and Poetry"),
            CivCardName::Music => f.write_str("Music"),
            CivCardName::Architecture => f.write_str("Architecture"),
            CivCardName::Literacy => f.write_str("Literacy"),
            CivCardName::Law => f.write_str("Law"),
            CivCardName::Military => f.write_str("Military"),
            CivCardName::Philosophy => f.write_str("Philosophy"),
            CivCardName::Democracy => f.write_str("Democracy"),
            CivCardName::Mysticism => f.write_str("Mysticism"),
            CivCardName::Deism => f.write_str("Deism"),
            CivCardName::Enlightenment => f.write_str("Enlightenment"),
            CivCardName::Monotheism => f.write_str("Monotheism"),
            CivCardName::Theology => f.write_str("Theology"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Credits {
    ToType(CivCardType, u32),
    ToAll(u32),
    ToSpecificCard(CivCardName, u32),
}
pub fn mathematics() -> CivCardDefinition {
    CivCardDefinition {
        name: CivCardName::Mathematics,
        description: "Founded by Pythagoras in the 6th century BC. ".to_string(),
        card_type: CivCardType::Arts | CivCardType::Sciences,
        cost: 230,
        discounts: vec![
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
}
