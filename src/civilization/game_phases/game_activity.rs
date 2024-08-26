use bevy::prelude::SubStates;

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(AppState = GameState::Playing)]
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