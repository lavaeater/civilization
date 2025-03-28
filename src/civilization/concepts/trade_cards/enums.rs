use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Copy, Clone, Deserialize,
    Serialize, Debug, Eq, Hash, PartialEq, Reflect)]
pub enum TradeCardType {
    CommodityCard(Commodity),
    CalamityCard(Calamity),
}

impl Display for TradeCardType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Deserialize,
    Serialize, Debug, Eq, Hash, PartialEq, Reflect, Copy)]
pub enum Commodity {
    Ochre,
    Hides,
    Iron,
    Papyrus,
    Salt,
    Timber,
    Grain,
    Oil,
    Cloth,
    Wine,
    Bronze,
    Silver,
    Spices,
    Resin,
    Gems,
    Dye,
    Gold,
    Ivory,
}

impl Display for Commodity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Extend Commodity to iterate through all variants
impl Commodity {
    pub fn iter() -> impl Iterator<Item=Commodity> {
        [
            Commodity::Ochre, Commodity::Hides, Commodity::Iron, Commodity::Papyrus,
            Commodity::Salt, Commodity::Timber, Commodity::Grain, Commodity::Oil,
            Commodity::Cloth, Commodity::Wine, Commodity::Bronze, Commodity::Silver,
            Commodity::Spices, Commodity::Resin, Commodity::Gems, Commodity::Dye,
            Commodity::Gold, Commodity::Ivory,
        ]
            .iter()
            .cloned()
    }
}

#[derive(Clone, Copy, Deserialize,
    Serialize, Debug, Eq, Hash, PartialEq, Reflect)]
pub enum Calamity {
    VolcanoEarthquake,
    Treachery,
    Famine,
    Superstition,
    CivilWar,
    SlaveRevolt,
    Flood,
    BarbarianHordes,
    Epidemic,
    CivilDisorder,
    IconoclasmAndHeresy,
    Piracy,
}

impl Display for Calamity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Calamity {
    pub fn iter() -> impl Iterator<Item=Calamity> {
        [Calamity::VolcanoEarthquake, Calamity::Treachery, Calamity::Famine,
            Calamity::Superstition, Calamity::CivilWar, Calamity::SlaveRevolt,
            Calamity::Flood, Calamity::BarbarianHordes, Calamity::Epidemic,
            Calamity::CivilDisorder, Calamity::IconoclasmAndHeresy, Calamity::Piracy]
            .iter()
            .cloned()
    }
}