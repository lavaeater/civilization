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
            assert!(calamity.is_calamity(), "{calamity:?} should be marked as calamity");
            assert!(!calamity.is_commodity(), "{calamity:?} should not be a commodity");
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
            assert!(!calamity.is_tradeable(), "{calamity:?} should be non-tradeable");
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
            assert!(calamity.is_tradeable(), "{calamity:?} should be tradeable");
        }
    }

    // ========================================================================
    // Rule 30.511: Flood primary-victim loss cap (ECS-level, via advance_flood)
    // ========================================================================
    //
    // Regression coverage for a real bug: `advance_flood`'s ApplyPrimaryLoss
    // phase previously ignored `FloodState::primary_max_loss` entirely and
    // instead wiped ALL tokens (any owner) from the flood plain AND every area
    // adjacent to it via `ClearAllTokens` — behavior copied from Volcano's
    // eruption logic (rule 30.211), which really does affect adjacent areas,
    // but Flood (rule 30.511) does not. Fixed to remove at most
    // `primary_max_loss` unit points from the primary victim, in the flood
    // plain area only.

    use bevy::ecs::system::RunSystemOnce;
    use crate::civilization::concepts::resolve_calamities::calamities::{
        ResolvingCalamity,
        flood::{FloodPhase, FloodState},
    };
    use crate::civilization::concepts::resolve_calamities::context::{
        ActiveCalamityResolution, CalamityContext,
    };
    use crate::civilization::concepts::resolve_calamities::resolve_calamities_systems::advance_flood;

    #[test]
    fn flood_primary_loss_is_capped_and_does_not_touch_adjacent_areas() {
        let mut world = World::new();
        world.init_resource::<bevy::prelude::Messages<crate::civilization::concepts::resolve_calamities::resolve_calamities_events::CalamityResolved>>();
        world.init_resource::<crate::civilization::concepts::resolve_calamities::resolve_calamities_ui_components::FloodSelectionState>();

        let victim = world.spawn(PlayerCities::default()).id();
        let other_player = world.spawn_empty().id();

        // Adjacent area: belongs to a different player entirely — must be untouched.
        let mut adjacent_pop = Population::new(6);
        let adjacent_tokens: Vec<Entity> = (0..3).map(|_| world.spawn_empty().id()).collect();
        for &t in &adjacent_tokens {
            adjacent_pop.add_token_to_area(other_player, t);
        }
        let adjacent_area = world
            .spawn((Name::new("adjacent"), GameArea::new(1), adjacent_pop, LandPassage::default()))
            .id();

        // Flood plain: victim has 15 tokens on board here; cap is 10 (test value).
        let mut fp_pop = Population::new(20);
        let victim_tokens: Vec<Entity> = (0..15).map(|_| world.spawn_empty().id()).collect();
        for &t in &victim_tokens {
            fp_pop.add_token_to_area(victim, t);
        }
        let mut fp_land_passage = LandPassage::default();
        fp_land_passage.add_passage(adjacent_area);
        let fp_area = world
            .spawn((
                Name::new("flood plain"),
                GameArea::new(2),
                FloodPlain,
                fp_pop,
                fp_land_passage,
            ))
            .id();

        let mut state = FloodState::new();
        state.phase = FloodPhase::ApplyPrimaryLoss;
        state.flood_plain_area = Some(fp_area);
        state.primary_max_loss = 10;

        let context = CalamityContext::new(TradeCard::Flood, victim, None);
        world.entity_mut(victim).insert((
            ActiveCalamityResolution::new(context),
            ResolvingCalamity::Flood(state),
        ));

        world.run_system_once(advance_flood).unwrap();

        // Victim lost exactly the capped amount (10), not all 15, from the flood plain.
        let fp_pop_after = world.get::<Population>(fp_area).unwrap();
        assert_eq!(fp_pop_after.population_for_player(victim), 5); // 15 - 10

        // The adjacent area — and the other player's tokens in it — are untouched.
        let adjacent_pop_after = world.get::<Population>(adjacent_area).unwrap();
        assert_eq!(adjacent_pop_after.population_for_player(other_player), 3);
    }

    #[test]
    fn flood_does_not_destroy_a_city_it_does_not_own() {
        let mut world = World::new();
        world.init_resource::<bevy::prelude::Messages<crate::civilization::concepts::resolve_calamities::resolve_calamities_events::CalamityResolved>>();
        world.init_resource::<crate::civilization::concepts::resolve_calamities::resolve_calamities_ui_components::FloodSelectionState>();

        let victim = world.spawn(PlayerCities::default()).id();
        let other_player = world.spawn_empty().id();
        let other_city_token = world.spawn_empty().id();

        // The flood plain has a city, but it belongs to `other_player`, not the victim.
        let fp_pop = Population::new(6);
        let fp_area = world
            .spawn((
                Name::new("flood plain"),
                GameArea::new(1),
                FloodPlain,
                fp_pop,
                LandPassage::default(),
                BuiltCity::new(other_player, other_city_token),
            ))
            .id();

        let mut state = FloodState::new();
        state.phase = FloodPhase::ApplyPrimaryLoss;
        state.flood_plain_area = Some(fp_area);
        state.primary_max_loss = 17;

        let context = CalamityContext::new(TradeCard::Flood, victim, None);
        world.entity_mut(victim).insert((
            ActiveCalamityResolution::new(context),
            ResolvingCalamity::Flood(state),
        ));

        world.run_system_once(advance_flood).unwrap();

        // The other player's city must still be standing — the flood only affects
        // the primary victim, and the victim doesn't own this city.
        assert!(world.get::<BuiltCity>(fp_area).is_some());
    }

    // ========================================================================
    // Rule 29.5: the calamities over the two-per-turn limit are discarded
    // ========================================================================

    /// 29.5 caps a player at two calamities per turn and says the rest "are
    /// returned to the appropriate trade card stacks"; 29.4 adds that calamity
    /// cards cannot be held for future turns. The excess used to stay in the
    /// player's hand, so a third calamity kept coming back every turn until it
    /// happened to be one of the two drawn.
    #[test]
    fn calamities_beyond_the_two_per_turn_limit_are_discarded_from_hand() {
        use crate::civilization::concepts::resolve_calamities::resolve_calamities_systems::start_calamity_resolution;
        use crate::GameActivity;

        let mut world = World::new();
        world.init_resource::<NextState<GameActivity>>();

        let mut cards = PlayerTradeCards::default();
        cards.add_trade_card(TradeCard::Famine);
        cards.add_trade_card(TradeCard::VolcanoEarthquake);
        cards.add_trade_card(TradeCard::Flood);
        cards.add_trade_card(TradeCard::Gold); // commodity, must survive

        let player = world.spawn((Player, Name::new("victim"), cards)).id();

        world.run_system_once(start_calamity_resolution).unwrap();

        let pending = world.get::<PendingCalamities>(player).expect("pending calamities attached");
        assert_eq!(pending.calamities.len(), 2, "29.5 caps resolution at two");

        let hand = world.get::<PlayerTradeCards>(player).unwrap();
        assert_eq!(
            hand.calamity_cards().len(),
            2,
            "only the two being resolved may remain in hand -- the third is returned to the stacks"
        );
        for (calamity, _) in &pending.calamities {
            assert!(hand.has_trade_card(*calamity), "the two selected are still held for resolution");
        }
        assert_eq!(hand.number_of_cards_for_trade_card(TradeCard::Gold), 1, "commodities untouched");
    }

    /// Two or fewer calamities are all resolved -- nothing is discarded.
    #[test]
    fn two_calamities_are_both_kept_for_resolution() {
        use crate::civilization::concepts::resolve_calamities::resolve_calamities_systems::start_calamity_resolution;
        use crate::GameActivity;

        let mut world = World::new();
        world.init_resource::<NextState<GameActivity>>();

        let mut cards = PlayerTradeCards::default();
        cards.add_trade_card(TradeCard::Famine);
        cards.add_trade_card(TradeCard::Flood);

        let player = world.spawn((Player, Name::new("victim"), cards)).id();

        world.run_system_once(start_calamity_resolution).unwrap();

        let pending = world.get::<PendingCalamities>(player).unwrap();
        assert_eq!(pending.calamities.len(), 2);
        assert_eq!(world.get::<PlayerTradeCards>(player).unwrap().calamity_cards().len(), 2);
    }

    // ========================================================================
    // Rules 29.61/30.221: who traded the calamity here
    // ========================================================================
    //
    // The trader is barred from being named a secondary victim (29.61) and is
    // the beneficiary of Treachery (30.221). `PendingCalamities` carried a
    // `traded_by` slot for this all along, but nothing ever filled it in --
    // it was hard-coded to `None`, so Treachery never had a beneficiary and
    // the trader could be picked as a secondary victim of their own gift.

    #[test]
    fn a_traded_calamity_records_who_handed_it_over() {
        let mut giver_hand = PlayerTradeCards::default();
        giver_hand.add_trade_card(TradeCard::Treachery);

        let giver = Entity::from_raw_u32(7).unwrap();
        let mut receiver_hand = PlayerTradeCards::default();
        receiver_hand.add_traded_cards(TradeCard::Treachery, 1, giver);

        assert_eq!(receiver_hand.calamity_traded_by(TradeCard::Treachery), Some(giver));
    }

    #[test]
    fn a_drawn_calamity_has_no_trader() {
        let mut hand = PlayerTradeCards::default();
        hand.add_trade_card(TradeCard::Treachery);
        assert_eq!(hand.calamity_traded_by(TradeCard::Treachery), None);
    }

    /// Passing the card onwards must not leave the previous trader attached --
    /// rule 29.3 lets a tradable calamity change hands any number of times,
    /// and only the last hand-off counts.
    #[test]
    fn trading_a_calamity_onwards_clears_the_stale_provenance() {
        let first = Entity::from_raw_u32(7).unwrap();
        let mut hand = PlayerTradeCards::default();
        hand.add_traded_cards(TradeCard::Treachery, 1, first);
        assert_eq!(hand.calamity_traded_by(TradeCard::Treachery), Some(first));

        hand.remove_n_trade_cards(1, TradeCard::Treachery);
        assert_eq!(hand.calamity_traded_by(TradeCard::Treachery), None);

        // Re-acquired from someone else later in the same round.
        let second = Entity::from_raw_u32(9).unwrap();
        hand.add_traded_cards(TradeCard::Treachery, 1, second);
        assert_eq!(hand.calamity_traded_by(TradeCard::Treachery), Some(second));
    }

    #[test]
    fn start_calamity_resolution_carries_the_trader_into_pending_calamities() {
        use crate::civilization::concepts::resolve_calamities::resolve_calamities_systems::start_calamity_resolution;
        use crate::GameActivity;

        let mut world = World::new();
        world.init_resource::<NextState<GameActivity>>();

        let trader = world.spawn((Player, Name::new("trader"), PlayerTradeCards::default())).id();

        let mut cards = PlayerTradeCards::default();
        cards.add_traded_cards(TradeCard::Treachery, 1, trader);
        cards.add_trade_card(TradeCard::Famine); // drawn, not traded
        let victim = world.spawn((Player, Name::new("victim"), cards)).id();

        world.run_system_once(start_calamity_resolution).unwrap();

        let pending = world.get::<PendingCalamities>(victim).unwrap();
        let treachery = pending
            .calamities
            .iter()
            .find(|(card, _)| *card == TradeCard::Treachery)
            .expect("treachery pending");
        assert_eq!(treachery.1, Some(trader), "the trader is recorded as beneficiary/immune");

        let famine = pending
            .calamities
            .iter()
            .find(|(card, _)| *card == TradeCard::Famine)
            .expect("famine pending");
        assert_eq!(famine.1, None, "a drawn calamity implicates nobody");
    }

    // ========================================================================
    // Rules 29.62/29.63: the victim chooses which of their own units to lose
    // ========================================================================
    //
    // The calamity dictates how many unit points are lost; picking the units
    // belongs to their owner. Before this, every unit-point loss was taken in
    // arbitrary `PlayerAreas` iteration order even for a human victim, so the
    // human got no prompt at all and tokens vanished from areas they would
    // never have picked.

    #[test]
    fn famine_primary_loss_pauses_for_a_human_victim_and_removes_exactly_what_they_chose() {
        use crate::civilization::concepts::resolve_calamities::calamities::famine::{FaminePhase, FamineState};
        use crate::civilization::concepts::resolve_calamities::resolve_calamities_systems::advance_famine;
        use crate::civilization::concepts::resolve_calamities::resolve_calamities_ui_components::{
            AwaitingHumanCalamitySelection, FamineSelectionState, UnitLossSelectionState,
        };
        use crate::stupid_ai::IsHuman;

        let mut world = World::new();
        world.init_resource::<bevy::prelude::Messages<crate::civilization::concepts::resolve_calamities::resolve_calamities_events::CalamityResolved>>();
        world.init_resource::<FamineSelectionState>();
        world.init_resource::<UnitLossSelectionState>();

        let victim = world.spawn((Player, IsHuman)).id();

        // Two areas, 6 tokens each: 12 available against a 4-point loss, so
        // there is a genuine choice to make.
        let mut victim_areas = PlayerAreas::default();
        let mut areas = Vec::new();
        for n in 0..2 {
            let mut pop = Population::new(20);
            let mut tokens = Vec::new();
            for _ in 0..6 {
                let token = world.spawn_empty().id();
                pop.add_token_to_area(victim, token);
                tokens.push(token);
            }
            let area = world
                .spawn((Name::new(format!("area {n}")), GameArea::new(n), pop, LandPassage::default()))
                .id();
            for token in tokens {
                victim_areas.add_token_to_area(area, token);
            }
            areas.push(area);
        }
        world.entity_mut(victim).insert(victim_areas);

        let mut state = FamineState::new();
        state.phase = FaminePhase::ComputeLosses;
        state.primary_loss = 4;

        let context = CalamityContext::new(TradeCard::Famine, victim, None);
        world.entity_mut(victim).insert((
            ActiveCalamityResolution::new(context),
            ResolvingCalamity::Famine(state),
        ));

        // First pass: pause for input, touch nothing.
        world.run_system_once(advance_famine).unwrap();

        assert!(
            world.get::<AwaitingHumanCalamitySelection>(victim).is_some(),
            "human victim should be prompted to choose their losses"
        );
        assert_eq!(world.get::<Population>(areas[0]).unwrap().population_for_player(victim), 6);
        assert_eq!(world.get::<Population>(areas[1]).unwrap().population_for_player(victim), 6);
        let ResolvingCalamity::Famine(ref waiting) = *world.get::<ResolvingCalamity>(victim).unwrap() else { panic!() };
        assert_eq!(waiting.phase, FaminePhase::ComputeLosses, "must not advance while waiting");

        {
            let selection = world.resource::<UnitLossSelectionState>();
            assert_eq!(selection.acting_player, Some(victim));
            assert_eq!(selection.total_budget, 4);
            assert_eq!(selection.total_available(), 12);
        }

        // The human puts the whole loss on the second area, as the UI buttons would.
        {
            let mut selection = world.resource_mut::<UnitLossSelectionState>();
            let second = areas[1];
            while selection.current_area().map(|(a, _, _)| a) != Some(second) {
                selection.next_area();
            }
            for _ in 0..4 {
                assert!(selection.increment_current());
            }
            assert!(selection.selection_valid());
        }
        world.entity_mut(victim).remove::<AwaitingHumanCalamitySelection>();

        // Second pass: apply exactly that.
        world.run_system_once(advance_famine).unwrap();

        assert_eq!(
            world.get::<Population>(areas[0]).unwrap().population_for_player(victim),
            6,
            "the area the human protected is untouched"
        );
        assert_eq!(
            world.get::<Population>(areas[1]).unwrap().population_for_player(victim),
            2,
            "all 4 points came off the area the human picked"
        );
        assert_eq!(world.resource::<UnitLossSelectionState>().acting_player, None, "state cleared after use");
        let ResolvingCalamity::Famine(ref done) = *world.get::<ResolvingCalamity>(victim).unwrap() else { panic!() };
        assert_ne!(done.phase, FaminePhase::ComputeLosses, "phase advanced once applied");
    }

    #[test]
    fn famine_primary_loss_is_automatic_for_an_ai_victim() {
        use crate::civilization::concepts::resolve_calamities::calamities::famine::{FaminePhase, FamineState};
        use crate::civilization::concepts::resolve_calamities::resolve_calamities_systems::advance_famine;
        use crate::civilization::concepts::resolve_calamities::resolve_calamities_ui_components::{
            AwaitingHumanCalamitySelection, FamineSelectionState, UnitLossSelectionState,
        };

        let mut world = World::new();
        world.init_resource::<bevy::prelude::Messages<crate::civilization::concepts::resolve_calamities::resolve_calamities_events::CalamityResolved>>();
        world.init_resource::<FamineSelectionState>();
        world.init_resource::<UnitLossSelectionState>();

        let victim = world.spawn(Player).id(); // no IsHuman
        let mut pop = Population::new(20);
        let mut tokens = Vec::new();
        for _ in 0..6 {
            let token = world.spawn_empty().id();
            pop.add_token_to_area(victim, token);
            tokens.push(token);
        }
        let area = world.spawn((Name::new("area"), GameArea::new(1), pop, LandPassage::default())).id();
        let mut victim_areas = PlayerAreas::default();
        for token in tokens {
            victim_areas.add_token_to_area(area, token);
        }
        world.entity_mut(victim).insert(victim_areas);

        let mut state = FamineState::new();
        state.phase = FaminePhase::ComputeLosses;
        state.primary_loss = 4;

        let context = CalamityContext::new(TradeCard::Famine, victim, None);
        world.entity_mut(victim).insert((
            ActiveCalamityResolution::new(context),
            ResolvingCalamity::Famine(state),
        ));

        world.run_system_once(advance_famine).unwrap();

        assert!(world.get::<AwaitingHumanCalamitySelection>(victim).is_none(), "AI is never prompted");
        assert_eq!(world.get::<Population>(area).unwrap().population_for_player(victim), 2);
        assert_eq!(world.resource::<UnitLossSelectionState>().acting_player, None);
    }

    #[test]
    fn famine_primary_loss_skips_the_prompt_when_the_victim_loses_everything_anyway() {
        use crate::civilization::concepts::resolve_calamities::calamities::famine::{FaminePhase, FamineState};
        use crate::civilization::concepts::resolve_calamities::resolve_calamities_systems::advance_famine;
        use crate::civilization::concepts::resolve_calamities::resolve_calamities_ui_components::{
            AwaitingHumanCalamitySelection, FamineSelectionState, UnitLossSelectionState,
        };
        use crate::stupid_ai::IsHuman;

        let mut world = World::new();
        world.init_resource::<bevy::prelude::Messages<crate::civilization::concepts::resolve_calamities::resolve_calamities_events::CalamityResolved>>();
        world.init_resource::<FamineSelectionState>();
        world.init_resource::<UnitLossSelectionState>();

        let victim = world.spawn((Player, IsHuman)).id();
        let mut pop = Population::new(20);
        let mut tokens = Vec::new();
        for _ in 0..3 {
            let token = world.spawn_empty().id();
            pop.add_token_to_area(victim, token);
            tokens.push(token);
        }
        let area = world.spawn((Name::new("area"), GameArea::new(1), pop, LandPassage::default())).id();
        let mut victim_areas = PlayerAreas::default();
        for token in tokens {
            victim_areas.add_token_to_area(area, token);
        }
        world.entity_mut(victim).insert(victim_areas);

        let mut state = FamineState::new();
        state.phase = FaminePhase::ComputeLosses;
        state.primary_loss = 10; // more than the 3 tokens they own

        let context = CalamityContext::new(TradeCard::Famine, victim, None);
        world.entity_mut(victim).insert((
            ActiveCalamityResolution::new(context),
            ResolvingCalamity::Famine(state),
        ));

        world.run_system_once(advance_famine).unwrap();

        assert!(
            world.get::<AwaitingHumanCalamitySelection>(victim).is_none(),
            "nothing to choose when everything is lost regardless"
        );
        assert_eq!(world.get::<Population>(area).unwrap().population_for_player(victim), 0);
    }

    /// Rule 30.612 keeps one token per area, so a human Epidemic victim is
    /// offered `count - 1` per area -- and the city half of the budget must be
    /// spent exactly once even though the token half pauses for input.
    #[test]
    fn epidemic_primary_loss_pauses_for_a_human_without_double_spending_the_city_budget() {
        use crate::civilization::concepts::resolve_calamities::calamities::epidemic::{EpidemicPhase, EpidemicState};
        use crate::civilization::concepts::resolve_calamities::resolve_calamities_systems::advance_epidemic;
        use crate::civilization::concepts::resolve_calamities::resolve_calamities_ui_components::{
            AwaitingHumanCalamitySelection, EpidemicSelectionState, UnitLossSelectionState,
        };
        use crate::stupid_ai::IsHuman;

        let mut world = World::new();
        world.init_resource::<bevy::prelude::Messages<crate::civilization::concepts::resolve_calamities::resolve_calamities_events::CalamityResolved>>();
        world.init_resource::<EpidemicSelectionState>();
        world.init_resource::<UnitLossSelectionState>();

        let victim = world.spawn((Player, IsHuman)).id();

        let mut victim_areas = PlayerAreas::default();
        let mut areas = Vec::new();
        for n in 0..2 {
            let mut pop = Population::new(20);
            let mut tokens = Vec::new();
            for _ in 0..5 {
                let token = world.spawn_empty().id();
                pop.add_token_to_area(victim, token);
                tokens.push(token);
            }
            let area = world
                .spawn((Name::new(format!("area {n}")), GameArea::new(n), pop, LandPassage::default()))
                .id();
            for token in tokens {
                victim_areas.add_token_to_area(area, token);
            }
            areas.push(area);
        }

        let mut player_cities = PlayerCities::default();
        let city_token = world.spawn_empty().id();
        player_cities.areas_and_cities.insert(areas[0], city_token);
        world.entity_mut(victim).insert((victim_areas, player_cities));

        let mut state = EpidemicState::new();
        state.phase = EpidemicPhase::ComputeEffects;
        state.primary_loss = 6; // 4 to the city, 2 left for tokens

        let context = CalamityContext::new(TradeCard::Epidemic, victim, None);
        world.entity_mut(victim).insert((
            ActiveCalamityResolution::new(context),
            ResolvingCalamity::Epidemic(state),
        ));

        // Pass 1: spend the city budget. Pass 2: pause for the token choice.
        world.run_system_once(advance_epidemic).unwrap();
        world.run_system_once(advance_epidemic).unwrap();

        assert!(world.get::<DestroyCity>(areas[0]).is_some(), "city spent against the budget");
        assert!(
            world.get::<AwaitingHumanCalamitySelection>(victim).is_some(),
            "human should choose which tokens cover the remaining 2 points"
        );
        {
            let selection = world.resource::<UnitLossSelectionState>();
            assert_eq!(selection.total_budget, 2, "4 of the 6 points went to the city");
            assert_eq!(
                selection.total_available(),
                8,
                "30.612 leaves one token per area, so 4 of 5 are offered in each"
            );
        }

        // Extra frames while waiting must not re-destroy the city or move on.
        world.run_system_once(advance_epidemic).unwrap();
        let ResolvingCalamity::Epidemic(ref waiting) = *world.get::<ResolvingCalamity>(victim).unwrap() else { panic!() };
        assert_eq!(waiting.phase, EpidemicPhase::ApplyPrimaryLoss, "still waiting");
        assert_eq!(waiting.primary_tokens_remaining, 2, "city budget not spent twice");

        // Human puts both points on the second area.
        {
            let mut selection = world.resource_mut::<UnitLossSelectionState>();
            let second = areas[1];
            while selection.current_area().map(|(a, _, _)| a) != Some(second) {
                selection.next_area();
            }
            assert!(selection.increment_current());
            assert!(selection.increment_current());
            assert!(selection.selection_valid());
        }
        world.entity_mut(victim).remove::<AwaitingHumanCalamitySelection>();

        world.run_system_once(advance_epidemic).unwrap();

        assert_eq!(world.get::<Population>(areas[0]).unwrap().population_for_player(victim), 5);
        assert_eq!(world.get::<Population>(areas[1]).unwrap().population_for_player(victim), 3);
    }

    /// Rule 30.311: "the secondary victims choose which units to remove" --
    /// the primary victim divides the total, but each victim picks their own
    /// tokens. A human secondary victim gets the same panel the primary does.
    #[test]
    fn famine_secondary_loss_lets_a_human_secondary_victim_pick_their_own_units() {
        use crate::civilization::concepts::resolve_calamities::calamities::famine::{FaminePhase, FamineState};
        use crate::civilization::concepts::resolve_calamities::resolve_calamities_systems::advance_famine;
        use crate::civilization::concepts::resolve_calamities::resolve_calamities_ui_components::{
            AwaitingHumanCalamitySelection, FamineSelectionState, UnitLossSelectionState,
        };
        use crate::stupid_ai::IsHuman;

        let mut world = World::new();
        world.init_resource::<bevy::prelude::Messages<crate::civilization::concepts::resolve_calamities::resolve_calamities_events::CalamityResolved>>();
        world.init_resource::<FamineSelectionState>();
        world.init_resource::<UnitLossSelectionState>();

        // The primary victim is AI here so the *division* is automatic and the
        // only interaction under test is the human secondary victim's choice.
        let victim = world.spawn(Player).id();
        let human_secondary = world.spawn((Player, IsHuman)).id();

        // Two areas, 5 tokens each for the human secondary: a real choice
        // against the 3 points they are told to lose.
        let mut sec_areas = PlayerAreas::default();
        let mut areas = Vec::new();
        for n in 0..2 {
            let mut pop = Population::new(30);
            let mut tokens = Vec::new();
            for _ in 0..5 {
                let token = world.spawn_empty().id();
                pop.add_token_to_area(human_secondary, token);
                tokens.push(token);
            }
            let area = world
                .spawn((Name::new(format!("sec area {n}")), GameArea::new(n), pop, LandPassage::default()))
                .id();
            for token in tokens {
                sec_areas.add_token_to_area(area, token);
            }
            areas.push(area);
        }
        world.entity_mut(human_secondary).insert(sec_areas);
        world.entity_mut(victim).insert(PlayerAreas::default());

        let mut state = FamineState::new();
        state.phase = FaminePhase::ApplySecondaryLosses;
        state.secondary_allocations = vec![(human_secondary, 3)];

        let context = CalamityContext::new(TradeCard::Famine, victim, None);
        world.entity_mut(victim).insert((
            ActiveCalamityResolution::new(context),
            ResolvingCalamity::Famine(state),
        ));

        // First pass: the secondary victim is prompted, nothing is removed.
        world.run_system_once(advance_famine).unwrap();

        assert!(
            world.get::<AwaitingHumanCalamitySelection>(human_secondary).is_some(),
            "the secondary victim, not the primary, chooses their own units"
        );
        assert!(world.get::<AwaitingHumanCalamitySelection>(victim).is_none());
        assert_eq!(world.get::<Population>(areas[0]).unwrap().population_for_player(human_secondary), 5);
        assert_eq!(world.get::<Population>(areas[1]).unwrap().population_for_player(human_secondary), 5);
        assert_eq!(world.resource::<UnitLossSelectionState>().acting_player, Some(human_secondary));

        {
            let mut selection = world.resource_mut::<UnitLossSelectionState>();
            let second = areas[1];
            while selection.current_area().map(|(a, _, _)| a) != Some(second) {
                selection.next_area();
            }
            for _ in 0..3 {
                assert!(selection.increment_current());
            }
        }
        world.entity_mut(human_secondary).remove::<AwaitingHumanCalamitySelection>();

        world.run_system_once(advance_famine).unwrap();

        assert_eq!(world.get::<Population>(areas[0]).unwrap().population_for_player(human_secondary), 5);
        assert_eq!(world.get::<Population>(areas[1]).unwrap().population_for_player(human_secondary), 2);
        let ResolvingCalamity::Famine(ref done) = *world.get::<ResolvingCalamity>(victim).unwrap() else { panic!() };
        assert_eq!(done.phase, FaminePhase::Complete);
        assert!(done.secondary_allocations.is_empty(), "work list drained");
    }

    /// A pause for one victim must not re-charge the victims already settled.
    #[test]
    fn famine_secondary_losses_are_applied_once_each_across_a_pause() {
        use crate::civilization::concepts::resolve_calamities::calamities::famine::{FaminePhase, FamineState};
        use crate::civilization::concepts::resolve_calamities::resolve_calamities_systems::advance_famine;
        use crate::civilization::concepts::resolve_calamities::resolve_calamities_ui_components::{
            AwaitingHumanCalamitySelection, FamineSelectionState, UnitLossSelectionState,
        };
        use crate::stupid_ai::IsHuman;

        let mut world = World::new();
        world.init_resource::<bevy::prelude::Messages<crate::civilization::concepts::resolve_calamities::resolve_calamities_events::CalamityResolved>>();
        world.init_resource::<FamineSelectionState>();
        world.init_resource::<UnitLossSelectionState>();

        let victim = world.spawn((Player, PlayerAreas::default())).id();
        let ai_secondary = world.spawn(Player).id();
        let human_secondary = world.spawn((Player, IsHuman)).id();

        let mut pop = Population::new(60);
        let mut ai_areas = PlayerAreas::default();
        let mut human_areas = PlayerAreas::default();
        for _ in 0..6 {
            let token = world.spawn_empty().id();
            pop.add_token_to_area(ai_secondary, token);
        }
        for _ in 0..6 {
            let token = world.spawn_empty().id();
            pop.add_token_to_area(human_secondary, token);
        }
        let area = world.spawn((Name::new("shared"), GameArea::new(1), pop, LandPassage::default())).id();
        for token in world.get::<Population>(area).unwrap().tokens_for_player(&ai_secondary).unwrap() {
            ai_areas.add_token_to_area(area, *token);
        }
        for token in world.get::<Population>(area).unwrap().tokens_for_player(&human_secondary).unwrap() {
            human_areas.add_token_to_area(area, *token);
        }
        // Give the human a second area so they actually have a choice to make.
        let mut second_pop = Population::new(30);
        for _ in 0..4 {
            let token = world.spawn_empty().id();
            second_pop.add_token_to_area(human_secondary, token);
        }
        let second_area = world.spawn((Name::new("second"), GameArea::new(2), second_pop, LandPassage::default())).id();
        for token in world.get::<Population>(second_area).unwrap().tokens_for_player(&human_secondary).unwrap() {
            human_areas.add_token_to_area(second_area, *token);
        }
        world.entity_mut(ai_secondary).insert(ai_areas);
        world.entity_mut(human_secondary).insert(human_areas);

        let mut state = FamineState::new();
        state.phase = FaminePhase::ApplySecondaryLosses;
        state.secondary_allocations = vec![(ai_secondary, 2), (human_secondary, 2)];

        let context = CalamityContext::new(TradeCard::Famine, victim, None);
        world.entity_mut(victim).insert((
            ActiveCalamityResolution::new(context),
            ResolvingCalamity::Famine(state),
        ));

        world.run_system_once(advance_famine).unwrap();
        // The AI victim is settled immediately; the human is paused.
        assert_eq!(world.get::<Population>(area).unwrap().population_for_player(ai_secondary), 4);
        assert!(world.get::<AwaitingHumanCalamitySelection>(human_secondary).is_some());

        // Idle frames while waiting must not charge the AI victim again.
        world.run_system_once(advance_famine).unwrap();
        world.run_system_once(advance_famine).unwrap();
        assert_eq!(
            world.get::<Population>(area).unwrap().population_for_player(ai_secondary),
            4,
            "settled victims are dropped from the work list"
        );

        {
            let mut selection = world.resource_mut::<UnitLossSelectionState>();
            while selection.current_area().map(|(a, _, _)| a) != Some(second_area) {
                selection.next_area();
            }
            for _ in 0..2 {
                assert!(selection.increment_current());
            }
        }
        world.entity_mut(human_secondary).remove::<AwaitingHumanCalamitySelection>();
        world.run_system_once(advance_famine).unwrap();

        assert_eq!(world.get::<Population>(area).unwrap().population_for_player(ai_secondary), 4);
        assert_eq!(world.get::<Population>(area).unwrap().population_for_player(human_secondary), 6);
        assert_eq!(world.get::<Population>(second_area).unwrap().population_for_player(human_secondary), 2);
    }

    // ── Rule 30.512: human primary victim allocation UI wiring ─────────────

    use crate::civilization::concepts::resolve_calamities::resolve_calamities_ui_components::{
        AwaitingHumanCalamitySelection, FloodSelectionState,
    };
    use crate::stupid_ai::IsHuman;

    #[test]
    fn flood_secondary_loss_pauses_for_a_human_primary_victim_and_applies_their_choice() {
        let mut world = World::new();
        world.init_resource::<bevy::prelude::Messages<crate::civilization::concepts::resolve_calamities::resolve_calamities_events::CalamityResolved>>();
        world.init_resource::<FloodSelectionState>();

        let victim = world.spawn((PlayerCities::default(), IsHuman)).id();
        let sec_a = world.spawn_empty().id();
        let sec_b = world.spawn_empty().id();

        // Combined secondary availability (20 + 20 = 40) exceeds the 10-point
        // budget, so this must trigger the interactive choice, not the
        // automatic "everyone loses everything" path.
        let mut fp_pop = Population::new(60);
        for _ in 0..20 {
            fp_pop.add_token_to_area(sec_a, world.spawn_empty().id());
        }
        for _ in 0..20 {
            fp_pop.add_token_to_area(sec_b, world.spawn_empty().id());
        }
        let fp_area = world
            .spawn((Name::new("flood plain"), GameArea::new(1), FloodPlain, fp_pop, LandPassage::default()))
            .id();

        let mut state = FloodState::new();
        state.phase = FloodPhase::ApplySecondaryLoss;
        state.flood_plain_area = Some(fp_area);

        let context = CalamityContext::new(TradeCard::Flood, victim, None);
        world.entity_mut(victim).insert((
            ActiveCalamityResolution::new(context),
            ResolvingCalamity::Flood(state),
        ));

        // First pass: should pause, not resolve.
        world.run_system_once(advance_flood).unwrap();
        assert!(world.get::<AwaitingHumanCalamitySelection>(victim).is_some(), "human victim should be paused for input");
        let selection = world.resource::<FloodSelectionState>();
        assert_eq!(selection.acting_player, Some(victim));
        assert_eq!(selection.total_budget, 10);
        let ResolvingCalamity::Flood(ref state_after) = *world.get::<ResolvingCalamity>(victim).unwrap() else { panic!() };
        assert_eq!(state_after.phase, FloodPhase::ApplySecondaryLoss, "must not advance while waiting for the human");

        // Simulate the human choosing to take all 10 from sec_a, nothing from sec_b
        // (as the actual UI buttons would produce), then confirming.
        {
            let mut selection = world.resource_mut::<FloodSelectionState>();
            for _ in 0..10 {
                assert!(selection.increment_current());
            }
        }
        world.entity_mut(victim).remove::<AwaitingHumanCalamitySelection>();

        // Second pass: should read back the choice and resolve.
        world.run_system_once(advance_flood).unwrap();

        let fp_pop_after = world.get::<Population>(fp_area).unwrap();
        assert_eq!(fp_pop_after.population_for_player(sec_a), 10, "all 10 points came from sec_a, per the human's choice");
        assert_eq!(fp_pop_after.population_for_player(sec_b), 20, "sec_b untouched");
        let ResolvingCalamity::Flood(ref state_final) = *world.get::<ResolvingCalamity>(victim).unwrap() else { panic!() };
        assert_eq!(state_final.phase, FloodPhase::Complete);
        assert_eq!(world.resource::<FloodSelectionState>().acting_player, None, "selection state cleared after use");
    }

    #[test]
    fn flood_secondary_loss_skips_the_ui_when_everyone_loses_everything_automatically() {
        let mut world = World::new();
        world.init_resource::<bevy::prelude::Messages<crate::civilization::concepts::resolve_calamities::resolve_calamities_events::CalamityResolved>>();
        world.init_resource::<FloodSelectionState>();

        let victim = world.spawn((PlayerCities::default(), IsHuman)).id();
        let sec_a = world.spawn_empty().id();
        let sec_b = world.spawn_empty().id();

        // Combined availability (4 + 3 = 7) is at or below the 10-point budget --
        // rule 30.512 says everyone automatically loses everything, no choice to make.
        let mut fp_pop = Population::new(20);
        for _ in 0..4 {
            fp_pop.add_token_to_area(sec_a, world.spawn_empty().id());
        }
        for _ in 0..3 {
            fp_pop.add_token_to_area(sec_b, world.spawn_empty().id());
        }
        let fp_area = world
            .spawn((Name::new("flood plain"), GameArea::new(1), FloodPlain, fp_pop, LandPassage::default()))
            .id();

        let mut state = FloodState::new();
        state.phase = FloodPhase::ApplySecondaryLoss;
        state.flood_plain_area = Some(fp_area);

        let context = CalamityContext::new(TradeCard::Flood, victim, None);
        world.entity_mut(victim).insert((
            ActiveCalamityResolution::new(context),
            ResolvingCalamity::Flood(state),
        ));

        world.run_system_once(advance_flood).unwrap();

        assert!(world.get::<AwaitingHumanCalamitySelection>(victim).is_none(), "no UI should be shown -- there's nothing to choose");
        let fp_pop_after = world.get::<Population>(fp_area).unwrap();
        assert_eq!(fp_pop_after.population_for_player(sec_a), 0);
        assert_eq!(fp_pop_after.population_for_player(sec_b), 0);
        let ResolvingCalamity::Flood(ref state_final) = *world.get::<ResolvingCalamity>(victim).unwrap() else { panic!() };
        assert_eq!(state_final.phase, FloodPhase::Complete);
    }

    // ========================================================================
    // Rule 30.612: Epidemic city cap (ECS-level, via advance_epidemic)
    // ========================================================================
    //
    // "Cities eliminated by Epidemic are replaced by at least one token; thus
    // cities account for a maximum of 4 unit points when calculating Epidemic
    // losses." A city in an affected area absorbs up to 4 points of the loss
    // budget (never more, regardless of Engineering -- unlike Flood/Volcano,
    // rule 30.612 doesn't mention Engineering at all), then whatever remains
    // is removed from tokens, still respecting the existing leave-one-per-area
    // rule.

    use crate::civilization::concepts::resolve_calamities::calamities::epidemic::{EpidemicPhase, EpidemicState};
    use crate::civilization::concepts::resolve_calamities::resolve_calamities_systems::{advance_epidemic, DestroyCity};

    #[test]
    fn epidemic_primary_loss_spends_up_to_4_points_on_an_owned_city_then_the_rest_on_tokens() {
        let mut world = World::new();
        world.init_resource::<bevy::prelude::Messages<crate::civilization::concepts::resolve_calamities::resolve_calamities_events::CalamityResolved>>();
        world.init_resource::<crate::civilization::concepts::resolve_calamities::resolve_calamities_ui_components::EpidemicSelectionState>();
        world.init_resource::<crate::civilization::concepts::resolve_calamities::resolve_calamities_ui_components::UnitLossSelectionState>();

        let victim = world.spawn_empty().id();
        let mut pop = Population::new(20);
        let mut token_ids = Vec::new();
        for _ in 0..10 {
            let token = world.spawn_empty().id();
            pop.add_token_to_area(victim, token);
            token_ids.push(token);
        }
        let city_area = world.spawn((Name::new("city area"), GameArea::new(1), pop, LandPassage::default())).id();
        let mut player_areas = PlayerAreas::default();
        for token in token_ids {
            player_areas.add_token_to_area(city_area, token);
        }
        let mut player_cities = PlayerCities::default();
        let victim_city_token = world.spawn_empty().id();
        player_cities.areas_and_cities.insert(city_area, victim_city_token);
        world.entity_mut(victim).insert((player_areas, player_cities));

        let mut state = EpidemicState::new();
        state.phase = EpidemicPhase::ComputeEffects;
        state.primary_loss = 6; // 4 for the city + 2 leftover for tokens

        let context = CalamityContext::new(TradeCard::Epidemic, victim, None);
        world.entity_mut(victim).insert((
            ActiveCalamityResolution::new(context),
            ResolvingCalamity::Epidemic(state),
        ));

        // Two passes: ComputeEffects spends the budget on cities, ApplyPrimaryLoss
        // takes the remainder off tokens (split so the token half can pause for
        // a human's choice without re-destroying the city).
        world.run_system_once(advance_epidemic).unwrap();
        world.run_system_once(advance_epidemic).unwrap();

        // The city was marked for destruction (spending 4 of the 6-point budget).
        assert!(world.get::<DestroyCity>(city_area).is_some());
        // The remaining 2 points came off tokens in that same area: 10 -> 8.
        let pop_after = world.get::<Population>(city_area).unwrap();
        assert_eq!(pop_after.population_for_player(victim), 8);
    }

    #[test]
    fn epidemic_primary_loss_under_4_points_still_spends_only_on_the_city_partially() {
        let mut world = World::new();
        world.init_resource::<bevy::prelude::Messages<crate::civilization::concepts::resolve_calamities::resolve_calamities_events::CalamityResolved>>();
        world.init_resource::<crate::civilization::concepts::resolve_calamities::resolve_calamities_ui_components::EpidemicSelectionState>();
        world.init_resource::<crate::civilization::concepts::resolve_calamities::resolve_calamities_ui_components::UnitLossSelectionState>();

        let victim = world.spawn_empty().id();
        let mut pop = Population::new(20);
        let mut token_ids = Vec::new();
        for _ in 0..5 {
            let token = world.spawn_empty().id();
            pop.add_token_to_area(victim, token);
            token_ids.push(token);
        }
        let city_area = world.spawn((Name::new("city area"), GameArea::new(1), pop, LandPassage::default())).id();
        let mut player_areas = PlayerAreas::default();
        for token in token_ids {
            player_areas.add_token_to_area(city_area, token);
        }
        let mut player_cities = PlayerCities::default();
        let victim_city_token = world.spawn_empty().id();
        player_cities.areas_and_cities.insert(city_area, victim_city_token);
        world.entity_mut(victim).insert((player_areas, player_cities));

        let mut state = EpidemicState::new();
        state.phase = EpidemicPhase::ComputeEffects;
        state.primary_loss = 2; // less than the 4-point city cost

        let context = CalamityContext::new(TradeCard::Epidemic, victim, None);
        world.entity_mut(victim).insert((
            ActiveCalamityResolution::new(context),
            ResolvingCalamity::Epidemic(state),
        ));

        world.run_system_once(advance_epidemic).unwrap();

        // The whole 2-point budget went to the city; no tokens were touched.
        assert!(world.get::<DestroyCity>(city_area).is_some());
        let pop_after = world.get::<Population>(city_area).unwrap();
        assert_eq!(pop_after.population_for_player(victim), 5);
    }

    #[test]
    fn epidemic_does_not_touch_a_city_the_player_does_not_own() {
        let mut world = World::new();
        world.init_resource::<bevy::prelude::Messages<crate::civilization::concepts::resolve_calamities::resolve_calamities_events::CalamityResolved>>();
        world.init_resource::<crate::civilization::concepts::resolve_calamities::resolve_calamities_ui_components::EpidemicSelectionState>();
        world.init_resource::<crate::civilization::concepts::resolve_calamities::resolve_calamities_ui_components::UnitLossSelectionState>();

        // Victim has no cities at all -- only an area with tokens.
        let victim = world.spawn_empty().id();
        let mut pop = Population::new(20);
        let mut token_ids = Vec::new();
        for _ in 0..10 {
            let token = world.spawn_empty().id();
            pop.add_token_to_area(victim, token);
            token_ids.push(token);
        }
        let area = world.spawn((Name::new("area"), GameArea::new(1), pop, LandPassage::default())).id();
        let mut player_areas = PlayerAreas::default();
        for token in token_ids {
            player_areas.add_token_to_area(area, token);
        }
        world.entity_mut(victim).insert((player_areas, PlayerCities::default()));

        let mut state = EpidemicState::new();
        state.phase = EpidemicPhase::ComputeEffects;
        state.primary_loss = 6;

        let context = CalamityContext::new(TradeCard::Epidemic, victim, None);
        world.entity_mut(victim).insert((
            ActiveCalamityResolution::new(context),
            ResolvingCalamity::Epidemic(state),
        ));

        world.run_system_once(advance_epidemic).unwrap();
        world.run_system_once(advance_epidemic).unwrap();

        // No city to destroy; the whole budget came off tokens instead: 10 -> 4.
        assert!(world.get::<DestroyCity>(area).is_none());
        let pop_after = world.get::<Population>(area).unwrap();
        assert_eq!(pop_after.population_for_player(victim), 4);
    }

    // ========================================================================
    // Rule 30.611: Epidemic secondary-victim allocation (ECS-level, via advance_epidemic)
    // ========================================================================
    //
    // Mirrors the Flood rule-30.512 coverage above: a human primary victim
    // gets an interactive choice for how the 25-point secondary loss is
    // divided (max 10 per secondary victim, 5 if that victim holds
    // Medicine, rule 30.613); the UI is skipped entirely when there's
    // nothing to choose (combined secondary caps <=25).

    use crate::civilization::concepts::civ_cards::PlayerCivilizationCards;
    use crate::civilization::concepts::resolve_calamities::resolve_calamities_ui_components::EpidemicSelectionState;
    use crate::civilization::CivCardName;

    #[test]
    fn epidemic_secondary_loss_pauses_for_a_human_primary_victim_and_applies_their_choice() {
        let mut world = World::new();
        world.init_resource::<bevy::prelude::Messages<crate::civilization::concepts::resolve_calamities::resolve_calamities_events::CalamityResolved>>();
        world.init_resource::<EpidemicSelectionState>();
        world.init_resource::<crate::civilization::concepts::resolve_calamities::resolve_calamities_ui_components::UnitLossSelectionState>();

        // Four secondary victims: one holds Medicine (cap 5, rule 30.613),
        // the other three don't (cap 10 each). Combined caps 5+10+10+10=35
        // exceed the 25-point budget, so this must trigger the interactive
        // choice, not the automatic "everyone loses everything" path.
        let shared_area = world
            .spawn((Name::new("shared area"), GameArea::new(1), Population::new(200), LandPassage::default()))
            .id();

        let mut secondaries = Vec::new();
        for i in 0..4 {
            let mut entity = world.spawn((Player, PlayerCities::default()));
            if i == 1 {
                let mut cards = PlayerCivilizationCards::default();
                cards.add_card(CivCardName::Medicine);
                entity.insert(cards);
            }
            let sec = entity.id();
            let mut areas = PlayerAreas::default();
            for _ in 0..12 {
                let token = world.spawn_empty().id();
                world.get_mut::<Population>(shared_area).unwrap().add_token_to_area(sec, token);
                areas.add_token_to_area(shared_area, token);
            }
            world.entity_mut(sec).insert(areas);
            secondaries.push(sec);
        }

        let victim = world.spawn((IsHuman, PlayerCities::default())).id();
        let mut victim_areas = PlayerAreas::default();
        victim_areas.add_token_to_area(shared_area, world.spawn_empty().id());
        world.entity_mut(victim).insert(victim_areas);

        let mut state = EpidemicState::new();
        state.phase = EpidemicPhase::ApplySecondaryLosses;

        let context = CalamityContext::new(TradeCard::Epidemic, victim, None);
        world.entity_mut(victim).insert((
            ActiveCalamityResolution::new(context),
            ResolvingCalamity::Epidemic(state),
        ));

        // First pass: should pause, not resolve.
        world.run_system_once(advance_epidemic).unwrap();
        assert!(world.get::<AwaitingHumanCalamitySelection>(victim).is_some(), "human victim should be paused for input");
        let selection = world.resource::<EpidemicSelectionState>();
        assert_eq!(selection.acting_player, Some(victim));
        assert_eq!(selection.total_budget, 25);
        assert_eq!(selection.victims.len(), 4);
        // The Medicine holder's cap is 5, everyone else's is 10.
        let medicine_victim_cap = selection
            .victims
            .iter()
            .find(|&&(e, _, _)| e == secondaries[1])
            .map(|&(_, cap, _)| cap);
        assert_eq!(medicine_victim_cap, Some(5));
        let ResolvingCalamity::Epidemic(ref state_after) = *world.get::<ResolvingCalamity>(victim).unwrap() else { panic!() };
        assert_eq!(state_after.phase, EpidemicPhase::ApplySecondaryLosses, "must not advance while waiting for the human");

        // Simulate the human choosing 10 from the first, 5 (its cap) from
        // the Medicine holder, 10 from the third, 0 from the fourth. Query
        // iteration order doesn't match spawn order once entities have
        // different archetypes (the Medicine holder has an extra
        // component), so look each victim's position up by entity rather
        // than assuming an order.
        {
            let mut selection = world.resource_mut::<EpidemicSelectionState>();
            let idx0 = selection.victims.iter().position(|&(e, _, _)| e == secondaries[0]).unwrap();
            let idx1 = selection.victims.iter().position(|&(e, _, _)| e == secondaries[1]).unwrap();
            let idx2 = selection.victims.iter().position(|&(e, _, _)| e == secondaries[2]).unwrap();

            selection.current_victim_index = idx0;
            for _ in 0..10 {
                assert!(selection.increment_current());
            }
            selection.current_victim_index = idx1;
            for _ in 0..5 {
                assert!(selection.increment_current());
            }
            selection.current_victim_index = idx2;
            for _ in 0..10 {
                assert!(selection.increment_current());
            }
        }
        world.entity_mut(victim).remove::<AwaitingHumanCalamitySelection>();

        // Second pass: reads back the choice, applies it, and (unlike Famine,
        // which has a separate ApplySecondaryLosses phase after its
        // SelectSecondaryVictims) reaches Complete in this same pass, since
        // Epidemic reuses ApplySecondaryLosses itself as the pause/resume
        // phase (mirroring Flood's ApplySecondaryLoss).
        world.run_system_once(advance_epidemic).unwrap();

        let pop_after = world.get::<Population>(shared_area).unwrap();
        assert_eq!(pop_after.population_for_player(secondaries[0]), 2, "12 - 10");
        assert_eq!(pop_after.population_for_player(secondaries[1]), 7, "12 - 5 (Medicine cap)");
        assert_eq!(pop_after.population_for_player(secondaries[2]), 2, "12 - 10");
        assert_eq!(pop_after.population_for_player(secondaries[3]), 12, "0 chosen");
        let ResolvingCalamity::Epidemic(ref state_final) = *world.get::<ResolvingCalamity>(victim).unwrap() else { panic!() };
        assert_eq!(state_final.phase, EpidemicPhase::Complete);
        assert_eq!(world.resource::<EpidemicSelectionState>().acting_player, None, "selection state cleared after use");
    }

    #[test]
    fn epidemic_secondary_loss_skips_the_ui_when_everyone_loses_everything_automatically() {
        let mut world = World::new();
        world.init_resource::<bevy::prelude::Messages<crate::civilization::concepts::resolve_calamities::resolve_calamities_events::CalamityResolved>>();
        world.init_resource::<EpidemicSelectionState>();
        world.init_resource::<crate::civilization::concepts::resolve_calamities::resolve_calamities_ui_components::UnitLossSelectionState>();

        // Two secondary victims, neither holds Medicine (cap 10 each):
        // 10+10=20 <= the 25-point budget -- no choice to make, everyone
        // automatically loses everything.
        let shared_area = world
            .spawn((Name::new("shared area"), GameArea::new(1), Population::new(60), LandPassage::default()))
            .id();

        let sec_a = world.spawn((Player, PlayerCities::default())).id();
        let sec_b = world.spawn((Player, PlayerCities::default())).id();
        let mut a_areas = PlayerAreas::default();
        let mut b_areas = PlayerAreas::default();
        for _ in 0..8 {
            let token = world.spawn_empty().id();
            world.get_mut::<Population>(shared_area).unwrap().add_token_to_area(sec_a, token);
            a_areas.add_token_to_area(shared_area, token);
        }
        for _ in 0..8 {
            let token = world.spawn_empty().id();
            world.get_mut::<Population>(shared_area).unwrap().add_token_to_area(sec_b, token);
            b_areas.add_token_to_area(shared_area, token);
        }
        world.entity_mut(sec_a).insert(a_areas);
        world.entity_mut(sec_b).insert(b_areas);

        let victim = world.spawn((IsHuman, PlayerCities::default())).id();
        let mut victim_areas = PlayerAreas::default();
        victim_areas.add_token_to_area(shared_area, world.spawn_empty().id());
        world.entity_mut(victim).insert(victim_areas);

        let mut state = EpidemicState::new();
        state.phase = EpidemicPhase::ApplySecondaryLosses;

        let context = CalamityContext::new(TradeCard::Epidemic, victim, None);
        world.entity_mut(victim).insert((
            ActiveCalamityResolution::new(context),
            ResolvingCalamity::Epidemic(state),
        ));

        world.run_system_once(advance_epidemic).unwrap();

        assert!(world.get::<AwaitingHumanCalamitySelection>(victim).is_none(), "no UI should be shown -- there's nothing to choose");
        let pop_after = world.get::<Population>(shared_area).unwrap();
        // Rule 30.612: at least one token must be left in each affected area,
        // so even "losing everything" leaves 1 of the original 8 behind.
        assert_eq!(pop_after.population_for_player(sec_a), 1);
        assert_eq!(pop_after.population_for_player(sec_b), 1);
        let ResolvingCalamity::Epidemic(ref state_final) = *world.get::<ResolvingCalamity>(victim).unwrap() else { panic!() };
        assert_eq!(state_final.phase, EpidemicPhase::Complete);
    }

    // ========================================================================
    // Rule 30.311: Famine secondary-victim allocation (ECS-level, via advance_famine)
    // ========================================================================
    //
    // Mirrors the Flood rule-30.512 coverage above: a human primary victim
    // gets an interactive choice for how the 20-point secondary loss is
    // divided (max 8 per secondary victim); the UI is skipped entirely when
    // there's nothing to choose (combined secondary availability <=20).

    use crate::civilization::concepts::resolve_calamities::calamities::famine::{FaminePhase, FamineState};
    use crate::civilization::concepts::resolve_calamities::resolve_calamities_systems::advance_famine;
    use crate::civilization::concepts::resolve_calamities::resolve_calamities_ui_components::FamineSelectionState;

    #[test]
    fn famine_secondary_loss_pauses_for_a_human_primary_victim_and_applies_their_choice() {
        let mut world = World::new();
        world.init_resource::<bevy::prelude::Messages<crate::civilization::concepts::resolve_calamities::resolve_calamities_events::CalamityResolved>>();
        world.init_resource::<FamineSelectionState>();
        world.init_resource::<crate::civilization::concepts::resolve_calamities::resolve_calamities_ui_components::UnitLossSelectionState>();

        // Three secondary victims, each capped at 8 (rule 30.311) even though
        // they each physically have 10 tokens: 8+8+8=24 > 20-point budget, so
        // this must trigger the interactive choice, not the automatic
        // "everyone loses everything" path.
        let shared_area = world
            .spawn((Name::new("shared area"), GameArea::new(1), Population::new(100), LandPassage::default()))
            .id();

        let mut secondaries = Vec::new();
        for _ in 0..3 {
            let sec = world.spawn(Player).id();
            let mut areas = PlayerAreas::default();
            for _ in 0..10 {
                let token = world.spawn_empty().id();
                world.get_mut::<Population>(shared_area).unwrap().add_token_to_area(sec, token);
                areas.add_token_to_area(shared_area, token);
            }
            world.entity_mut(sec).insert(areas);
            secondaries.push(sec);
        }

        let victim = world.spawn(IsHuman).id();
        let mut victim_areas = PlayerAreas::default();
        victim_areas.add_token_to_area(shared_area, world.spawn_empty().id());
        world.entity_mut(victim).insert(victim_areas);

        let mut state = FamineState::new();
        state.phase = FaminePhase::SelectSecondaryVictims;

        let context = CalamityContext::new(TradeCard::Famine, victim, None);
        world.entity_mut(victim).insert((
            ActiveCalamityResolution::new(context),
            ResolvingCalamity::Famine(state),
        ));

        // First pass: should pause, not resolve.
        world.run_system_once(advance_famine).unwrap();
        assert!(world.get::<AwaitingHumanCalamitySelection>(victim).is_some(), "human victim should be paused for input");
        let selection = world.resource::<FamineSelectionState>();
        assert_eq!(selection.acting_player, Some(victim));
        assert_eq!(selection.total_budget, 20);
        assert_eq!(selection.victims.len(), 3);
        let ResolvingCalamity::Famine(ref state_after) = *world.get::<ResolvingCalamity>(victim).unwrap() else { panic!() };
        assert_eq!(state_after.phase, FaminePhase::SelectSecondaryVictims, "must not advance while waiting for the human");

        // Simulate the human choosing 8 from the first secondary, 8 from the
        // second, 4 from the third (as the actual UI buttons would produce).
        {
            let mut selection = world.resource_mut::<FamineSelectionState>();
            for _ in 0..8 {
                assert!(selection.increment_current());
            }
            selection.next_victim();
            for _ in 0..8 {
                assert!(selection.increment_current());
            }
            selection.next_victim();
            for _ in 0..4 {
                assert!(selection.increment_current());
            }
        }
        world.entity_mut(victim).remove::<AwaitingHumanCalamitySelection>();

        // Second pass: reads back the choice, still needs one more pass to apply it.
        world.run_system_once(advance_famine).unwrap();
        world.run_system_once(advance_famine).unwrap();

        let pop_after = world.get::<Population>(shared_area).unwrap();
        assert_eq!(pop_after.population_for_player(secondaries[0]), 2, "10 - 8");
        assert_eq!(pop_after.population_for_player(secondaries[1]), 2, "10 - 8");
        assert_eq!(pop_after.population_for_player(secondaries[2]), 6, "10 - 4");
        let ResolvingCalamity::Famine(ref state_final) = *world.get::<ResolvingCalamity>(victim).unwrap() else { panic!() };
        assert_eq!(state_final.phase, FaminePhase::Complete);
        assert_eq!(world.resource::<FamineSelectionState>().acting_player, None, "selection state cleared after use");
    }

    #[test]
    fn famine_secondary_loss_skips_the_ui_when_everyone_loses_everything_automatically() {
        let mut world = World::new();
        world.init_resource::<bevy::prelude::Messages<crate::civilization::concepts::resolve_calamities::resolve_calamities_events::CalamityResolved>>();
        world.init_resource::<FamineSelectionState>();
        world.init_resource::<crate::civilization::concepts::resolve_calamities::resolve_calamities_ui_components::UnitLossSelectionState>();

        // Two secondary victims, each capped at 8: 8+8=16 <= the 20-point
        // budget -- no choice to make, everyone automatically loses everything.
        let shared_area = world
            .spawn((Name::new("shared area"), GameArea::new(1), Population::new(60), LandPassage::default()))
            .id();

        let sec_a = world.spawn(Player).id();
        let sec_b = world.spawn(Player).id();
        let mut a_areas = PlayerAreas::default();
        let mut b_areas = PlayerAreas::default();
        for _ in 0..8 {
            let token = world.spawn_empty().id();
            world.get_mut::<Population>(shared_area).unwrap().add_token_to_area(sec_a, token);
            a_areas.add_token_to_area(shared_area, token);
        }
        for _ in 0..8 {
            let token = world.spawn_empty().id();
            world.get_mut::<Population>(shared_area).unwrap().add_token_to_area(sec_b, token);
            b_areas.add_token_to_area(shared_area, token);
        }
        world.entity_mut(sec_a).insert(a_areas);
        world.entity_mut(sec_b).insert(b_areas);

        let victim = world.spawn(IsHuman).id();
        let mut victim_areas = PlayerAreas::default();
        victim_areas.add_token_to_area(shared_area, world.spawn_empty().id());
        world.entity_mut(victim).insert(victim_areas);

        let mut state = FamineState::new();
        state.phase = FaminePhase::SelectSecondaryVictims;

        let context = CalamityContext::new(TradeCard::Famine, victim, None);
        world.entity_mut(victim).insert((
            ActiveCalamityResolution::new(context),
            ResolvingCalamity::Famine(state),
        ));

        world.run_system_once(advance_famine).unwrap();
        world.run_system_once(advance_famine).unwrap();

        assert!(world.get::<AwaitingHumanCalamitySelection>(victim).is_none(), "no UI should be shown -- there's nothing to choose");
        let pop_after = world.get::<Population>(shared_area).unwrap();
        assert_eq!(pop_after.population_for_player(sec_a), 0);
        assert_eq!(pop_after.population_for_player(sec_b), 0);
        let ResolvingCalamity::Famine(ref state_final) = *world.get::<ResolvingCalamity>(victim).unwrap() else { panic!() };
        assert_eq!(state_final.phase, FaminePhase::Complete);
    }

    // ── Rule 30.52: Barbarian Hordes -- real conflict + movement cascade ────
    //
    // Unlike the other calamities tested above, these exercise real conflict
    // resolution (on_add_unresolved_conflict/on_add_unresolved_city_conflict,
    // the same observers the Conflict phase itself uses), not just this
    // module's own state math -- that's the whole point of the rewrite (see
    // BarbarianHordesState's doc comment).

    use crate::civilization::concepts::conflict::{
        on_add_unresolved_city_conflict, on_add_unresolved_conflict, ConflictCounterResource,
        UnresolvedCityConflict, UnresolvedConflict,
    };
    use crate::civilization::concepts::resolve_calamities::calamities::barbarian_hordes::BarbarianHordesState;
    use crate::civilization::concepts::resolve_calamities::resolve_calamities_systems::advance_barbarian_hordes;
    use crate::civilization::{CameraFocusQueue, GameFaction};
    use crate::GameActivity;

    /// Common resource wiring the conflict observers need to run at all,
    /// independent of the actual Conflict-phase state machinery (this test
    /// never runs that phase -- it only needs the observers' ResMut params
    /// to resolve).
    fn barbarian_test_world() -> World {
        let mut world = World::new();
        world.init_resource::<Messages<crate::civilization::concepts::resolve_calamities::resolve_calamities_events::CalamityResolved>>();
        world.init_resource::<ConflictCounterResource>();
        world.init_resource::<NextState<GameActivity>>();
        world.init_resource::<CameraFocusQueue>();
        world.add_observer(on_add_unresolved_conflict);
        world.add_observer(on_add_unresolved_city_conflict);
        world
    }

    /// Ticks `advance_barbarian_hordes` until `victim`'s `ResolvingCalamity`
    /// is gone (resolution complete), or panics if that doesn't happen
    /// within a generous bound -- a real safety net, not a rules concern
    /// (MAX_CASCADE_ITERATIONS already bounds the cascade itself; this just
    /// bounds the test).
    fn run_barbarian_hordes_to_completion(world: &mut World, victim: Entity) {
        for _ in 0..40 {
            if world.get::<ResolvingCalamity>(victim).is_none() {
                return;
            }
            world.run_system_once(advance_barbarian_hordes).unwrap();
            world.flush();
        }
        let stuck_phase = world.get::<ResolvingCalamity>(victim).map(|rc| format!("{rc:?}"));
        panic!("Barbarian Hordes resolution did not converge within 40 ticks; stuck at: {stuck_phase:?}");
    }

    fn start_barbarian_hordes(world: &mut World, victim: Entity) {
        let context = CalamityContext::new(TradeCard::BarbarianHordes, victim, None);
        world.entity_mut(victim).insert((
            ActiveCalamityResolution::new(context),
            ResolvingCalamity::BarbarianHordes(BarbarianHordesState::new()),
        ));
    }

    #[test]
    fn crete_is_immune_and_never_places_any_tokens() {
        let mut world = barbarian_test_world();
        let victim = world.spawn(Faction { faction: GameFaction::Crete }).id();
        start_barbarian_hordes(&mut world, victim);

        run_barbarian_hordes_to_completion(&mut world, victim);

        // Resolved with no landing area / owner entity ever set -- proves it
        // short-circuited at FindLandingArea (rule 30.527) rather than
        // placing and then somehow reversing.
        assert_eq!(world.resource::<Messages<crate::civilization::concepts::resolve_calamities::resolve_calamities_events::CalamityResolved>>().len(), 1);
    }

    #[test]
    fn real_conflict_resolution_wipes_the_weaker_side_not_a_flat_subtraction() {
        // A prior version of this module removed a flat 15 (or 10 with
        // Military) unit points regardless of what was actually on the
        // board. Two victim tokens against fifteen real Barbarian tokens
        // must be wiped out by the SAME conflict math the rest of the game
        // uses (handle_unequal_lengths: repeatedly pops the weakest
        // remaining player), not by a point-budget subtraction.
        let mut world = barbarian_test_world();
        let victim = world.spawn(Faction { faction: GameFaction::Egypt }).id();

        let mut pop = Population::new(4);
        let victim_tokens: Vec<Entity> = (0..2).map(|_| world.spawn_empty().id()).collect();
        for &t in &victim_tokens {
            pop.add_token_to_area(victim, t);
        }
        let landing = world
            .spawn((
                Name::new("landing"),
                GameArea::new(1),
                StartArea::new(GameFaction::Egypt),
                pop,
                LandPassage::default(),
                Transform::default(),
            ))
            .id();

        start_barbarian_hordes(&mut world, victim);
        run_barbarian_hordes_to_completion(&mut world, victim);

        let pop_after = world.get::<Population>(landing).unwrap();
        assert_eq!(
            pop_after.population_for_player(victim), 0,
            "2 victim tokens cannot survive real conflict resolution against 15 barbarian tokens"
        );
    }

    #[test]
    fn surplus_cascades_to_the_adjacent_area_and_fights_a_second_real_conflict() {
        // Rule 30.5231/30.5232/30.5234: once the landing area's fight is
        // resolved, Barbarians beyond the area's population limit move on to
        // the (here: only, so trivially "greatest damage") adjacent area and
        // fight again there too.
        let mut world = barbarian_test_world();
        let victim = world.spawn(Faction { faction: GameFaction::Egypt }).id();

        // Landing area: 1 victim token, low max_population so almost all 15
        // barbarians become surplus after this one-sided fight.
        let mut landing_pop = Population::new(2);
        let landing_victim_token = world.spawn_empty().id();
        landing_pop.add_token_to_area(victim, landing_victim_token);
        let mut landing_passage = LandPassage::default();
        // Second area: the victim's other holdings, reachable from landing.
        let mut second_pop = Population::new(8);
        let second_tokens: Vec<Entity> = (0..3).map(|_| world.spawn_empty().id()).collect();
        for &t in &second_tokens {
            second_pop.add_token_to_area(victim, t);
        }
        let second_area = world
            .spawn((Name::new("second"), GameArea::new(2), second_pop, LandPassage::default(), Transform::default()))
            .id();
        landing_passage.add_passage(second_area);
        world.spawn((
            Name::new("landing"),
            GameArea::new(1),
            StartArea::new(GameFaction::Egypt),
            landing_pop,
            landing_passage,
            Transform::default(),
        ));

        start_barbarian_hordes(&mut world, victim);
        run_barbarian_hordes_to_completion(&mut world, victim);

        // The cascade must have reached the second area and fought there too
        // -- the victim's 3 tokens there cannot survive against a barbarian
        // surplus this lopsided, by the same deterministic conflict math
        // exercised in the single-area test above.
        let second_pop_after = world.get::<Population>(second_area).unwrap();
        assert_eq!(
            second_pop_after.population_for_player(victim), 0,
            "the movement cascade must reach the second area and trigger real conflict there"
        );
    }

    #[test]
    fn barbarians_eliminating_a_city_draw_no_trade_card_from_the_victim() {
        // Rule 30.526: if Barbarians eliminate a city, NO trade card is
        // drawn from the victim at all -- unlike a normal player attacker
        // (rule 24.51), where the victim always loses a card to a real
        // attacker's hand.
        let mut world = barbarian_test_world();

        let spare_city_token = world.spawn_empty().id();
        let victim = world.spawn((
            Faction { faction: GameFaction::Egypt },
            TokenStock::new(0, vec![]),
            CityTokenStock::new(1, vec![spare_city_token]),
            PlayerCities::default(),
            PlayerAreas::default(),
        )).id();
        // Give the victim one trade card -- if 30.526 is violated, this gets
        // removed even though no Barbarian PlayerTradeCards exists to receive it.
        let mut victim_cards = PlayerTradeCards::default();
        victim_cards.add_trade_card(TradeCard::Ochre);
        world.entity_mut(victim).insert(victim_cards);

        let city_token = world.spawn_empty().id();
        let mut player_cities = PlayerCities::default();
        let landing_pop = Population::new(6); // no defenders -- an easy city kill
        let landing = world
            .spawn((
                Name::new("landing"),
                GameArea::new(1),
                StartArea::new(GameFaction::Egypt),
                landing_pop,
                LandPassage::default(),
                BuiltCity::new(city_token, victim),
                Transform::default(),
            ))
            .id();
        player_cities.build_city_in_area(landing, city_token);
        world.entity_mut(victim).insert(player_cities);

        start_barbarian_hordes(&mut world, victim);
        run_barbarian_hordes_to_completion(&mut world, victim);

        // The city must actually have been eliminated (proves the city-
        // conflict path, not the plain-token path, ran) ...
        assert!(world.get::<BuiltCity>(landing).is_none());
        // ... and the victim's trade card must still be there.
        let cards_after = world.get::<PlayerTradeCards>(victim).unwrap();
        assert_eq!(cards_after.number_of_trade_cards(), 1, "rule 30.526: no card draw on a Barbarian city kill");
    }

    // ── Rule 30.91: Piracy -- real Pirate cities, not a beneficiary transfer ──
    //
    // Like the Barbarian Hordes tests above, these exercise real ECS state
    // (BuiltCity/PlayerCities/CityTokenStock ownership, city support) through
    // the actual advance_piracy system, not just PiracyState's own math.

    use crate::civilization::concepts::check_city_support::{
        start_check_city_support, HasTooManyCities, NeedsToCheckCitySupport,
    };
    use crate::civilization::concepts::resolve_calamities::calamities::piracy::PiracyState;
    use crate::civilization::concepts::resolve_calamities::resolve_calamities_systems::{
        advance_piracy, transfer_city_to_new_owner,
    };
    use crate::civilization::concepts::resolve_calamities::resolve_calamities_ui_components::CalamitySelectionState;
    use crate::player::Player;

    fn piracy_test_world() -> World {
        let mut world = World::new();
        world.init_resource::<Messages<crate::civilization::concepts::resolve_calamities::resolve_calamities_events::CalamityResolved>>();
        world.init_resource::<CalamitySelectionState>();
        world
    }

    /// Ticks `advance_piracy` until `victim`'s `ResolvingCalamity` is gone, or
    /// panics if that doesn't happen within a generous bound (there's no
    /// unbounded cascade here like Barbarian Hordes -- 4 phases is the real
    /// worst case -- so a stall means a real bug, not slow convergence).
    /// Also runs `transfer_city_to_new_owner` every tick: `TransferCityTo` is
    /// consumed by that regular scheduled system, not an observer (unlike
    /// the conflict markers Barbarian Hordes' tests rely on), so it has to
    /// be driven explicitly for `ApplyEffects`'s city transfers to land.
    fn run_piracy_to_completion(world: &mut World, victim: Entity) {
        for _ in 0..20 {
            if world.get::<ResolvingCalamity>(victim).is_none() {
                return;
            }
            world.run_system_once(advance_piracy).unwrap();
            world.flush();
            world.run_system_once(transfer_city_to_new_owner).unwrap();
            world.flush();
        }
        let stuck_phase = world.get::<ResolvingCalamity>(victim).map(|rc| format!("{rc:?}"));
        panic!("Piracy resolution did not converge within 20 ticks; stuck at: {stuck_phase:?}");
    }

    fn start_piracy(world: &mut World, victim: Entity, traded_by: Option<Entity>) {
        let context = CalamityContext::new(TradeCard::Piracy, victim, traded_by);
        let mut state = PiracyState::new();
        if let Some(trader) = traded_by {
            state = state.with_immune_player(trader);
        }
        world.entity_mut(victim).insert((
            ActiveCalamityResolution::new(context),
            ResolvingCalamity::Piracy(state),
        ));
    }

    /// Spawns a player-owned coastal (or, if `coastal` is false, inland)
    /// city area and gives the owner the components a city owner needs
    /// (`PlayerCities`, `CityTokenStock`) if they don't already have them.
    fn spawn_city_for(world: &mut World, owner: Entity, name: &str, coastal: bool) -> Entity {
        let city_token = world.spawn_empty().id();
        let mut area = world.spawn((
            Name::new(name.to_string()),
            GameArea::new(world.entities().len() as i32),
            Population::new(6),
            LandPassage::default(),
            BuiltCity::new(city_token, owner),
            Transform::default(),
        ));
        if coastal {
            area.insert(SeaPassage::default());
        }
        let area = area.id();

        if let Some(mut cities) = world.get_mut::<PlayerCities>(owner) {
            cities.build_city_in_area(area, city_token);
        } else {
            let mut cities = PlayerCities::default();
            cities.build_city_in_area(area, city_token);
            world.entity_mut(owner).insert(cities);
        }
        if world.get::<CityTokenStock>(owner).is_none() {
            world.entity_mut(owner).insert(CityTokenStock::new(9, vec![]));
        }
        area
    }

    fn find_pirate_nation(world: &mut World) -> Entity {
        world
            .query_filtered::<Entity, With<PirateNation>>()
            .single(world)
            .expect("a PirateNation entity must exist after Piracy resolves")
    }

    #[test]
    fn primary_victim_loses_2_coastal_cities_to_real_pirate_cities() {
        let mut world = piracy_test_world();
        let victim = world.spawn_empty().id();
        let coastal_1 = spawn_city_for(&mut world, victim, "coastal-1", true);
        let coastal_2 = spawn_city_for(&mut world, victim, "coastal-2", true);
        // A third, inland city that must NOT be touched -- proves coastal
        // targeting, not "any 2 cities", drives the selection (rule 30.911).
        let inland = spawn_city_for(&mut world, victim, "inland", false);

        start_piracy(&mut world, victim, None);
        run_piracy_to_completion(&mut world, victim);

        let pirate_nation = find_pirate_nation(&mut world);
        for area in [coastal_1, coastal_2] {
            let built_city = world.get::<BuiltCity>(area).expect("area must still have a city");
            assert_eq!(built_city.player, pirate_nation, "coastal city must now be Pirate-owned");
        }
        let inland_city = world.get::<BuiltCity>(inland).unwrap();
        assert_eq!(inland_city.player, victim, "the inland city must be untouched");

        let victim_cities = world.get::<PlayerCities>(victim).unwrap();
        assert_eq!(victim_cities.number_of_cities(), 1, "victim keeps only the inland city");
    }

    #[test]
    fn two_other_players_each_lose_one_coastal_city_but_the_trading_player_is_immune() {
        let mut world = piracy_test_world();
        // advance_piracy's query requires &PlayerCities unconditionally (as
        // every real player has, even with zero cities via setup_player) --
        // an entity missing it entirely wouldn't match the query at all and
        // would never be processed.
        let victim = world.spawn(PlayerCities::default()).id();
        // The victim has no coastal cities of its own -- 30.912 still applies
        // to secondary victims regardless of the primary victim's holdings.

        let trader = world.spawn((Player, TokenStock::new(0, vec![]))).id();
        let trader_city = spawn_city_for(&mut world, trader, "trader-coastal", true);

        let first_secondary = world.spawn((Player, TokenStock::new(0, vec![]))).id();
        let first_secondary_city = spawn_city_for(&mut world, first_secondary, "first-secondary-coastal", true);

        let other_secondary = world.spawn((Player, TokenStock::new(0, vec![]))).id();
        let other_secondary_city = spawn_city_for(&mut world, other_secondary, "other-secondary-coastal", true);

        start_piracy(&mut world, victim, Some(trader));
        run_piracy_to_completion(&mut world, victim);

        let pirate_nation = find_pirate_nation(&mut world);
        assert_eq!(
            world.get::<BuiltCity>(trader_city).unwrap().player, trader,
            "rule 30.912: the trading player may not be a secondary victim"
        );
        for area in [first_secondary_city, other_secondary_city] {
            assert_eq!(
                world.get::<BuiltCity>(area).unwrap().player, pirate_nation,
                "the other 2 players must each lose their coastal city to the Pirate nation"
            );
        }
    }

    #[test]
    fn pirate_city_is_exempt_from_city_support() {
        // Rule 30.913: Pirate cities do not require city support.
        let mut world = piracy_test_world();
        let victim = world.spawn_empty().id();
        spawn_city_for(&mut world, victim, "coastal-1", true);
        spawn_city_for(&mut world, victim, "coastal-2", true);

        start_piracy(&mut world, victim, None);
        run_piracy_to_completion(&mut world, victim);
        let pirate_nation = find_pirate_nation(&mut world);

        world.init_resource::<NextState<GameActivity>>();
        world.run_system_once(start_check_city_support).unwrap();
        world.flush();

        assert!(
            world.get::<NeedsToCheckCitySupport>(pirate_nation).is_none(),
            "the Pirate nation must never be flagged for a city support check"
        );
        assert!(
            world.get::<HasTooManyCities>(pirate_nation).is_none(),
            "the Pirate nation must never be marked as under-supported"
        );
    }

    #[test]
    fn a_real_player_can_attack_and_destroy_a_pirate_city_and_pillage_it() {
        // Rule 30.913: Pirate cities remain until attacked and destroyed;
        // when destroyed, the attacker may pillage. This must work through
        // the SAME conflict-resolution machinery (on_add_unresolved_city_conflict)
        // any other city attack uses -- no bespoke Pirate-combat code exists.
        let mut world = piracy_test_world();
        world.init_resource::<ConflictCounterResource>();
        world.init_resource::<NextState<GameActivity>>();
        world.init_resource::<CameraFocusQueue>();
        world.add_observer(on_add_unresolved_conflict);
        world.add_observer(on_add_unresolved_city_conflict);

        let victim = world.spawn_empty().id();
        let city_area = spawn_city_for(&mut world, victim, "coastal-1", true);
        start_piracy(&mut world, victim, None);
        run_piracy_to_completion(&mut world, victim);
        let pirate_nation = find_pirate_nation(&mut world);
        assert_eq!(world.get::<BuiltCity>(city_area).unwrap().player, pirate_nation);

        // A real attacker: 8 tokens (>= the base 7-token attack threshold,
        // neither side has Engineering) placed directly in the Pirate
        // city's area -- enough to qualify as a "large invader" and force
        // the city-replaced-by-tokens path.
        let attacker_stock_tokens: Vec<Entity> = (0..5).map(|_| world.spawn_empty().id()).collect();
        let attacker = world
            .spawn((
                Player,
                CityTokenStock::new(9, vec![]),
                TokenStock::new(5, attacker_stock_tokens),
                PlayerCities::default(),
                PlayerAreas::default(),
                Treasury::default(),
            ))
            .id();
        let attack_tokens: Vec<Entity> = (0..8).map(|_| world.spawn_empty().id()).collect();
        {
            let mut pop = world.get_mut::<Population>(city_area).unwrap();
            for &t in &attack_tokens {
                pop.add_token_to_area(attacker, t);
            }
        }

        world.entity_mut(city_area).insert(UnresolvedCityConflict);
        // Cascading observers (city conflict -> chained token conflict) need
        // a couple of flush points to fully resolve, same as the Barbarian
        // Hordes AwaitingConflict polling pattern.
        for _ in 0..5 {
            world.flush();
            if world.get::<UnresolvedCityConflict>(city_area).is_none()
                && world.get::<UnresolvedConflict>(city_area).is_none()
            {
                break;
            }
        }

        let built_city_after = world.get::<BuiltCity>(city_area);
        assert!(
            built_city_after.is_none() || built_city_after.unwrap().player != pirate_nation,
            "the Pirate city must no longer be Pirate-owned after a successful attack"
        );
        let attacker_treasury = world.get::<Treasury>(attacker).unwrap();
        assert!(
            attacker_treasury.tokens_in_treasury() > 0,
            "rule 30.913: the attacker may pillage a destroyed Pirate city"
        );
    }

    // ========================================================================
    // Rule 30.41 Civil War (ECS-level, via advance_civil_war)
    // ========================================================================
    //
    // Regression coverage for the 30.412/30.413/30.415 rework: the
    // beneficiary's 20-point top-up (30.4123) is now additive to the victim's
    // 15-point pick (30.4121), drawn from what's left of the victim's board,
    // rather than a subset of it; the second faction (30.413) is the
    // remainder; the primary victim chooses which faction to keep (30.415);
    // and Military (30.414) removes 5 points from BOTH factions.

    use crate::civilization::concepts::resolve_calamities::calamities::civil_war::{
        CivilWarState, FactionChoice,
    };
    use crate::civilization::concepts::resolve_calamities::resolve_calamities_systems::advance_civil_war;
    use crate::civilization::concepts::resolve_calamities::resolve_calamities_ui_components::{
        CivilWarSelectionState, CivilWarUiRole,
    };
    use bevy::platform::collections::HashSet;

    fn civil_war_test_world() -> World {
        let mut world = World::new();
        world.init_resource::<Messages<crate::civilization::concepts::resolve_calamities::resolve_calamities_events::CalamityResolved>>();
        world.init_resource::<CivilWarSelectionState>();
        world
    }

    /// Spawns a full player: `Player`, `TokenStock` seeded with `stock` spare
    /// tokens, and empty `PlayerAreas`/`PlayerCities` (both are queried
    /// unconditionally by `advance_civil_war` for the victim).
    fn spawn_civil_war_player(world: &mut World, stock: usize) -> Entity {
        let stock_tokens: Vec<Entity> = (0..stock).map(|_| world.spawn_empty().id()).collect();
        world
            .spawn((
                Player,
                TokenStock::new(stock + 200, stock_tokens),
                PlayerAreas::default(),
                PlayerCities::default(),
            ))
            .id()
    }

    /// Spawns an area and places `count` fresh tokens on it, owned by `owner`,
    /// via both `Population` (board bookkeeping) and `PlayerAreas` (owner-side
    /// bookkeeping) -- the two `advance_civil_war` actually reads.
    fn place_tokens(world: &mut World, owner: Entity, count: usize) -> (Entity, Vec<Entity>) {
        let tokens: Vec<Entity> = (0..count).map(|_| world.spawn_empty().id()).collect();
        let mut pop = Population::new(count + 10);
        for &t in &tokens {
            pop.add_token_to_area(owner, t);
        }
        let area = world
            .spawn((Name::new("victim-heartland"), GameArea::new(1), pop, Transform::default()))
            .id();
        {
            let mut areas = world.get_mut::<PlayerAreas>(owner).unwrap();
            for &t in &tokens {
                areas.add_token_to_area(area, t);
            }
        }
        (area, tokens)
    }

    fn start_civil_war(world: &mut World, victim: Entity, state: CivilWarState) {
        let context = CalamityContext::new(TradeCard::CivilWar, victim, None);
        world.entity_mut(victim).insert((
            ActiveCalamityResolution::new(context),
            ResolvingCalamity::CivilWar(state),
        ));
    }

    /// Ticks `advance_civil_war` until `victim`'s `ResolvingCalamity` is gone.
    /// Whenever a human is awaiting input, auto-resolves it: for the
    /// 30.4121 victim-selection step, selects exactly the required token
    /// count; for the 30.415 faction choice, keeps `keep_second`'s faction.
    fn run_civil_war_to_completion(world: &mut World, victim: Entity, keep_second: bool) {
        for _ in 0..30 {
            if world.get::<ResolvingCalamity>(victim).is_none() {
                return;
            }
            if let Some(acting) = world.resource::<CivilWarSelectionState>().acting_player
                && world.get::<AwaitingHumanCalamitySelection>(acting).is_some()
            {
                let role = world.resource::<CivilWarSelectionState>().role.clone();
                match role {
                    CivilWarUiRole::Victim => {
                        let target = world.resource::<CivilWarSelectionState>().target_points;
                        let mut cw = world.resource_mut::<CivilWarSelectionState>();
                        for _ in 0..target {
                            cw.increment_tokens();
                        }
                    }
                    CivilWarUiRole::Beneficiary => {
                        // Take nothing extra; taking 0 is a valid beneficiary choice.
                    }
                    CivilWarUiRole::ChooseFaction => {
                        let mut cw = world.resource_mut::<CivilWarSelectionState>();
                        cw.choose_faction(if keep_second { FactionChoice::Second } else { FactionChoice::First });
                    }
                }
                world.entity_mut(acting).remove::<AwaitingHumanCalamitySelection>();
            }
            world.run_system_once(advance_civil_war).unwrap();
            world.flush();
        }
        let stuck_phase = world.get::<ResolvingCalamity>(victim).map(|rc| format!("{rc:?}"));
        panic!("Civil War resolution did not converge within 30 ticks; stuck at: {stuck_phase:?}");
    }

    /// Rule 30.415: a human primary victim can choose to keep the SECOND
    /// faction, sending the FIRST faction (victim's 15-pt pick + beneficiary's
    /// 20-pt top-up) to the beneficiary instead. The kept faction's tokens
    /// must be completely untouched.
    #[test]
    fn human_victim_choosing_second_faction_sends_the_first_faction_to_the_beneficiary() {
        let mut world = civil_war_test_world();
        let victim = spawn_civil_war_player(&mut world, 5);
        let beneficiary = spawn_civil_war_player(&mut world, 200);
        world.entity_mut(victim).insert(IsHuman);

        let (area, board_tokens) = place_tokens(&mut world, victim, 40);
        start_civil_war(&mut world, victim, CivilWarState::new());

        run_civil_war_to_completion(&mut world, victim, true); // keep second

        // 30.412: first faction = 15 (victim) + 20 (beneficiary top-up) = 35 pts,
        // all drawn from the 40-token board. 30.413: second faction = the
        // remaining 5. 30.415: victim keeps the second faction, so the first
        // faction (35 tokens) transfers to the beneficiary.
        let pop_after = world.get::<Population>(area).unwrap();
        let victim_remaining = pop_after.tokens_for_player(&victim).cloned().unwrap_or_default();
        let beneficiary_now_present = pop_after.tokens_for_player(&beneficiary).cloned().unwrap_or_default();

        assert_eq!(victim_remaining.len(), 5, "victim keeps exactly the untransferred second faction");
        assert!(
            victim_remaining.is_subset(&board_tokens.iter().copied().collect::<HashSet<_>>()),
            "the tokens the victim kept must be original board tokens, untouched"
        );
        assert_eq!(beneficiary_now_present.len(), 35, "the beneficiary's own tokens replace the transferred 35");

        let victim_stock = world.get::<TokenStock>(victim).unwrap();
        assert_eq!(victim_stock.tokens_in_stock(), 5 /* starting stock */ + 35 /* returned */);

        let beneficiary_stock = world.get::<TokenStock>(beneficiary).unwrap();
        assert_eq!(beneficiary_stock.tokens_in_stock(), 200 - 35, "beneficiary drew 35 replacement tokens from its own stock");
    }

    /// Rule 30.415, AI default: with no explicit choice, the AI keeps
    /// whichever faction has MORE unit points -- here the first faction
    /// (35 pts) dwarfs the second (5 pts), so the small second faction is
    /// the one that transfers.
    #[test]
    fn ai_victim_keeps_the_bigger_faction_by_default() {
        let mut world = civil_war_test_world();
        let victim = spawn_civil_war_player(&mut world, 5); // no IsHuman -- AI
        let beneficiary = spawn_civil_war_player(&mut world, 200);

        let (area, _board_tokens) = place_tokens(&mut world, victim, 40);
        start_civil_war(&mut world, victim, CivilWarState::new());

        run_civil_war_to_completion(&mut world, victim, false);

        let pop_after = world.get::<Population>(area).unwrap();
        let victim_remaining = pop_after.tokens_for_player(&victim).cloned().unwrap_or_default();
        let beneficiary_now_present = pop_after.tokens_for_player(&beneficiary).cloned().unwrap_or_default();

        assert_eq!(victim_remaining.len(), 35, "AI kept the bigger (first) faction");
        assert_eq!(beneficiary_now_present.len(), 5, "only the smaller (second) faction transferred");
    }

    /// Rule 30.413: if the first faction (15 + 20 = 35 pts) consumes the
    /// victim's ENTIRE board, there is no second faction and therefore no
    /// Civil War -- the calamity must fizzle without moving a single token.
    #[test]
    fn no_second_faction_means_no_civil_war() {
        let mut world = civil_war_test_world();
        let victim = spawn_civil_war_player(&mut world, 5); // no IsHuman -- AI
        let beneficiary = spawn_civil_war_player(&mut world, 200);

        let (area, board_tokens) = place_tokens(&mut world, victim, 35); // exactly 15 + 20
        start_civil_war(&mut world, victim, CivilWarState::new());

        run_civil_war_to_completion(&mut world, victim, false);

        let pop_after = world.get::<Population>(area).unwrap();
        let victim_remaining = pop_after.tokens_for_player(&victim).cloned().unwrap_or_default();
        assert_eq!(
            victim_remaining, board_tokens.iter().copied().collect::<HashSet<_>>(),
            "the whole board must be untouched -- 30.413 fizzles the calamity"
        );
        assert!(pop_after.tokens_for_player(&beneficiary).is_none(), "nothing transfers when there's no second faction");

        let victim_stock = world.get::<TokenStock>(victim).unwrap();
        assert_eq!(victim_stock.tokens_in_stock(), 5, "no tokens returned to stock -- nothing was ever selected for transfer");
    }

    /// Rule 30.414: Military removes 5 points from EACH faction before the
    /// 30.415 choice. With a 60-token board (15 + 20 = 35 pt first faction,
    /// 25 pt second faction pre-penalty), the penalty shrinks them to 30 and
    /// 20 respectively; the AI then keeps the (still bigger) first faction,
    /// so only the reduced 20-token second faction transfers -- proving the
    /// second faction's penalty actually landed (without it, 25 would transfer).
    #[test]
    fn military_penalty_shrinks_the_transferring_second_faction() {
        let mut world = civil_war_test_world();
        let victim = spawn_civil_war_player(&mut world, 5); // no IsHuman -- AI
        let beneficiary = spawn_civil_war_player(&mut world, 200);

        let (area, _board_tokens) = place_tokens(&mut world, victim, 60);
        start_civil_war(&mut world, victim, CivilWarState::new().with_military_penalty());

        run_civil_war_to_completion(&mut world, victim, false);

        let pop_after = world.get::<Population>(area).unwrap();
        let beneficiary_now_present = pop_after.tokens_for_player(&beneficiary).cloned().unwrap_or_default();
        assert_eq!(
            beneficiary_now_present.len(), 20,
            "30.414: the second faction (25 pts) must be reduced to 20 before transferring"
        );
    }
}
