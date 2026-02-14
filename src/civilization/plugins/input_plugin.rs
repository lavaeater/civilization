use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

struct InputPlugin {}

///
/// This plugin only enables input, all the 
/// systems under concepts have to have their own actual
/// systems for handling the actions, I think?
/// This plugin must also register all the actions,
/// it would seem, but the actual systems should probably be under
/// the concepts.
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EnhancedInputPlugin)
            .add_input_context::<MovementInput>();
            // .add_observer(apply_movement)
            // .add_observer(attack)
            // .add_observer(open_inventory)
            // .add_observer(navigate_inventory)
            // .add_observer(close_inventory)
            // ;
    }
}
/// - General input handling
/// This is a context to be used with the input handling
/// When this context is active we will support Enter for choice
/// ESC for cancel etc.
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
