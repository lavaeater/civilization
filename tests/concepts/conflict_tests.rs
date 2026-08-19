use crate::setup_player;
use adv_civ::civilization::{
    find_conflict_zones, on_add_unresolved_city_conflict, on_add_unresolved_conflict, BuiltCity,
    CameraFocusQueue, CivCardName, GameArea, GameFaction, LandPassage,
    PlayerAreas, PlayerCities, PlayerCivilizationCards, PlayerTradeCards, Population, TokenStock,
    Treasury, TradeCard, UnresolvedCityConflict, UnresolvedConflict, CityTokenStock,
    ConflictCounterResource,
};
use adv_civ::{GameActivity, GameState};
use bevy::ecs::system::{RunSystemOnce};
use bevy::prelude::{App, AppExtStates, Name, Transform};
use bevy::state::app::StatesPlugin;
/****************************************************
Test for the find_conflict_zones system
Given two players that have tokens in an area,
when the system is run, that area should have a component
added indicating that it has a conflict.
*****************************************************/

fn setup_conflict_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(StatesPlugin)
        .insert_state(GameState::Playing)
        .add_sub_state::<GameActivity>()
        .insert_state(GameActivity::Conflict)
        .init_resource::<ConflictCounterResource>()
        .init_resource::<CameraFocusQueue>();
    app
}
#[test]
fn given_an_area_with_a_city_and_some_population() {
    // Arrange
    let mut app = setup_conflict_test_app();

    let (player_one, _, mut p_one_cities) =
        setup_player(&mut app, "player one", GameFaction::Egypt);

    let (player_two, mut player_two_tokens, _) =
        setup_player(&mut app, "player two", GameFaction::Crete);

    let mut population = Population::new(4);
    for token in player_two_tokens.drain(0..5).collect::<Vec<_>>() {
        population.add_token_to_area(player_two, token);
    }

    let area = app
        .world_mut()
        .spawn((
            Name::new("egypt"),
            GameArea::new(1),
            LandPassage::default(),
            population,
            BuiltCity::new(player_one, p_one_cities.pop().unwrap()),
            Transform::default(),
        ))
        .id();

    // Act
    app.world_mut().run_system_once(find_conflict_zones).unwrap();
    // Assert
    assert!(app.world().get::<UnresolvedCityConflict>(area).is_some());
}

#[test]
fn given_a_city_conflict_with_too_few_tokens() {
    // Arrange
    let mut app = setup_conflict_test_app();
    app.add_observer(on_add_unresolved_conflict)
        .add_observer(on_add_unresolved_city_conflict);

    let (player_one, _, mut p_one_cities) =
        setup_player(&mut app, "player one", GameFaction::Egypt);

    let (player_two, mut player_two_tokens, _) =
        setup_player(&mut app, "player two", GameFaction::Crete);

    let mut population = Population::new(4);

    for token in player_two_tokens.drain(0..5).collect::<Vec<_>>() {
        population.add_token_to_area(player_two, token);
    }

    let area = app
        .world_mut()
        .spawn((
            Name::new("egypt"),
            GameArea::new(1),
            LandPassage::default(),
            population,
            BuiltCity::new(player_one, p_one_cities.pop().unwrap()),
            UnresolvedCityConflict,
            Transform::default(),
        ))
        .id();

    // Act
    app.world_mut().run_system_once(find_conflict_zones).unwrap();
    // Assert
    assert!(app.world().get::<UnresolvedConflict>(area).is_none());
    assert!(app.world().get::<UnresolvedCityConflict>(area).is_none());
}

