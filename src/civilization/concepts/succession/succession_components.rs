use crate::civilization::enums::GameFaction;
use bevy::platform::collections::HashMap;
use bevy::prelude::{Component, Reflect, ReflectComponent, Resource};
use serde::{Deserialize, Serialize};

/// Final standings after the game ends (rule 35), computed by `determine_winner`
/// and read by the victory screen. Sorted best-first; the winner is `[0]`.
#[derive(Resource, Debug, Clone, Default)]
pub struct GameResult {
    /// `(player name, total score, A.S.T. space)`, highest score first.
    pub standings: Vec<(String, u32, u32)>,
}

/// Number of spaces on the A.S.T., indices `0..=AST_FINISH`.
pub const AST_FINISH: u32 = 16;
/// Total count of spaces (START + 16).
pub const AST_LEN: usize = (AST_FINISH as usize) + 1;

/// A player's marker location on the A.S.T.
///
/// `space` is **0-indexed**: `0` is START, `AST_FINISH` (16) is FINISH. This
/// matches the "AST" sheet in `docs/ast.xslx` (row 8, `POS->`).
#[derive(Component, Debug, Clone, Serialize, Deserialize, Default, Reflect)]
#[reflect(Component)]
pub struct AstPosition {
    pub space: u32,
}

impl AstPosition {
    pub fn new(space: u32) -> Self {
        Self { space }
    }

    pub fn epoch(&self) -> AstEpoch {
        AstEpoch::for_space(self.space)
    }

    pub fn is_finished(&self) -> bool {
        self.space >= AST_FINISH
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum AstEpoch {
    StoneAge,
    EarlyBronze,
    LateBronze,
    EarlyIron,
    LateIron,
}

impl AstEpoch {
    /// Epoch that a given 0-indexed space belongs to (rule 33.2 / sheet bands).
    /// START (0) shares the Stone Age's (zero) requirements.
    pub fn for_space(space: u32) -> Self {
        match space {
            0..=4 => AstEpoch::StoneAge,
            5..=7 => AstEpoch::EarlyBronze,
            8..=10 => AstEpoch::LateBronze,
            11..=13 => AstEpoch::EarlyIron,
            _ => AstEpoch::LateIron,
        }
    }

    /// Human-readable name for UI/logging.
    pub fn name(&self) -> &'static str {
        match self {
            AstEpoch::StoneAge => "Stone Age",
            AstEpoch::EarlyBronze => "Early Bronze Age",
            AstEpoch::LateBronze => "Late Bronze Age",
            AstEpoch::EarlyIron => "Early Iron Age",
            AstEpoch::LateIron => "Late Iron Age",
        }
    }

    /// Short entry-criteria label for UI, mirroring the sheet header (row 7).
    pub fn criteria_label(&self) -> &'static str {
        match self {
            AstEpoch::StoneAge => "",
            AstEpoch::EarlyBronze => "2 cities",
            AstEpoch::LateBronze => "3 cities; 3 groups",
            AstEpoch::EarlyIron => "4 cities; 9 cards, 5 groups",
            AstEpoch::LateIron => "5 cities; points",
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

    /// Minimum distinct civ card colour groups required to enter this epoch (rule 33.2).
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

/// One space on the A.S.T. board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AstSpace {
    pub index: u32,
    pub epoch: AstEpoch,
    pub is_start: bool,
    pub is_finish: bool,
}

/// The static A.S.T. board definition (rule 33).
///
/// `spaces` holds the standard 17-space layout used by all civilizations.
/// `overrides` lets individual factions diverge (different age lengths / Late-Iron
/// point values) — empty for now; see `docs/ast-design.md` §4.
#[derive(Resource, Debug, Clone)]
pub struct AstTrack {
    pub spaces: Vec<AstSpace>,
    pub overrides: HashMap<GameFaction, Vec<AstSpace>>,
}

impl AstTrack {
    /// The standard layout: START at 0, FINISH at `AST_FINISH`, bands per the sheet.
    pub fn standard() -> Self {
        let spaces = (0..=AST_FINISH)
            .map(|index| AstSpace {
                index,
                epoch: AstEpoch::for_space(index),
                is_start: index == 0,
                is_finish: index == AST_FINISH,
            })
            .collect();
        Self {
            spaces,
            overrides: HashMap::default(),
        }
    }

    /// Spaces for a given faction, honouring any per-faction override.
    pub fn spaces_for(&self, faction: GameFaction) -> &[AstSpace] {
        self.overrides
            .get(&faction)
            .map(|v| v.as_slice())
            .unwrap_or(&self.spaces)
    }

    /// The FINISH index for a faction (last space).
    pub fn finish_index(&self, faction: GameFaction) -> u32 {
        self.spaces_for(faction)
            .last()
            .map(|s| s.index)
            .unwrap_or(AST_FINISH)
    }
}

impl Default for AstTrack {
    fn default() -> Self {
        Self::standard()
    }
}
