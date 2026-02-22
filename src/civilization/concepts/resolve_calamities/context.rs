use bevy::prelude::{Component, Entity, Reflect, ReflectComponent};
use crate::civilization::TradeCard;

#[derive(Debug, Clone, Reflect)]
pub struct CalamityContext {
    pub calamity: TradeCard,
    pub primary_victim: Entity,
    pub traded_by: Option<Entity>,
}

impl CalamityContext {
    pub fn new(calamity: TradeCard, primary_victim: Entity, traded_by: Option<Entity>) -> Self {
        Self {
            calamity,
            primary_victim,
            traded_by,
        }
    }
    
    pub fn is_immune_to_secondary(&self, player: Entity) -> bool {
        self.traded_by == Some(player)
    }
}

#[derive(Debug, Clone, Default, Reflect)]
pub struct CalamityEffects {
    pub unit_points_to_lose: i32,
    pub cities_to_destroy: usize,
    pub cities_to_reduce: usize,
    pub secondary_effects: Vec<SecondaryEffect>,
    pub special_effect: Option<SpecialEffect>,
}

impl CalamityEffects {
    pub fn with_unit_point_loss(mut self, amount: i32) -> Self {
        self.unit_points_to_lose = amount;
        self
    }
    
    pub fn with_cities_destroyed(mut self, count: usize) -> Self {
        self.cities_to_destroy = count;
        self
    }
    
    pub fn with_cities_reduced(mut self, count: usize) -> Self {
        self.cities_to_reduce = count;
        self
    }
    
    pub fn with_secondary_effect(mut self, effect: SecondaryEffect) -> Self {
        self.secondary_effects.push(effect);
        self
    }
    
    pub fn with_special_effect(mut self, effect: SpecialEffect) -> Self {
        self.special_effect = Some(effect);
        self
    }
}

#[derive(Debug, Clone, Reflect)]
pub struct SecondaryEffect {
    pub victim: Entity,
    pub unit_points_to_lose: i32,
}

impl SecondaryEffect {
    pub fn new(victim: Entity, unit_points: i32) -> Self {
        Self {
            victim,
            unit_points_to_lose: unit_points,
        }
    }
}

#[derive(Debug, Clone, Reflect)]
pub enum SpecialEffect {
    VolcanoEruption { volcano_area: Entity },
    Earthquake { city_to_destroy: Entity, city_to_reduce: Option<Entity> },
    Treachery { city_to_replace: Entity, beneficiary: Option<Entity> },
    CivilWar(CivilWarEffect),
    SlaveRevolt { cities_to_reduce: Vec<Entity> },
    Flood { flood_plain_area: Entity },
    BarbarianHordes { areas_to_attack: Vec<Entity> },
    Piracy { coastal_cities: Vec<Entity>, beneficiary: Entity },
}

#[derive(Debug, Clone, Reflect)]
pub struct CivilWarEffect {
    pub beneficiary: Entity,
    pub victim_selection_points: usize,
    pub beneficiary_selection_points: usize,
}

impl CivilWarEffect {
    pub fn new(beneficiary: Entity, victim_selection: usize) -> Self {
        Self {
            beneficiary,
            victim_selection_points: victim_selection,
            beneficiary_selection_points: 20,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum CalamityPhase {
    ComputeEffects,
    AwaitingInput,
    ApplyEffects,
    Resolved,
}

#[derive(Debug, Clone, Reflect)]
pub enum CalamityInputRequest {
    SelectUnitsToLose {
        player: Entity,
        amount: usize,
        exact: bool,
    },
    SelectCitiesToReduce {
        player: Entity,
        count: usize,
    },
    SelectCitiesToDestroy {
        player: Entity,
        count: usize,
    },
    SelectSecondaryVictims {
        selector: Entity,
        total_points: usize,
        max_per_victim: usize,
        immune_players: Vec<Entity>,
    },
    CivilWarVictimSelection {
        victim: Entity,
        points_to_select: usize,
    },
    CivilWarBeneficiarySelection {
        beneficiary: Entity,
        victim: Entity,
        points_to_select: usize,
    },
}

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct ActiveCalamityResolution {
    pub context: CalamityContext,
    pub effects: CalamityEffects,
    pub phase: CalamityPhase,
    pub pending_input: Option<CalamityInputRequest>,
}

impl ActiveCalamityResolution {
    pub fn new(context: CalamityContext) -> Self {
        Self {
            context,
            effects: CalamityEffects::default(),
            phase: CalamityPhase::ComputeEffects,
            pending_input: None,
        }
    }
    
    pub fn with_effects(mut self, effects: CalamityEffects) -> Self {
        self.effects = effects;
        self
    }
    
    pub fn request_input(&mut self, request: CalamityInputRequest) {
        self.pending_input = Some(request);
        self.phase = CalamityPhase::AwaitingInput;
    }
    
    pub fn advance_to_apply(&mut self) {
        self.pending_input = None;
        self.phase = CalamityPhase::ApplyEffects;
    }
    
    pub fn mark_resolved(&mut self) {
        self.phase = CalamityPhase::Resolved;
    }
}