#[test]
fn given_a_city_conflict_with_enough_tokens() {
    // Arrange
    let mut app = setup_conflict_test_app();
    app.add_observer(on_add_unresolved_conflict)
        .add_observer(on_add_unresolved_city_conflict);

    let (player_one, _, mut p_one_cities) =
        setup_player(&mut app, "player one", GameFaction::Egypt);

    let (player_two, mut player_two_tokens, _) =
        setup_player(&mut app, "player two", GameFaction::Crete);

    let mut population = Population::new(4);

    for token in player_two_tokens.drain(0..8).collect::<Vec<_>>() {
        population.add_token_to_area(player_two, token);
    }

    let city_token = p_one_cities.pop().unwrap();
    let mut player_one_cities = PlayerCities::default();

    let area = app
        .world_mut()
        .spawn((
            Name::new("egypt"),
            GameArea::new(1),
            LandPassage::default(),
            population,
            BuiltCity::new(city_token, player_one),
            UnresolvedCityConflict,
            Transform::default(),
        ))
        .id();
    player_one_cities.build_city_in_area(area, city_token);

    app.world_mut()
        .entity_mut(player_one)
        .insert(player_one_cities);

    assert!(app.world().get::<PlayerAreas>(player_one).is_some());
    assert!(app.world().get::<PlayerCities>(player_one).is_some());
    assert!(app.world().get::<TokenStock>(player_one).is_some());
    assert!(app.world().get::<CityTokenStock>(player_one).is_some());

    // Act
    app.world_mut().run_system_once(find_conflict_zones).unwrap();
    app.update();
    app.update();
    // Assert
    assert!(app.world().get::<UnresolvedCityConflict>(area).is_none());
    assert!(app.world().get::<UnresolvedConflict>(area).is_none());
}

#[test]
fn given_two_players_in_an_area_with_too_much_population_area_is_marked_as_conflict_zone() {
    // Arrange
    let mut app = setup_conflict_test_app();

    let (player_one, mut player_one_tokens, _) =
        setup_player(&mut app, "player one", GameFaction::Egypt);

    let (player_two, mut player_two_tokens, _) =
        setup_player(&mut app, "player two", GameFaction::Crete);

    let mut population = Population::new(4);

    for token in player_one_tokens.drain(0..7).collect::<Vec<_>>() {
        population.add_token_to_area(player_one, token);
    }
    for token in player_two_tokens.drain(0..5).collect::<Vec<_>>() {
        population.add_token_to_area(player_two, token);
    }

    let area = app
        .world_mut()
        .spawn((
            Name::new("egypt"),
            GameArea::new(1),
            LandPassage::default(),
            Transform::default(),
            population,
        ))
        .id();

    // Act
    app.world_mut().run_system_once(find_conflict_zones).unwrap();
    // Assert
    assert!(app.world().get::<UnresolvedConflict>(area).is_some());
}

pub struct TwoPlayerTestStruct {
    pub player_one_tokens: usize,
    pub player_two_tokens: usize,
    pub area_max_population: usize,
    pub player_one_expected: usize,
    pub player_two_expected: usize,
}

impl TwoPlayerTestStruct {
    fn expected(&self) -> usize {
        self.player_one_expected + self.player_two_expected
    }

    fn new(
        player_one_tokens: usize,
        player_two_tokens: usize,
        area_max_population: usize,
        player_one_expected: usize,
        player_two_expected: usize,
    ) -> Self {
        TwoPlayerTestStruct {
            player_one_tokens,
            player_two_tokens,
            area_max_population,
            player_one_expected,
            player_two_expected,
        }
    }
}

pub struct ThreePlayerTestStruct {
    pub player_one_tokens: usize,
    pub player_two_tokens: usize,
    pub player_three_tokens: usize,
    pub area_max_population: usize,
    pub player_one_expected: usize,
    pub player_two_expected: usize,
    pub player_three_expected: usize,
}

impl ThreePlayerTestStruct {
    fn expected(&self) -> usize {
        self.player_one_expected + self.player_two_expected + self.player_three_expected
    }

    fn new(
        player_one_tokens: usize,
        player_two_tokens: usize,
        player_three_tokens: usize,
        area_max_population: usize,
        player_one_expected: usize,
        player_two_expected: usize,
        player_three_expected: usize,
    ) -> Self {
        ThreePlayerTestStruct {
            player_one_tokens,
            player_two_tokens,
            player_three_tokens,
            area_max_population,
            player_one_expected,
            player_two_expected,
            player_three_expected,
        }
    }
}

