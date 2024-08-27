use bevy::prelude::SubStates;
use crate::GameState;
use StateSet;

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(GameState = GameState::Playing)]
pub enum GameActivity {
    // CollectTaxes,
    #[default]
    PopulationExpansion,
    Census,
    // ShipConstruction,
    Movement,
    // Conflict,
    // CityConstruction,
    // RemoveSurplusPopulation,
    // CheckCitySupport,
    // AcquireTradeCards,
    // Trade,
    // ResolveCalamities,
    // AcquireCivilizationCards,
    // MoveSuccessionMarkers,
}