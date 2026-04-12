pub mod volcano_earthquake;
pub mod treachery;
pub mod famine;
pub mod superstition;
pub mod civil_war;
pub mod slave_revolt;
pub mod flood;
pub mod barbarian_hordes;
pub mod epidemic;
pub mod civil_disorder;
pub mod iconoclasm_heresy;
pub mod piracy;

use bevy::prelude::{Component, Reflect, ReflectComponent};

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub enum ResolvingCalamity {
    VolcanoEarthquake(volcano_earthquake::VolcanoEarthquakeState),
    Treachery(treachery::TreacheryState),
    Famine(famine::FamineState),
    Superstition(superstition::SuperstitionState),
    CivilWar(civil_war::CivilWarState),
    SlaveRevolt(slave_revolt::SlaveRevoltState),
    Flood(flood::FloodState),
    BarbarianHordes(barbarian_hordes::BarbarianHordesState),
    Epidemic(epidemic::EpidemicState),
    CivilDisorder(civil_disorder::CivilDisorderState),
    IconoclasmAndHeresy(iconoclasm_heresy::IconoclasmHeresyState),
    Piracy(piracy::PiracyState),
}
