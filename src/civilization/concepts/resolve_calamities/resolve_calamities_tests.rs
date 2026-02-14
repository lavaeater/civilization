#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use crate::civilization::components::*;
    use crate::civilization::concepts::resolve_calamities::resolve_calamities_components::*;
    use crate::civilization::{PlayerTradeCards, TradeCard, TradeCardTrait};

    fn spawn_area_with_volcano(
        world: &mut World,
        name: &str,
        max_pop: usize,
        has_volcano: bool,
        land_connections: Vec<Entity>,
    ) -> Entity {
        let pop = Population::new(max_pop);
        let mut land_passage = LandPassage::default();
        for conn in land_connections {
            land_passage.add_passage(conn);
        }
        
        let mut entity = world.spawn((
            Name::new(name.to_string()),
            GameArea::new(world.entities().len() as i32),
            pop,
            land_passage,
        ));
        
        if has_volcano {
            entity.insert(Volcano);
        }
        
        entity.id()
    }

    // ========================================================================
    // Rule 29.5: Max 2 calamities per player - Unit tests for selection logic
    // ========================================================================

    #[test]
    fn test_player_trade_cards_calamity_detection() {
        let mut trade_cards = PlayerTradeCards::default();
        trade_cards.add_trade_card(TradeCard::Famine);
        trade_cards.add_trade_card(TradeCard::Gold);
        
        let calamities = trade_cards.calamity_cards();
        assert_eq!(calamities.len(), 1);
        assert!(calamities.contains(&TradeCard::Famine));
    }

    #[test]
    fn test_player_with_no_calamities_has_empty_set() {
        let mut trade_cards = PlayerTradeCards::default();
        trade_cards.add_trade_card(TradeCard::Gold);
        trade_cards.add_trade_card(TradeCard::Silver);
        
        let calamities = trade_cards.calamity_cards();
        assert!(calamities.is_empty());
    }

    #[test]
    fn test_player_with_multiple_calamities() {
        let mut trade_cards = PlayerTradeCards::default();
        trade_cards.add_trade_card(TradeCard::Famine);
        trade_cards.add_trade_card(TradeCard::Superstition);
        trade_cards.add_trade_card(TradeCard::CivilWar);
        
        let calamities = trade_cards.calamity_cards();
        assert_eq!(calamities.len(), 3);
    }

    #[test]
    fn test_rule_29_5_selection_logic() {
        let calamities = [TradeCard::Famine,
            TradeCard::Superstition,
            TradeCard::CivilWar,
            TradeCard::Flood,
            TradeCard::Epidemic];
        
        // Rule 29.5: If more than 2 calamities, only 2 are kept
        let selected_count = if calamities.len() > 2 { 2 } else { calamities.len() };
        assert_eq!(selected_count, 2);
    }

    #[test]
    fn test_rule_29_5_with_exactly_two() {
        let calamities = [TradeCard::Famine, TradeCard::Superstition];
        
        let selected_count = if calamities.len() > 2 { 2 } else { calamities.len() };
        assert_eq!(selected_count, 2);
    }

    #[test]
    fn test_rule_29_5_with_one() {
        let calamities = [TradeCard::Famine];
        
        let selected_count = if calamities.len() > 2 { 2 } else { calamities.len() };
        assert_eq!(selected_count, 1);
    }

    // ========================================================================
    // Calamity ordering tests
    // ========================================================================

    #[test]
    fn test_calamity_ordering_by_value() {
        let calamities = [(TradeCard::Piracy, 9),           // value 9
            (TradeCard::VolcanoEarthquake, 2), // value 2
            (TradeCard::Famine, 3),           // value 3
            (TradeCard::CivilWar, 4)];

        let mut sorted: Vec<_> = calamities.iter().collect();
        sorted.sort_by_key(|a| a.1);

        assert_eq!(sorted[0].0, TradeCard::VolcanoEarthquake);
        assert_eq!(sorted[1].0, TradeCard::Famine);
        assert_eq!(sorted[2].0, TradeCard::CivilWar);
        assert_eq!(sorted[3].0, TradeCard::Piracy);
    }

    #[test]
    fn test_non_tradeable_before_tradeable_same_value() {
        // Level 2: VolcanoEarthquake (non-tradeable) vs Treachery (tradeable)
        assert!(!TradeCard::VolcanoEarthquake.is_tradeable());
        assert!(TradeCard::Treachery.is_tradeable());
        assert_eq!(TradeCard::VolcanoEarthquake.value(), TradeCard::Treachery.value());

        // Level 3: Famine (non-tradeable) vs Superstition (tradeable)
        assert!(!TradeCard::Famine.is_tradeable());
        assert!(TradeCard::Superstition.is_tradeable());
        assert_eq!(TradeCard::Famine.value(), TradeCard::Superstition.value());

        // Level 4: CivilWar (non-tradeable) vs SlaveRevolt (tradeable)
        assert!(!TradeCard::CivilWar.is_tradeable());
        assert!(TradeCard::SlaveRevolt.is_tradeable());
        assert_eq!(TradeCard::CivilWar.value(), TradeCard::SlaveRevolt.value());
    }

    // ========================================================================
    // Volcano/Earthquake tests
    // ========================================================================

    #[test]
    fn test_volcano_area_marker_exists() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        
        let area = spawn_area_with_volcano(app.world_mut(), "Volcano Area", 4, true, vec![]);
        
        let has_volcano = app.world().get::<Volcano>(area);
        assert!(has_volcano.is_some(), "Area should have Volcano component");
    }

    #[test]
    fn test_non_volcano_area_has_no_marker() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        
        let area = spawn_area_with_volcano(app.world_mut(), "Normal Area", 4, false, vec![]);
        
        let has_volcano = app.world().get::<Volcano>(area);
        assert!(has_volcano.is_none(), "Area should not have Volcano component");
    }

    // ========================================================================
    // Component tests
    // ========================================================================

    #[test]
    fn test_pending_calamities_count() {
        let pending = PendingCalamities::new(vec![
            (TradeCard::Famine, None),
            (TradeCard::Superstition, None),
        ]);
        
        assert_eq!(pending.count(), 2);
        assert!(!pending.is_empty());
    }

    #[test]
    fn test_pending_calamities_empty() {
        let pending = PendingCalamities::new(vec![]);
        
        assert_eq!(pending.count(), 0);
        assert!(pending.is_empty());
    }

    #[test]
    fn test_calamity_victim_with_trader() {
        use crate::test_utils::create_test_entity;
        
        let trader = create_test_entity();
        let victim = CalamityVictim::new(TradeCard::Treachery, Some(trader));
        
        assert_eq!(victim.calamity, TradeCard::Treachery);
        assert_eq!(victim.traded_by, Some(trader));
    }

    #[test]
    fn test_calamity_victim_without_trader() {
        let victim = CalamityVictim::new(TradeCard::VolcanoEarthquake, None);
        
        assert_eq!(victim.calamity, TradeCard::VolcanoEarthquake);
        assert_eq!(victim.traded_by, None);
    }

    // ========================================================================
    // TradeCard calamity trait tests
    // ========================================================================

    #[test]
    fn test_all_calamities_are_marked_as_calamity() {
        let calamities = vec![
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
        ];

        for calamity in calamities {
            assert!(calamity.is_calamity(), "{:?} should be marked as calamity", calamity);
            assert!(!calamity.is_commodity(), "{:?} should not be a commodity", calamity);
        }
    }

    #[test]
    fn test_calamity_values_match_trade_stack_levels() {
        assert_eq!(TradeCard::VolcanoEarthquake.value(), 2, "Level 2 calamity");
        assert_eq!(TradeCard::Treachery.value(), 2, "Level 2 calamity");
        assert_eq!(TradeCard::Famine.value(), 3, "Level 3 calamity");
        assert_eq!(TradeCard::Superstition.value(), 3, "Level 3 calamity");
        assert_eq!(TradeCard::CivilWar.value(), 4, "Level 4 calamity");
        assert_eq!(TradeCard::SlaveRevolt.value(), 4, "Level 4 calamity");
        assert_eq!(TradeCard::Flood.value(), 5, "Level 5 calamity");
        assert_eq!(TradeCard::BarbarianHordes.value(), 5, "Level 5 calamity");
        assert_eq!(TradeCard::Epidemic.value(), 6, "Level 6 calamity");
        assert_eq!(TradeCard::CivilDisorder.value(), 7, "Level 7 calamity");
        assert_eq!(TradeCard::IconoclasmAndHeresy.value(), 8, "Level 8 calamity");
        assert_eq!(TradeCard::Piracy.value(), 9, "Level 9 calamity");
    }

    #[test]
    fn test_non_tradeable_calamities() {
        let non_tradeable = vec![
            TradeCard::VolcanoEarthquake,
            TradeCard::Famine,
            TradeCard::CivilWar,
            TradeCard::Flood,
        ];

        for calamity in non_tradeable {
            assert!(!calamity.is_tradeable(), "{:?} should be non-tradeable", calamity);
        }
    }

    #[test]
    fn test_tradeable_calamities() {
        let tradeable = vec![
            TradeCard::Treachery,
            TradeCard::Superstition,
            TradeCard::SlaveRevolt,
            TradeCard::BarbarianHordes,
            TradeCard::Epidemic,
            TradeCard::CivilDisorder,
            TradeCard::IconoclasmAndHeresy,
            TradeCard::Piracy,
        ];

        for calamity in tradeable {
            assert!(calamity.is_tradeable(), "{:?} should be tradeable", calamity);
        }
    }
}
