use crate::civilization::{ActivityDisplay, CensusDisplay, PlayerInfoDisplay, TradeCardList};
use crate::GameActivity;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use lava_ui_builder::{Collapsible, CollapsibleContent};

pub struct CivilizationInputPlugin;

impl Plugin for CivilizationInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EnhancedInputPlugin)
            .add_input_context::<HudContext>()
            .add_systems(OnEnter(GameActivity::StartGame), spawn_hud_context)
            .add_observer(toggle_player_info)
            .add_observer(toggle_trade_cards)
            .add_observer(toggle_game_state)
            .add_observer(toggle_activity_card);
    }
}

// ── HUD context ───────────────────────────────────────────────────────────────

/// Always-active input context for HUD keyboard shortcuts (F1-F4).
/// Spawned once at `StartGame` and never despawned.
#[derive(Component)]
pub struct HudContext;

fn spawn_hud_context(mut commands: Commands) {
    commands.spawn((
        HudContext,
        actions!(HudContext[
            (Action::<TogglePlayerInfoCard>::new(), bindings![KeyCode::F1]),
            (Action::<ToggleTradeCardsCard>::new(),  bindings![KeyCode::F2]),
            (Action::<ToggleGameStateCard>::new(),   bindings![KeyCode::F3]),
            (Action::<ToggleActivityCard>::new(),    bindings![KeyCode::F4]),
        ]),
    ));
}

// ── HUD toggle observers ──────────────────────────────────────────────────────

fn toggle_player_info(
    _: On<Fire<TogglePlayerInfoCard>>,
    content: Query<&CollapsibleContent, With<PlayerInfoDisplay>>,
    mut collapsibles: Query<&mut Collapsible>,
) {
    if let Ok(c) = content.single()
        && let Ok(mut coll) = collapsibles.get_mut(c.parent)
    {
        coll.collapsed = !coll.collapsed;
    }
}

fn toggle_trade_cards(
    _: On<Fire<ToggleTradeCardsCard>>,
    content: Query<&CollapsibleContent, With<TradeCardList>>,
    mut collapsibles: Query<&mut Collapsible>,
) {
    if let Ok(c) = content.single()
        && let Ok(mut coll) = collapsibles.get_mut(c.parent)
    {
        coll.collapsed = !coll.collapsed;
    }
}

fn toggle_game_state(
    _: On<Fire<ToggleGameStateCard>>,
    content: Query<&CollapsibleContent, With<CensusDisplay>>,
    mut collapsibles: Query<&mut Collapsible>,
) {
    if let Ok(c) = content.single()
        && let Ok(mut coll) = collapsibles.get_mut(c.parent)
    {
        coll.collapsed = !coll.collapsed;
    }
}

fn toggle_activity_card(
    _: On<Fire<ToggleActivityCard>>,
    content: Query<&CollapsibleContent, With<ActivityDisplay>>,
    mut collapsibles: Query<&mut Collapsible>,
) {
    if let Ok(c) = content.single()
        && let Ok(mut coll) = collapsibles.get_mut(c.parent)
    {
        coll.collapsed = !coll.collapsed;
    }
}

// ── HUD action types ──────────────────────────────────────────────────────────

#[derive(InputAction)]
#[action_output(bool)]
pub struct TogglePlayerInfoCard;

#[derive(InputAction)]
#[action_output(bool)]
pub struct ToggleTradeCardsCard;

#[derive(InputAction)]
#[action_output(bool)]
pub struct ToggleGameStateCard;

#[derive(InputAction)]
#[action_output(bool)]
pub struct ToggleActivityCard;

// ── Shared navigation action types (used by phase contexts in Step 5) ─────────

#[derive(InputAction)]
#[action_output(bool)]
pub struct Confirm;

#[derive(InputAction)]
#[action_output(bool)]
pub struct Cancel;

#[derive(InputAction)]
#[action_output(bool)]
pub struct NavigateNext;

#[derive(InputAction)]
#[action_output(bool)]
pub struct NavigatePrev;

#[derive(InputAction)]
#[action_output(bool)]
pub struct IncrementValue;

#[derive(InputAction)]
#[action_output(bool)]
pub struct DecrementValue;

#[derive(InputAction)]
#[action_output(bool)]
pub struct ToggleSelected;

// ── Phase context stubs (wired up incrementally in Step 5) ────────────────────

#[derive(Component)]
pub struct MovementInput;

#[derive(Component)]
pub struct AreaNavigationInput;

#[derive(Component)]
pub struct CityConstructionInput;

#[derive(Component)]
pub struct TradeCardsInventoryInput;

#[derive(Component)]
pub struct CitySelectionInput;

#[derive(Component)]
pub struct PopulationSelectionInput;

#[derive(Component)]
pub struct CivilizationCardsInput;
