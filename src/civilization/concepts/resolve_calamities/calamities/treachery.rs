use bevy::prelude::{Entity, Reflect};

#[derive(Debug, Clone, Default, Reflect)]
pub struct TreacheryState {
    pub phase: TreacheryPhase,
    pub city_to_replace: Option<Entity>,
    pub beneficiary: Option<Entity>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect)]
pub enum TreacheryPhase {
    #[default]
    SelectCity,
    ApplyEffects,
    Complete,
}

impl TreacheryState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_trade(city: Entity, beneficiary: Entity) -> Self {
        Self {
            phase: TreacheryPhase::ApplyEffects,
            city_to_replace: Some(city),
            beneficiary: Some(beneficiary),
        }
    }

    pub fn without_trade(city: Entity) -> Self {
        Self {
            phase: TreacheryPhase::ApplyEffects,
            city_to_replace: Some(city),
            beneficiary: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Rule 30.221: traded Treachery transfers a city from the victim to the trader.
    #[test]
    fn with_trade_sets_beneficiary_and_city() {
        let city = crate::test_utils::create_test_entity();
        let beneficiary = crate::test_utils::create_test_entity();
        let state = TreacheryState::with_trade(city, beneficiary);
        assert_eq!(state.city_to_replace, Some(city));
        assert_eq!(state.beneficiary, Some(beneficiary));
        assert_eq!(state.phase, TreacheryPhase::ApplyEffects);
    }

    /// Rule 30.222: untraded Treachery just reduces the victim's own city — no beneficiary.
    #[test]
    fn without_trade_has_no_beneficiary() {
        let city = crate::test_utils::create_test_entity();
        let state = TreacheryState::without_trade(city);
        assert_eq!(state.city_to_replace, Some(city));
        assert_eq!(state.beneficiary, None);
    }
}
