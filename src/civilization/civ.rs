use bevy::app::{App, Plugin, Update};
use crate::player::Player;
use bevy::prelude::{in_state, BuildChildren, Children, Commands, Component, Entity, Event, EventReader, IntoSystemConfigs, Name, OnEnter, Query, Reflect, With};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use crate::civilization::census::{perform_census, Census, CensusOrder};
use crate::civilization::population_expansion::{check_population_expansion_eligibility, expand_population, handle_manual_population_expansion, handle_population_expansion_end, handle_population_expansion_start, BeginPopulationExpansionEvent, CheckPopulationExpansionEligibilityEvent, StartManualPopulationExpansionEvent};
use crate::GameState;

pub struct CivilizationPlugin;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for CivilizationPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<Token>()
            .register_type::<Census>()
            .add_event::<GameActivityStarted>()
            .add_event::<GameActivityEnded>()
            .add_event::<BeginPopulationExpansionEvent>()
            .add_event::<CheckPopulationExpansionEligibilityEvent>()
            .add_event::<StartManualPopulationExpansionEvent>()
            .add_event::<StartHandleSurplusPopulationEvent>()
            .add_event::<MoveTokensFromStockToAreaCommand>()
            .add_event::<MoveTokenFromAreaToAreaCommand>()
            .add_systems(OnEnter(GameState::Playing), setup_game)
            .add_plugins(WorldInspectorPlugin::new())
            .insert_resource(CensusOrder { players_by_population: vec![] })
            .add_systems(
                Update, (
                    move_token_from_area_to_area
                        .run_if(in_state(GameState::Playing)),
                    handle_manual_population_expansion
                        .run_if(in_state(GameState::Playing)),
                    check_population_expansion_eligibility
                        .run_if(in_state(GameState::Playing)),
                    expand_population
                        .run_if(in_state(GameState::Playing)),
                    handle_population_expansion_start
                        .run_if(in_state(GameState::Playing)),
                    handle_population_expansion_end
                        .run_if(in_state(GameState::Playing)),
                    perform_census
                        .run_if(in_state(GameState::Playing)),
                    move_tokens_from_stock_to_area
                        .run_if(in_state(GameState::Playing))
                ));
    }
}

#[derive(Debug, Reflect, PartialEq)]
pub enum GameActivity {
    CollectTaxes,
    PopulationExpansion,
    Census,
    ShipConstruction,
    Movement,
    Conflict,
    CityConstruction,
    RemoveSurplusPopulation,
    CheckCitySupport,
    AcquireTradeCards,
    Trade,
    ResolveCalamities,
    AcquireCivilizationCards,
    MoveSuccessionMarkers,
}

#[derive(Event, Debug, Reflect)]
pub struct GameActivityStarted(pub GameActivity);

#[derive(Event, Debug, Reflect)]
pub struct GameActivityEnded(pub GameActivity);

#[derive(Event, Debug)]
pub struct StartHandleSurplusPopulationEvent;

#[derive(Event, Debug)]
pub struct MoveTokensFromStockToAreaCommand {
    pub population_entity: Entity,
    pub stock_entity: Entity,
    pub number_of_tokens: usize,
}

#[derive(Event, Debug)]
pub struct MoveTokenFromAreaToAreaCommand {
    pub from_area: Entity,
    pub to_area: Entity,
    pub tokens: Vec<Entity>,
}

#[derive(Component, Debug)]
pub struct Area {
    pub max_population: u8,
}

#[derive(Component, Debug)]
pub struct Stock {
    pub max_tokens: usize,
}

#[derive(Component, Debug)]
pub struct Population;

#[derive(Component, Debug)]
pub struct StartArea;

#[derive(Component, Debug, Reflect)]
pub struct Token {
    pub player: Entity,
}

#[derive(Component, Debug)]
pub struct CannotAutoExpandPopulation;

fn setup_game(
    mut commands: Commands,
) {
    // Create Player
    let player = commands
        .spawn(
            (
                Player {},
                Name::new("Player"),
                Census { population: 0 }
            )
        )
        .id();

    let stock = commands.spawn((Stock { max_tokens: 47 }, Name::new("Stock"))).id();

    commands.entity(player).add_child(stock);

    for _n in 0..47 {
        let token = commands.spawn((Name::new("Token"), Token { player })).id();
        commands.entity(stock).add_child(token);
    }

    // Create some Areas
    commands
        .spawn((
            Name::new("Start Area"),
            Area { max_population: 2 },
            StartArea {}))
        .with_children(|c| { c.spawn((Name::new("Population"), Population {})); });
    commands
        .spawn((Area { max_population: 3 }, Name::new("Area two")))
        .with_children(|c| { c.spawn((Name::new("Population"), Population {})); });
    commands
        .spawn((Area { max_population: 1 }, Name::new("Area three")))
        .with_children(|c| { c.spawn((Name::new("Population"), Population {})); });
    commands
        .spawn((Area { max_population: 5 }, Name::new("Area four")))
        .with_children(|c| { c.spawn((Name::new("Population"), Population {})); });
}

fn move_token_from_area_to_area(
    mut move_events: EventReader<MoveTokenFromAreaToAreaCommand>,
    mut commands: Commands,
) {
    for ev in move_events.read() {
        commands.entity(ev.from_area).remove_children(&ev.tokens);
        commands.entity(ev.to_area).push_children(&ev.tokens);
    }
}

/***
A system that checks if an area has children... I mean, this is completely unnecessary really
 */

/**
This is 100% needed to be able to test expansion and stuff.
*/
fn move_tokens_from_stock_to_area(
    mut move_commands: EventReader<MoveTokensFromStockToAreaCommand>,
    player_stock_query: Query<&Children, With<Stock>>,
    player_query: Query<&Children>,
    mut commands: Commands,
) {
    for ev in move_commands.read() {
        if let Ok(children) = player_query.get(ev.stock_entity) {
            for child in children {
                if let Ok(tokens) = player_stock_query.get(*child) {
                    let tokens_to_move = &tokens.into_iter().as_slice()[0..ev.number_of_tokens];
                    commands.entity(ev.stock_entity).remove_children(tokens_to_move);
                    commands.entity(ev.population_entity).push_children(tokens_to_move);
                }
            }
        }
    }
}