#[test]
fn when_resolving_conflicts_the_correct_result_is_obtained() {
    // Arrange
    let mut app = setup_conflict_test_app();
    app.add_observer(on_add_unresolved_conflict)
        .add_observer(on_add_unresolved_city_conflict);

    let test_cases = vec![
        TwoPlayerTestStruct::new(1, 1, 1, 0, 0),
        TwoPlayerTestStruct::new(2, 1, 1, 2, 0),
        TwoPlayerTestStruct::new(2, 2, 1, 0, 0),
        TwoPlayerTestStruct::new(2, 1, 2, 2, 0),
        TwoPlayerTestStruct::new(2, 2, 2, 1, 1),
        TwoPlayerTestStruct::new(3, 2, 2, 2, 0),
        TwoPlayerTestStruct::new(3, 2, 3, 2, 1),
        TwoPlayerTestStruct::new(3, 3, 3, 1, 1),
        TwoPlayerTestStruct::new(4, 2, 3, 3, 0),
        TwoPlayerTestStruct::new(3, 3, 4, 2, 2),
        TwoPlayerTestStruct::new(4, 1, 4, 4, 0),
        TwoPlayerTestStruct::new(4, 2, 4, 3, 1),
        TwoPlayerTestStruct::new(4, 3, 4, 3, 1),
        TwoPlayerTestStruct::new(5, 3, 4, 3, 1),
    ];

    let (player_one, mut player_one_tokens, _) =
        setup_player(&mut app, "player one", GameFaction::Egypt);

    let (player_two, mut player_two_tokens, _) =
        setup_player(&mut app, "player two", GameFaction::Crete);

    for test_case in test_cases {
        let mut population = Population::new(test_case.area_max_population);

        for token in player_one_tokens
            .drain(0..test_case.player_one_tokens)
            .collect::<Vec<_>>()
        {
            population.add_token_to_area(player_one, token);
        }
        for token in player_two_tokens
            .drain(0..test_case.player_two_tokens)
            .collect::<Vec<_>>()
        {
            population.add_token_to_area(player_two, token);
        }

        let area = app
            .world_mut()
            .spawn((
                Name::new("egypt"),
                GameArea::new(1),
                LandPassage::default(),
                UnresolvedConflict,
                Transform::default(),
                population,
            ))
            .id();

        // Act
        app.world_mut().run_system_once(find_conflict_zones).unwrap();
        app.update();
        // Assert
        let population = app.world().get::<Population>(area).unwrap();
        assert_eq!(
            population.total_population(),
            test_case.expected(),
            "Conflict resolution with p1: {}, p2: {}, max_pop: {}, exp: {}",
            test_case.player_one_tokens,
            test_case.player_two_tokens,
            test_case.area_max_population,
            test_case.expected()
        );
    }
}

