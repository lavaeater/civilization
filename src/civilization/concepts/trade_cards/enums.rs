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
        match self {
            TradeCardType::CommodityCard(commodity) => {
                write!(f, "{:?}", commodity)
                }
            TradeCardType::CalamityCard(calamity) => {
            write!(f, "{:?}", calamity)
        }
        }
        
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
        match self {
            Commodity::Ochre => {
                write!(f, "Ochre")  
            }
            Commodity::Hides => {
                write!(f, "Hides")
            }
            Commodity::Iron => {
                write!(f, "Iron")
            }
            Commodity::Papyrus => {
                write!(f, "Papyrus")
            }
            Commodity::Salt => {
                write!(f, "Salt")
            }
            Commodity::Timber => {
                write!(f, "Timber")
            }
            Commodity::Grain => {
                write!(f, "Grain")
            }
            Commodity::Oil => {
                write!(f, "Oil")
            }
            Commodity::Cloth => {
                write!(f, "Cloth")
            }
            Commodity::Wine => {
                write!(f, "Wine")
            }
            Commodity::Bronze => {
                write!(f, "Bronze")
            }
            Commodity::Silver => {
                write!(f, "Silver")
            }
            Commodity::Spices => {
                write!(f, "Spices")
            }
            Commodity::Resin => {
                write!(f, "Resin")
            }
            Commodity::Gems => {
                write!(f, "Gems")
            }
            Commodity::Dye => {
                write!(f, "Dye")
            }
            Commodity::Gold => {
                write!(f, "Gold")
            }
            Commodity::Ivory => {
                write!(f, "Ivory")
            }
        }
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
        match self {
            Calamity::VolcanoEarthquake => {
                write!(f, "Volcano Earthquake")
            }
            Calamity::Treachery => {
                write!(f, "Treachery")
            }
            Calamity::Famine => {
                write!(f, "Famine")
            }
            Calamity::Superstition => {
                write!(f, "Superstition")
            }
            Calamity::CivilWar => {
                write!(f, "Civil War")
            }
            Calamity::SlaveRevolt => {
                write!(f, "Slave Revolt")
            }
            Calamity::Flood => {
                write!(f, "Flood")
            }
            Calamity::BarbarianHordes => {
                write!(f, "Barbarian Hordes")
            }
            Calamity::Epidemic => {
                write!(f, "Epidemic")
            }
            Calamity::CivilDisorder => {
                write!(f, "Civil Disorder")
            }
            Calamity::IconoclasmAndHeresy => {
                write!(f, "Iconoclasm and Heresy")
            }
            Calamity::Piracy => {
                write!(f, "Piracy")
            }
        }
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