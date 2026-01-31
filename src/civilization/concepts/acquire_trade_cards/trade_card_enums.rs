use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::hash::Hash;

pub trait TradeCardTrait: Copy + Clone + Reflect + Display + Eq + Hash + PartialEq {
    fn value(&self) -> usize;
    fn is_tradeable(&self) -> bool;
    fn is_commodity(&self) -> bool;
    fn is_calamity(&self) -> bool;
    fn number_of_cards(&self) -> usize;
}

#[derive(Clone, Deserialize, Serialize, Debug, Eq, Hash, PartialEq, Reflect, Copy)]
pub enum TradeCard {
    Ochre, //1
    Hides, //1
    Iron,  //2
    Papyrus, //2
    Salt, //3
    Timber, //3
    Grain, //4
    Oil, //4
    Cloth, //5
    Wine, //5
    Bronze, //6
    Silver, //6
    Spices, //7
    Resin, //7
    Gems, //8
    Dye, //8
    Gold, //9
    Ivory, //9
    VolcanoEarthquake, //2
    Treachery, //2, true
    Famine, //3, false
    Superstition, //3, true
    CivilWar, //4, false
    SlaveRevolt, //4, true
    Flood, //5, false
    BarbarianHordes, //5, true
    Epidemic, //6, true
    CivilDisorder, //7, true
    IconoclasmAndHeresy, //8, true
    Piracy, //9, true
}

impl TradeCardTrait for TradeCard {
    fn value(&self) -> usize {
        match *self {
            TradeCard::Ochre => 1,
            TradeCard::Hides => 1,
            TradeCard::Iron => 2,
            TradeCard::Papyrus => 2,
            TradeCard::Salt => 3,
            TradeCard::Timber => 3,
            TradeCard::Grain => 4,
            TradeCard::Oil => 4,
            TradeCard::Cloth => 5,
            TradeCard::Wine => 5,
            TradeCard::Bronze => 6,
            TradeCard::Silver => 6,
            TradeCard::Spices => 7,
            TradeCard::Resin => 7,
            TradeCard::Gems => 8,
            TradeCard::Dye => 8,
            TradeCard::Gold => 9,
            TradeCard::Ivory => 9,
            TradeCard::VolcanoEarthquake => 2,
            TradeCard::Treachery => 2,
            TradeCard::Famine => 3,
            TradeCard::Superstition => 3,
            TradeCard::CivilWar => 4,
            TradeCard::SlaveRevolt => 4,
            TradeCard::Flood => 5,
            TradeCard::BarbarianHordes => 5,
            TradeCard::Epidemic => 6,
            TradeCard::CivilDisorder => 7,
            TradeCard::IconoclasmAndHeresy => 8,
            TradeCard::Piracy => 9,
        }
    }

    fn is_tradeable(&self) -> bool {
        match *self {
            TradeCard::VolcanoEarthquake => false,
            TradeCard::Treachery => true,
            TradeCard::Famine => false,
            TradeCard::Superstition => true,
            TradeCard::CivilWar => false,
            TradeCard::SlaveRevolt => true,
            TradeCard::Flood => false,
            TradeCard::BarbarianHordes => true,
            TradeCard::Epidemic => true,
            TradeCard::CivilDisorder => true,
            TradeCard::IconoclasmAndHeresy => true,
            TradeCard::Piracy => true,
            _ => true,
        }
    }

    fn is_commodity(&self) -> bool {
        match *self {
            TradeCard::VolcanoEarthquake |
            TradeCard::Treachery |
            TradeCard::Famine |
            TradeCard::Superstition |
            TradeCard::CivilWar |
            TradeCard::SlaveRevolt |
            TradeCard::Flood |
            TradeCard::BarbarianHordes |
            TradeCard::Epidemic |
            TradeCard::CivilDisorder |
            TradeCard::IconoclasmAndHeresy |
            TradeCard::Piracy => false,
            _ => true,
        }
    }

    fn is_calamity(&self) -> bool {
        match *self {
            TradeCard::VolcanoEarthquake |
            TradeCard::Treachery |
            TradeCard::Famine |
            TradeCard::Superstition |
            TradeCard::CivilWar |
            TradeCard::SlaveRevolt |
            TradeCard::Flood |
            TradeCard::BarbarianHordes |
            TradeCard::Epidemic |
            TradeCard::CivilDisorder |
            TradeCard::IconoclasmAndHeresy |
            TradeCard::Piracy => true,
            _ => false,
        }
    }

