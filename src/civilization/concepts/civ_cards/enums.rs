use enumflags2::{bitflags, make_bitflags, BitFlags};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use bevy::prelude::Reflect;

#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CivCardType {
    Arts,
    Crafts,
    Sciences,
    Civics,
    Religion,
}

impl CivCardType {
    pub fn all_types() -> Vec<BitFlags<CivCardType>> {
        let arts = make_bitflags!(Self::Arts);
        let crafts = make_bitflags!(Self::Crafts);
        let sciences = make_bitflags!(Self::Sciences);
        let civics = make_bitflags!(Self::Civics);
        let religion = make_bitflags!(Self::Religion);
        
        // let arts_sciences = make_bitflags!(Self::{Arts | Sciences});
        // let crafts_sciences = make_bitflags!(Self::{Crafts | Sciences});
        // let arts_civics = make_bitflags!(Self::{Arts | Civics});
        // let religion_arts = make_bitflags!(Self::{Arts | Religion});
        
        
        vec![arts, crafts, sciences, civics, religion]//, arts_sciences, crafts_sciences, arts_civics, religion_arts]
    }
}

#[derive(Reflect, Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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