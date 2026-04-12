use bevy::prelude::{Component, Reflect, ReflectComponent};
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Serialize, Deserialize, Default, Reflect)]
#[reflect(Component)]
pub struct AstPosition {
    /// 1-indexed position on the AST track (1 = start, higher = further along)
    pub space: u32,
}

impl AstPosition {
    pub fn new(space: u32) -> Self {
        Self { space }
    }

    pub fn epoch(&self) -> AstEpoch {
        AstEpoch::for_space(self.space)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AstEpoch {
    StoneAge,
    EarlyBronze,
    LateBronze,
    EarlyIron,
    LateIron,
}

impl AstEpoch {
    pub fn for_space(space: u32) -> Self {
        match space {
            0..=3 => AstEpoch::StoneAge,
            4..=6 => AstEpoch::EarlyBronze,
            7..=9 => AstEpoch::LateBronze,
            10..=12 => AstEpoch::EarlyIron,
            _ => AstEpoch::LateIron,
        }
    }

    /// Minimum city count required to enter (or remain in) this epoch.
    pub fn min_cities(&self) -> usize {
        match self {
            AstEpoch::StoneAge => 0,
            AstEpoch::EarlyBronze => 2,
            AstEpoch::LateBronze => 3,
            AstEpoch::EarlyIron => 4,
            AstEpoch::LateIron => 5,
        }
    }

    /// Minimum distinct civ card color groups required to enter this epoch (rule 33.2).
    pub fn min_card_groups(&self) -> usize {
        match self {
            AstEpoch::StoneAge | AstEpoch::EarlyBronze => 0,
            AstEpoch::LateBronze => 3,
            AstEpoch::EarlyIron | AstEpoch::LateIron => 5,
        }
    }

    /// Minimum total civ cards required to enter this epoch (rule 33.2).
    pub fn min_card_count(&self) -> usize {
        match self {
            AstEpoch::StoneAge | AstEpoch::EarlyBronze | AstEpoch::LateBronze => 0,
            AstEpoch::EarlyIron | AstEpoch::LateIron => 9,
        }
    }
}
