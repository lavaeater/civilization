use bevy::prelude::*;

/// A named archetype that fills in a [`Weights`] preset. Carried for display and
/// logging; the actual behaviour comes entirely from the weights it expands to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Reflect)]
pub enum Playstyle {
    /// The reasonable default — every knob around the middle.
    Balanced,
    /// Attacks, contests cities, keeps little in reserve.
    Warlord,
    /// Grabs land fast, spreads thin.
    Expansionist,
    /// Cities and upkeep; hard to dislodge.
    Builder,
    /// Farms trade cards and civ-card tech.
    Merchant,
    /// Minimal footprint, never overextends.
    Turtle,
}

impl Playstyle {
    pub const ALL: [Playstyle; 6] = [
        Playstyle::Balanced,
        Playstyle::Warlord,
        Playstyle::Expansionist,
        Playstyle::Builder,
        Playstyle::Merchant,
        Playstyle::Turtle,
    ];

    /// Parse from a (case-insensitive) string, e.g. for `DebugOptions`/env overrides.
    pub fn from_name(name: &str) -> Option<Playstyle> {
        match name.trim().to_ascii_lowercase().as_str() {
            "balanced" => Some(Playstyle::Balanced),
            "warlord" | "aggressive" => Some(Playstyle::Warlord),
            "expansionist" | "expansion" => Some(Playstyle::Expansionist),
            "builder" => Some(Playstyle::Builder),
            "merchant" | "trader" => Some(Playstyle::Merchant),
            "turtle" | "defensive" => Some(Playstyle::Turtle),
            _ => None,
        }
    }
}

/// The tunable knobs the scoring functions read. All in roughly `[0, 1]`, where
/// higher = "I care more about this". Hand-tuned for now; later these are exactly
/// what reinforcement learning could optimise (see `docs/reinforcement-learning.md`).
#[derive(Clone, Copy, Debug, Reflect)]
pub struct Weights {
    // expansion / economy
    /// Value of feeding population and expanding.
    pub growth: f32,
    /// Value of building and holding cities (tax + card income).
    pub city_income: f32,
    /// Value of grabbing empty / contested territory.
    pub expansion: f32,
    // map control / aggression
    /// Value of attacking enemies and weakening neighbours.
    pub aggression: f32,
    /// Penalty weight for leaving own areas/cities exposed.
    pub defense: f32,
    // card economy
    /// Eagerness to propose and accept trades.
    pub trade_drive: f32,
    /// Weight on offloading calamity risk and avoiding it.
    pub calamity_aversion: f32,
    /// Value of civ-card credits and AST progress.
    pub tech_focus: f32,
    /// 0 = cautious (keep reserves, avoid even fights), 1 = all-in.
    pub risk: f32,
}

impl Weights {
    pub const fn uniform(v: f32) -> Self {
        Weights {
            growth: v,
            city_income: v,
            expansion: v,
            aggression: v,
            defense: v,
            trade_drive: v,
            calamity_aversion: v,
            tech_focus: v,
            risk: v,
        }
    }
}

/// How the highest-utility move is chosen from the scored list.
#[derive(Clone, Copy, Debug, Reflect)]
pub enum Picker {
    /// Always take the highest score (ties broken randomly).
    Greedy,
    /// Sample proportional to `exp(score / temperature)`. Adds non-robotic variety
    /// and doubles as exploration if we ever log `(state, move)` for imitation.
    Softmax { temperature: f32 },
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct Personality {
    pub playstyle: Playstyle,
    pub picker: Picker,
    pub weights: Weights,
}

impl Personality {
    /// Build a personality from a named archetype.
    pub fn from_playstyle(playstyle: Playstyle) -> Self {
        let (weights, picker) = match playstyle {
            Playstyle::Balanced => (
                Weights::uniform(0.5),
                Picker::Softmax { temperature: 0.35 },
            ),
            Playstyle::Warlord => (
                Weights {
                    aggression: 0.95,
                    risk: 0.9,
                    expansion: 0.6,
                    city_income: 0.5,
                    defense: 0.2,
                    growth: 0.5,
                    trade_drive: 0.3,
                    calamity_aversion: 0.2,
                    tech_focus: 0.4,
                },
                Picker::Greedy,
            ),
            Playstyle::Expansionist => (
                Weights {
                    expansion: 0.95,
                    growth: 0.85,
                    aggression: 0.5,
                    city_income: 0.55,
                    defense: 0.25,
                    risk: 0.6,
                    trade_drive: 0.4,
                    calamity_aversion: 0.3,
                    tech_focus: 0.5,
                },
                Picker::Softmax { temperature: 0.3 },
            ),
            Playstyle::Builder => (
                Weights {
                    city_income: 0.95,
                    defense: 0.85,
                    growth: 0.7,
                    expansion: 0.5,
                    aggression: 0.2,
                    risk: 0.3,
                    trade_drive: 0.5,
                    calamity_aversion: 0.6,
                    tech_focus: 0.7,
                },
                Picker::Greedy,
            ),
            Playstyle::Merchant => (
                Weights {
                    trade_drive: 0.95,
                    tech_focus: 0.9,
                    city_income: 0.6,
                    growth: 0.6,
                    expansion: 0.5,
                    defense: 0.5,
                    aggression: 0.25,
                    calamity_aversion: 0.5,
                    risk: 0.4,
                },
                Picker::Softmax { temperature: 0.3 },
            ),
            Playstyle::Turtle => (
                Weights {
                    defense: 0.95,
                    calamity_aversion: 0.85,
                    city_income: 0.7,
                    growth: 0.5,
                    expansion: 0.25,
                    aggression: 0.1,
                    risk: 0.15,
                    trade_drive: 0.5,
                    tech_focus: 0.6,
                },
                Picker::Greedy,
            ),
        };
        Personality {
            playstyle,
            picker,
            weights,
        }
    }
}

impl Default for Personality {
    fn default() -> Self {
        Personality::from_playstyle(Playstyle::Balanced)
    }
}
