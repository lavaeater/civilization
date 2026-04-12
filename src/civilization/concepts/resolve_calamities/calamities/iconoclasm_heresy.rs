use bevy::prelude::{Entity, Reflect};

/// State for resolving Iconoclasm and Heresy (rule 30.81).
///
/// Default: 4 cities reduced (30.811). Modifiers are cumulative (30.817):
/// - Law       : −1 city (30.812)
/// - Philosophy: −1 city (30.813)
/// - Theology  : −3 cities (30.814)
/// - Monotheism: +1 city (30.815)
/// - Road Building: +1 city (30.816)
///
/// Secondary: primary victim orders reduction of 2 other players' cities (30.818).
/// Secondary protections (30.819):
/// - Philosophy holder: cannot lose more than 1 city
/// - Theology holder: cannot be named as secondary victim
#[derive(Debug, Clone, Default, Reflect)]
pub struct IconoclasmHeresyState {
    pub phase: IconoclasmHeresyPhase,
    /// Number of the primary victim's cities to reduce.
    pub cities_to_reduce: usize,
    /// Number of secondary-victim cities the primary victim must order reduced (always 2).
    pub secondary_cities: usize,
    pub selected_cities: Vec<Entity>,
    pub immune_player: Option<Entity>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect)]
pub enum IconoclasmHeresyPhase {
    #[default]
    ComputeEffects,
    SelectCities,
    SelectSecondaryVictims,
    ApplySecondaryLosses,
    Complete,
}

impl IconoclasmHeresyState {
    pub fn new() -> Self {
        Self {
            cities_to_reduce: 4,
            secondary_cities: 2,
            ..Default::default()
        }
    }

    /// Law: reduces primary cities by 1 (30.812).
    pub fn with_law(mut self) -> Self {
        self.cities_to_reduce = self.cities_to_reduce.saturating_sub(1);
        self
    }

    /// Philosophy: reduces primary cities by 1 (30.813).
    pub fn with_philosophy(mut self) -> Self {
        self.cities_to_reduce = self.cities_to_reduce.saturating_sub(1);
        self
    }

    /// Theology: reduces primary cities by 3 (30.814).
    pub fn with_theology(mut self) -> Self {
        self.cities_to_reduce = self.cities_to_reduce.saturating_sub(3);
        self
    }

    /// Monotheism: increases primary cities by 1 (30.815).
    pub fn with_monotheism(mut self) -> Self {
        self.cities_to_reduce += 1;
        self
    }

    /// Road Building: increases primary cities by 1 (30.816).
    pub fn with_road_building(mut self) -> Self {
        self.cities_to_reduce += 1;
        self
    }

    pub fn with_immune_player(mut self, player: Entity) -> Self {
        self.immune_player = Some(player);
        self
    }

    pub fn select_city(&mut self, city_area: Entity) {
        if self.selected_cities.len() < self.cities_to_reduce {
            self.selected_cities.push(city_area);
        }
    }

    pub fn selection_complete(&self) -> bool {
        self.selected_cities.len() >= self.cities_to_reduce
    }
}