#[test]
fn given_three_conflicteers_the_correct_result_is_obtained() {
    // Arrange
    let mut app = setup_conflict_test_app();
    app.add_observer(on_add_unresolved_conflict)
        .add_observer(on_add_unresolved_city_conflict);

    let test_cases = vec![
        ThreePlayerTestStruct::new(1, 1, 1, 1, 0, 0, 0),
        ThreePlayerTestStruct::new(4, 2, 1, 3, 3, 0, 0),
        ThreePlayerTestStruct::new(4, 4, 4, 3, 1, 1, 1),
        ThreePlayerTestStruct::new(2, 5, 1, 2, 0, 4, 0),
        ThreePlayerTestStruct::new(2, 5, 1, 3, 0, 4, 0),
    ];

    let (player_one, mut player_one_tokens, _) =
        setup_player(&mut app, "player one", GameFaction::Egypt);

    let (player_two, mut player_two_tokens, _) =
        setup_player(&mut app, "player two", GameFaction::Crete);
    let (player_three, mut player_three_tokens, _) =
        setup_player(&mut app, "player three", GameFaction::Babylon);

    for test_case in test_cases {
        let mut population = Population::new(test_case.area_max_population);

        for token in player_one_tokens
            .drain(0..test_case.player_one_tokens)
            .collect::<Vec<_>>()
        {
            population.add_token_to_area(player_one, token);
        }
        for token in player_two_tokens
            .drain(0..test_case.player_two_tokens)
            .collect::<Vec<_>>()
        {
            population.add_token_to_area(player_two, token);
        }
        for token in player_three_tokens
            .drain(0..test_case.player_three_tokens)
            .collect::<Vec<_>>()
        {
            population.add_token_to_area(player_three, token);
        }

        let area = app
            .world_mut()
            .spawn((
                Name::new("egypt"),
                GameArea::new(1),
                LandPassage::default(),
                UnresolvedConflict,
                Transform::default(),
                population,
            ))
            .id();

        // Act
        app.world_mut().run_system_once(find_conflict_zones).unwrap();
        app.update();
        // Assert
        let population = app.world().get::<Population>(area).unwrap();
        assert_eq!(
            population.total_population(),
            test_case.expected(),
            "Conflict resolution with p1: {}, p2: {}, p3: {}, max_pop: {}, exp: {}",
            test_case.player_one_tokens,
            test_case.player_two_tokens,
            test_case.player_three_tokens,
            test_case.area_max_population,
            test_case.expected()
        );
    }
}

// ============================================================================
// Rules 24.51-24.52: consequences of city elimination by direct attack
// ============================================================================

#[test]
fn eliminating_a_city_draws_a_random_trade_card_from_the_victim() {
    // Arrange: player one owns a defenseless city; player two attacks with 7
    // tokens (the base rule-24.31 threshold, neither side holds Engineering).
    let mut app = setup_conflict_test_app();
    app.add_observer(on_add_unresolved_conflict)
        .add_observer(on_add_unresolved_city_conflict);

    let (player_one, _, mut p_one_cities) =
        setup_player(&mut app, "player one", GameFaction::Egypt);
    let (player_two, mut player_two_tokens, _) =
        setup_player(&mut app, "player two", GameFaction::Crete);

    app.world_mut()
        .entity_mut(player_one)
        .insert(PlayerTradeCards::default());
    app.world_mut()
        .get_mut::<PlayerTradeCards>(player_one)
        .unwrap()
        .add_trade_card(TradeCard::Ochre);
    app.world_mut()
        .entity_mut(player_two)
        .insert(PlayerTradeCards::default());

    let mut population = Population::new(4);
    for token in player_two_tokens.drain(0..7).collect::<Vec<_>>() {
        population.add_token_to_area(player_two, token);
    }

    let city_token = p_one_cities.pop().unwrap();
    let mut player_one_cities = PlayerCities::default();
    let area = app
        .world_mut()
        .spawn((
            Name::new("egypt"),
            GameArea::new(1),
            LandPassage::default(),
            population,
            BuiltCity::new(city_token, player_one),
            UnresolvedCityConflict,
            Transform::default(),
        ))
        .id();
    player_one_cities.build_city_in_area(area, city_token);
    app.world_mut()
        .entity_mut(player_one)
        .insert(player_one_cities);

    // Act
    app.world_mut().run_system_once(find_conflict_zones).unwrap();
    app.update();
    app.update();

    // Assert: the city fell (BuiltCity gone), and the card moved from victim to attacker.
    assert!(app.world().get::<BuiltCity>(area).is_none(), "city should have been eliminated");
    let victim_cards = app.world().get::<PlayerTradeCards>(player_one).unwrap();
    let attacker_cards = app.world().get::<PlayerTradeCards>(player_two).unwrap();
    assert_eq!(victim_cards.number_of_trade_cards(), 0, "victim should have lost their only card");
    assert_eq!(attacker_cards.number_of_trade_cards(), 1, "attacker should have drawn the card");
    assert!(attacker_cards.has_trade_card(TradeCard::Ochre));
}

