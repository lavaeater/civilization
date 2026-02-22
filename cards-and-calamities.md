# Cards and calamities

Analysis of Current Implementation
Your current resolve_volcano_earthquake system is monolithic - it handles all the logic inline:

Determining volcano vs earthquake
Finding affected areas
Calculating damage
Applying effects
This works but doesn't scale to calamities like Civil War which requires:

Multi-step player interaction (victim selects 15+ units, beneficiary selects 20)
Civilization card modifiers (Music +5, Drama +5, Democracy +10)
Transfer of units to another player
Proposed Architecture: Calamity Resolution Pipeline
The key insight is that calamities share common patterns but differ in their phases and modifiers:

Core Abstractions
rust
/// Describes what a calamity needs to resolve
pub struct CalamityContext {
pub calamity: TradeCard,
pub primary_victim: Entity,
pub traded_by: Option<Entity>,  // immune to secondary effects
pub phase: CalamityPhase,
}

pub enum CalamityPhase {
/// Calculate base effects, apply civ card modifiers
ComputeEffects,
/// Player must make choices (select cities, units, victims)
AwaitingPlayerInput { input_type: CalamityInputType },
/// Apply computed effects to the world
ApplyEffects,
/// Done
Resolved,
}

pub enum CalamityInputType {
SelectUnitsToLose { amount: usize, from_player: Entity },
SelectCitiesToReduce { count: usize },
SelectSecondaryVictims { count: usize, max_per_victim: usize },
SelectUnitsForFaction { amount: usize, selector: Entity },  // Civil War
// etc.
}
Modifier System
Instead of hardcoding civ card effects, use a modifier query pattern:

rust
/// Computed effects after applying all modifiers
pub struct CalamityEffects {
pub unit_point_loss: i32,
pub cities_destroyed: usize,
pub cities_reduced: usize,
pub secondary_victims: Vec<SecondaryVictimEffect>,
pub special: Option<SpecialEffect>,  // Civil War faction split, etc.
}

/// Trait for civ cards that modify calamities
pub trait CalamityModifier {
fn modifies(&self, calamity: TradeCard) -> bool;
fn apply(&self, effects: &mut CalamityEffects, context: &CalamityContext);
}
For example, Engineering for Earthquake:

rust
impl CalamityModifier for EngineeringModifier {
fn modifies(&self, calamity: TradeCard) -> bool {
calamity == TradeCard::VolcanoEarthquake
}

    fn apply(&self, effects: &mut CalamityEffects, ctx: &CalamityContext) {
        // Earthquake reduces instead of destroys
        if !ctx.is_volcano {
            effects.cities_destroyed = 0;
            effects.cities_reduced += 1;
        }
        // Also grants immunity as secondary victim
    }
}
Civil War Example
Civil War is the most complex. Here's how it would decompose:

rust
pub struct CivilWarState {
pub beneficiary: Entity,           // Player with most units in stock
pub victim_selection: usize,       // Base 15, modified by cards
pub beneficiary_selection: usize,  // Always 20
pub victim_selected: Vec<Entity>,  // Units selected by victim
pub phase: CivilWarPhase,
}

pub enum CivilWarPhase {
DetermineBeneficiary,
VictimSelectsUnits,
BeneficiarySelectsUnits,
TransferFaction,
}

fn compute_civil_war_victim_selection(civ_cards: &PlayerCivilizationCards) -> usize {
let mut base = 15;
if civ_cards.owns(&CivCardName::Music) { base += 5; }
if civ_cards.owns(&CivCardName::DramaAndPoetry) { base += 5; }
if civ_cards.owns(&CivCardName::Democracy) { base += 10; }
base
}
State Machine Approach
Each calamity becomes a state machine with typed states:

rust
#[derive(Component)]
pub enum ResolvingCalamity {
VolcanoEarthquake(VolcanoEarthquakeState),
Treachery(TreacheryState),
Famine(FamineState),
Superstition(SuperstitionState),
CivilWar(CivilWarState),
SlaveRevolt(SlaveRevoltState),
Flood(FloodState),
BarbarianHordes(BarbarianHordesState),
Epidemic(EpidemicState),
CivilDisorder(CivilDisorderState),
IconoclasmAndHeresy(IconoclasmAndHeresyState),
Piracy(PiracyState),
}
Each variant holds its own state, and systems match on the variant to advance it.

Recommended File Structure
resolve_calamities/
├── mod.rs
├── plugin.rs
├── context.rs           # CalamityContext, CalamityEffects
├── modifiers.rs         # Civ card modifier system
├── calamities/
│   ├── mod.rs
│   ├── volcano_earthquake.rs
│   ├── treachery.rs
│   ├── famine.rs
│   ├── superstition.rs
│   ├── civil_war.rs     # Complex multi-phase
│   ├── slave_revolt.rs
│   ├── flood.rs
│   ├── barbarian_hordes.rs
│   ├── epidemic.rs
│   ├── civil_disorder.rs
│   ├── iconoclasm_heresy.rs
│   └── piracy.rs
└── ui/                  # Player selection UI for interactive phases
└── calamity_selection_ui.rs
Key Design Decisions
Aspect	Recommendation
Civ card modifiers	Query PlayerCivilizationCards at resolution time, compute effects dynamically
Multi-phase calamities	Use component-based state machines, one system per phase transition
Player input	Spawn UI, wait for CalamityInputComplete message with selections
Secondary victims	Store immune_players: Vec<Entity> (the trader) in context
Unit point accounting	Reusable UnitPointCalculator that handles cities=5, tokens=1, exact-or-exceed rule