    fn number_of_cards(&self) -> usize {
        match *self {
            TradeCard::Ochre => 7,
            TradeCard::Hides => 7,
            TradeCard::Iron => 8,
            TradeCard::Papyrus => 7,
            TradeCard::Salt => 9,
            TradeCard::Timber => 8,
            TradeCard::Grain => 8,
            TradeCard::Oil => 7,
            TradeCard::Cloth => 7,
            TradeCard::Wine => 6,
            TradeCard::Bronze => 6,
            TradeCard::Silver => 5,
            TradeCard::Spices => 6,
            TradeCard::Resin => 5,
            TradeCard::Gems => 5,
            TradeCard::Dye => 4,
            TradeCard::Gold => 5,
            TradeCard::Ivory => 4,
            TradeCard::VolcanoEarthquake => 2,
            TradeCard::Treachery => 2,
            TradeCard::Famine => 3,
            TradeCard::Superstition => 3,
            TradeCard::CivilWar => 4,
            TradeCard::SlaveRevolt => 4,
            TradeCard::Flood => 5,
            TradeCard::BarbarianHordes => 5,
            TradeCard::Epidemic => 6,
            TradeCard::CivilDisorder => 7,
            TradeCard::IconoclasmAndHeresy => 8,
            TradeCard::Piracy => 9,
        }
    }
}

impl Display for TradeCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TradeCard::Ochre => {
                write!(f, "Ochre")
            }
            TradeCard::Hides => {
                write!(f, "Hides")
            }
            TradeCard::Iron => {
                write!(f, "Iron")
            }
            TradeCard::Papyrus => {
                write!(f, "Papyrus")
            }
            TradeCard::Salt => {
                write!(f, "Salt")
            }
            TradeCard::Timber => {
                write!(f, "Timber")
            }
            TradeCard::Grain => {
                write!(f, "Grain")
            }
            TradeCard::Oil => {
                write!(f, "Oil")
            }
            TradeCard::Cloth => {
                write!(f, "Cloth")
            }
            TradeCard::Wine => {
                write!(f, "Wine")
            }
            TradeCard::Bronze => {
                write!(f, "Bronze")
            }
            TradeCard::Silver => {
                write!(f, "Silver")
            }
            TradeCard::Spices => {
                write!(f, "Spices")
            }
            TradeCard::Resin => {
                write!(f, "Resin")
            }
            TradeCard::Gems => {
                write!(f, "Gems")
            }
            TradeCard::Dye => {
                write!(f, "Dye")
            }
            TradeCard::Gold => {
                write!(f, "Gold")
            }
            TradeCard::Ivory => {
                write!(f, "Ivory")
            }
            TradeCard::VolcanoEarthquake => {
                write!(f, "Volcano Earthquake")
            }
            TradeCard::Treachery => {
                write!(f, "Treachery")
            }
            TradeCard::Famine => {
                write!(f, "Famine")
            }
            TradeCard::Superstition => {
                write!(f, "Superstition")
            }
            TradeCard::CivilWar => {
                write!(f, "Civil War")
            }
            TradeCard::SlaveRevolt => {
                write!(f, "Slave Revolt")
            }
            TradeCard::Flood => {
                write!(f, "Flood")
            }
            TradeCard::BarbarianHordes => {
                write!(f, "Barbarian Hordes")
            }
            TradeCard::Epidemic => {
                write!(f, "Epidemic")
            }
            TradeCard::CivilDisorder => {
                write!(f, "Civil Disorder")
            }
            TradeCard::IconoclasmAndHeresy => {
                write!(f, "Iconoclasm and Heresy")
            }
            TradeCard::Piracy => {
                write!(f, "Piracy")
            }
        }
    }
}

// Extend Commodity to iterate through all variants
impl TradeCard {
    pub fn iter() -> impl Iterator<Item =TradeCard> {
        [
            TradeCard::Ochre,
            TradeCard::Hides,
            TradeCard::Iron,
            TradeCard::Papyrus,
            TradeCard::Salt,
            TradeCard::Timber,
            TradeCard::Grain,
            TradeCard::Oil,
            TradeCard::Cloth,
            TradeCard::Wine,
            TradeCard::Bronze,
            TradeCard::Silver,
            TradeCard::Spices,
            TradeCard::Resin,
            TradeCard::Gems,
            TradeCard::Dye,
            TradeCard::Gold,
            TradeCard::Ivory,
            TradeCard::VolcanoEarthquake,
            TradeCard::Treachery,
            TradeCard::Famine,
            TradeCard::Superstition,
            TradeCard::CivilWar,
            TradeCard::SlaveRevolt,
            TradeCard::Flood,
            TradeCard::BarbarianHordes,
            TradeCard::Epidemic,
            TradeCard::CivilDisorder,
            TradeCard::IconoclasmAndHeresy,
            TradeCard::Piracy,
        ]
        .iter()
        .cloned()
    }
}