#[test]
fn eliminating_a_city_with_no_trade_cards_draws_nothing() {
    // Same as above, but the victim's hand is empty: no draw should occur,
    // and the attacker's own (also empty) hand should stay empty.
    let mut app = setup_conflict_test_app();
    app.add_observer(on_add_unresolved_conflict)
        .add_observer(on_add_unresolved_city_conflict);

    let (player_one, _, mut p_one_cities) =
        setup_player(&mut app, "player one", GameFaction::Egypt);
    let (player_two, mut player_two_tokens, _) =
        setup_player(&mut app, "player two", GameFaction::Crete);

    app.world_mut()
        .entity_mut(player_one)
        .insert(PlayerTradeCards::default());
    app.world_mut()
        .entity_mut(player_two)
        .insert(PlayerTradeCards::default());

    let mut population = Population::new(4);
    for token in player_two_tokens.drain(0..7).collect::<Vec<_>>() {
        population.add_token_to_area(player_two, token);
    }

    let city_token = p_one_cities.pop().unwrap();
    let mut player_one_cities = PlayerCities::default();
    let area = app
        .world_mut()
        .spawn((
            Name::new("egypt"),
            GameArea::new(1),
            LandPassage::default(),
            population,
            BuiltCity::new(city_token, player_one),
            UnresolvedCityConflict,
            Transform::default(),
        ))
        .id();
    player_one_cities.build_city_in_area(area, city_token);
    app.world_mut()
        .entity_mut(player_one)
        .insert(player_one_cities);

    app.world_mut().run_system_once(find_conflict_zones).unwrap();
    app.update();
    app.update();

    assert!(app.world().get::<BuiltCity>(area).is_none());
    let attacker_cards = app.world().get::<PlayerTradeCards>(player_two).unwrap();
    assert_eq!(attacker_cards.number_of_trade_cards(), 0);
}

#[test]
fn eliminating_a_city_pillages_up_to_3_tokens_from_stock_to_treasury() {
    // Arrange: same base attack; attacker starts with an empty treasury and a
    // full stock (minus the 7 tokens spent attacking).
    let mut app = setup_conflict_test_app();
    app.add_observer(on_add_unresolved_conflict)
        .add_observer(on_add_unresolved_city_conflict);

    let (player_one, _, mut p_one_cities) =
        setup_player(&mut app, "player one", GameFaction::Egypt);
    let (player_two, mut player_two_tokens, _) =
        setup_player(&mut app, "player two", GameFaction::Crete);

    let stock_before = app.world().get::<TokenStock>(player_two).unwrap().tokens_in_stock();
    assert_eq!(app.world().get::<Treasury>(player_two).unwrap().tokens_in_treasury(), 0);

    let mut population = Population::new(4);
    for token in player_two_tokens.drain(0..7).collect::<Vec<_>>() {
        population.add_token_to_area(player_two, token);
    }

    let city_token = p_one_cities.pop().unwrap();
    let mut player_one_cities = PlayerCities::default();
    let area = app
        .world_mut()
        .spawn((
            Name::new("egypt"),
            GameArea::new(1),
            LandPassage::default(),
            population,
            BuiltCity::new(city_token, player_one),
            UnresolvedCityConflict,
            Transform::default(),
        ))
        .id();
    player_one_cities.build_city_in_area(area, city_token);
    app.world_mut()
        .entity_mut(player_one)
        .insert(player_one_cities);

    // Act
    app.world_mut().run_system_once(find_conflict_zones).unwrap();
    app.update();
    app.update();

    // Assert: rule 24.52 — up to 3 tokens moved stock -> treasury for the attacker only.
    assert!(app.world().get::<BuiltCity>(area).is_none());
    let treasury_after = app.world().get::<Treasury>(player_two).unwrap().tokens_in_treasury();
    let stock_after = app.world().get::<TokenStock>(player_two).unwrap().tokens_in_stock();
    assert_eq!(treasury_after, 3, "should pillage the full 3 tokens (plenty in stock)");
    // The 7 "attacking" tokens were placed directly into `population` by this
    // test's Arrange step (bypassing move_from_stock_to_area), so TokenStock
    // never counted them as spent -- only the 3 pillaged tokens leave stock.
    assert_eq!(stock_after, stock_before - 3);
    assert_eq!(
        app.world().get::<Treasury>(player_one).unwrap().tokens_in_treasury(),
        0,
        "victim's treasury must be unaffected (24.52)"
    );
}

