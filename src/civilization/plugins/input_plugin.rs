use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

struct InputPlugin {}

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DefaultPlugins, EnhancedInputPlugin))
            // .add_input_context::<AreaNavigationContext>()
            .add_input_context::<MovementInput>()
            .add_input_context::<GeneralInput>()
            .add_observer(apply_movement)
            .add_observer(attack)
            .add_observer(open_inventory)
            .add_observer(navigate_inventory)
            .add_observer(close_inventory)
            .add_systems(Startup, spawn);
    }
}
/// - General input handling
/// This is a context to be used with the input handling
/// When this context is active we will support Enter for choice
/// ESC for cancel etc.
#[derive(Component)]
struct GeneralInput;

fn spawn(mut commands: Commands) {
    commands.spawn(player_bundle());
}

fn apply_movement(movement: On<Fire<Movement>>) {
    info!("moving: {}", movement.value);
}

fn attack(_on: On<Fire<Attack>>) {
    info!("attacking");
}

fn open_inventory(open: On<Start<OpenTradeCardsInventory>>, mut commands: Commands) {
    info!("opening inventory");
    commands.entity(open.context).insert((
        ContextActivity::<AreaNavigationInput>::INACTIVE,
        ContextActivity::<TradeCardInventory>::ACTIVE,
    ));
}

fn navigate_inventory(_on: On<Fire<NavigateInventory>>) {
    info!("navigating inventory");
}

fn close_inventory(close: On<Start<CloseTradeCardInventory>>, mut commands: Commands) {
    info!("closing inventory");
    commands.entity(close.context).insert((
        ContextActivity::<AreaNavigationInput>::ACTIVE,
        ContextActivity::<TradeCardInventory>::INACTIVE,
    ));
}

fn player_bundle() -> impl Bundle {
    (
        AreaNavigationInput,
        actions!(AreaNavigationInput[
            (
                Action::<Movement>::new(),
                DeadZone::default(),
                Bindings::spawn((Cardinal::wasd_keys(), Axial::left_stick())),
            ),
            (
                Action::<Attack>::new(),
                bindings![MouseButton::Left, GamepadButton::West],
            ),
            (
                Action::<OpenTradeCardsInventory>::new(),
                // We set `require_reset` to `true` because `CloseInventory` action uses the same input,
                // and we want it to be triggerable only after the button is released.
                ActionSettings {
                    require_reset: true,
                    ..Default::default()
                },
                bindings![KeyCode::KeyI, GamepadButton::Select],
            ),
        ]),
        TradeCardInventory,
        actions!(TradeCardInventory[
            (
                Action::<NavigateInventory>::new(),
                Bindings::spawn((Cardinal::wasd_keys(), Axial::left_stick())),
                Pulse::new(0.2), // Avoid triggering every frame on hold for UI.
            ),
            (
                Action::<CloseTradeCardInventory>::new(),
                ActionSettings {
                    require_reset: true,
                    ..Default::default()
                },
                bindings![KeyCode::KeyI, GamepadButton::Select],
            )
        ]),
    )
}
/*
PrepareGame,
    StartGame,
    // CollectTaxes,
    PopulationExpansion,
    Census,
    // ShipConstruction,
    Movement,
    Conflict,
    CityConstruction,
    RemoveSurplusPopulation,
    CheckCitySupportAfterRemoveSurplusPopulation,
    AcquireTradeCards,
    Trade,
    ResolveCalamities,
    CheckCitySupportAfterResolveCalamities,
 */

#[derive(Component)]
struct AreaNavigationInput;

#[derive(Component)]
struct CityConstructionInput;

#[derive(Component)]
struct TradeCardsInventoryInput;

#[derive(Component)]
struct CitySelectionInput;

#[derive(Component)]
struct PopulationSelectionInput;

#[derive(Component)]
struct CivilizationCardsInput;

#[derive(Component)]
struct MovementInput;