#[test]
fn engineering_attacker_needs_only_6_tokens_to_eliminate_a_city() {
    // Rule 24.35: an attacker who alone holds Engineering needs only 6 tokens
    // (not the base 7) to bring down a city.
    let mut app = setup_conflict_test_app();
    app.add_observer(on_add_unresolved_conflict)
        .add_observer(on_add_unresolved_city_conflict);

    let (player_one, _, mut p_one_cities) =
        setup_player(&mut app, "player one", GameFaction::Egypt);
    let (player_two, mut player_two_tokens, _) =
        setup_player(&mut app, "player two", GameFaction::Crete);

    let mut engineering = PlayerCivilizationCards::default();
    engineering.add_card(CivCardName::Engineering);
    app.world_mut().entity_mut(player_two).insert(engineering);

    let mut population = Population::new(4);
    for token in player_two_tokens.drain(0..6).collect::<Vec<_>>() {
        population.add_token_to_area(player_two, token);
    }

    let city_token = p_one_cities.pop().unwrap();
    let mut player_one_cities = PlayerCities::default();
    let area = app
        .world_mut()
        .spawn((
            Name::new("egypt"),
            GameArea::new(1),
            LandPassage::default(),
            population,
            BuiltCity::new(city_token, player_one),
            UnresolvedCityConflict,
            Transform::default(),
        ))
        .id();
    player_one_cities.build_city_in_area(area, city_token);
    app.world_mut()
        .entity_mut(player_one)
        .insert(player_one_cities);

    app.world_mut().run_system_once(find_conflict_zones).unwrap();
    app.update();
    app.update();

    assert!(
        app.world().get::<BuiltCity>(area).is_none(),
        "6 tokens should be enough to eliminate a non-Engineering city for an Engineering attacker"
    );
}

#[test]
fn six_tokens_without_engineering_does_not_eliminate_a_city() {
    // Control for the test above: without Engineering, 6 tokens is one short
    // of the base-7 threshold (rule 24.31), so the city must survive.
    let mut app = setup_conflict_test_app();
    app.add_observer(on_add_unresolved_conflict)
        .add_observer(on_add_unresolved_city_conflict);

    let (player_one, _, mut p_one_cities) =
        setup_player(&mut app, "player one", GameFaction::Egypt);
    let (player_two, mut player_two_tokens, _) =
        setup_player(&mut app, "player two", GameFaction::Crete);

    let mut population = Population::new(4);
    for token in player_two_tokens.drain(0..6).collect::<Vec<_>>() {
        population.add_token_to_area(player_two, token);
    }

    let city_token = p_one_cities.pop().unwrap();
    let mut player_one_cities = PlayerCities::default();
    let area = app
        .world_mut()
        .spawn((
            Name::new("egypt"),
            GameArea::new(1),
            LandPassage::default(),
            population,
            BuiltCity::new(city_token, player_one),
            UnresolvedCityConflict,
            Transform::default(),
        ))
        .id();
    player_one_cities.build_city_in_area(area, city_token);
    app.world_mut()
        .entity_mut(player_one)
        .insert(player_one_cities);

    app.world_mut().run_system_once(find_conflict_zones).unwrap();
    app.update();
    app.update();

    assert!(
        app.world().get::<BuiltCity>(area).is_some(),
        "6 tokens without Engineering must not be enough to eliminate a city"
    );
}

#[test]
fn engineering_defender_requires_8_tokens_to_be_eliminated() {
    // Rule 24.35: a defender who alone holds Engineering raises the
    // requirement to 8 tokens. 7 (the non-Engineering base) is not enough.
    let mut app = setup_conflict_test_app();
    app.add_observer(on_add_unresolved_conflict)
        .add_observer(on_add_unresolved_city_conflict);

    let (player_one, _, mut p_one_cities) =
        setup_player(&mut app, "player one", GameFaction::Egypt);
    let (player_two, mut player_two_tokens, _) =
        setup_player(&mut app, "player two", GameFaction::Crete);

    let mut engineering = PlayerCivilizationCards::default();
    engineering.add_card(CivCardName::Engineering);
    app.world_mut().entity_mut(player_one).insert(engineering);

    let mut population = Population::new(4);
    for token in player_two_tokens.drain(0..7).collect::<Vec<_>>() {
        population.add_token_to_area(player_two, token);
    }

    let city_token = p_one_cities.pop().unwrap();
    let mut player_one_cities = PlayerCities::default();
    let area = app
        .world_mut()
        .spawn((
            Name::new("egypt"),
            GameArea::new(1),
            LandPassage::default(),
            population,
            BuiltCity::new(city_token, player_one),
            UnresolvedCityConflict,
            Transform::default(),
        ))
        .id();
    player_one_cities.build_city_in_area(area, city_token);
    app.world_mut()
        .entity_mut(player_one)
        .insert(player_one_cities);

    app.world_mut().run_system_once(find_conflict_zones).unwrap();
    app.update();
    app.update();

    assert!(
        app.world().get::<BuiltCity>(area).is_some(),
        "7 tokens must not be enough against an Engineering-holding defender (needs 8)"
    );
}

#[test]
fn both_sides_holding_engineering_cancels_back_to_base_threshold() {
    // Rule 24.35: if both attacker and defender hold Engineering, the
    // modifiers cancel and the base 7-token threshold applies again.
    let mut app = setup_conflict_test_app();
    app.add_observer(on_add_unresolved_conflict)
        .add_observer(on_add_unresolved_city_conflict);

    let (player_one, _, mut p_one_cities) =
        setup_player(&mut app, "player one", GameFaction::Egypt);
    let (player_two, mut player_two_tokens, _) =
        setup_player(&mut app, "player two", GameFaction::Crete);

    let mut defender_engineering = PlayerCivilizationCards::default();
    defender_engineering.add_card(CivCardName::Engineering);
    app.world_mut().entity_mut(player_one).insert(defender_engineering);
    let mut attacker_engineering = PlayerCivilizationCards::default();
    attacker_engineering.add_card(CivCardName::Engineering);
    app.world_mut().entity_mut(player_two).insert(attacker_engineering);

    let mut population = Population::new(4);
    for token in player_two_tokens.drain(0..7).collect::<Vec<_>>() {
        population.add_token_to_area(player_two, token);
    }

    let city_token = p_one_cities.pop().unwrap();
    let mut player_one_cities = PlayerCities::default();
    let area = app
        .world_mut()
        .spawn((
            Name::new("egypt"),
            GameArea::new(1),
            LandPassage::default(),
            population,
            BuiltCity::new(city_token, player_one),
            UnresolvedCityConflict,
            Transform::default(),
        ))
        .id();
    player_one_cities.build_city_in_area(area, city_token);
    app.world_mut()
        .entity_mut(player_one)
        .insert(player_one_cities);

    app.world_mut().run_system_once(find_conflict_zones).unwrap();
    app.update();
    app.update();

    assert!(
        app.world().get::<BuiltCity>(area).is_none(),
        "with both sides holding Engineering, 7 tokens (the base threshold) should be enough again"
    );
}
