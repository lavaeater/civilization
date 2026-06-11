use super::saturating;
use crate::civilization::AcquireCivilizationCardsMove;
use crate::stupid_ai::Weights;

/// Pre-computed inputs for scoring one `AcquireCard` option, gathered in the system
/// from the card definition + the player's wealth (so the scorer stays pure).
#[derive(Clone, Copy, Debug, Default)]
pub struct CivCardOption {
    /// Cost after this player's existing credits/discounts.
    pub effective_cost: u32,
    /// Total future-discount this card hands out (credit synergy toward a tech line).
    pub credit_value: u32,
    /// The player's spendable commodity value (`total_stack_value`).
    pub wealth: u32,
}

/// Score a civilization-card-acquisition move. Cards are the win engine (credits,
/// AST progress), so `tech_focus` players buy eagerly; `risk` governs how far down
/// they spend their trade-card reserve.
pub fn score_civ_card(
    mv: &AcquireCivilizationCardsMove,
    option: Option<CivCardOption>,
    w: &Weights,
) -> f32 {
    match mv {
        // "Stop buying" baseline. Kept deliberately LOW: civ cards are the win
        // engine (A.S.T. progress, credits, calamity defence) and any commodity
        // cards we don't spend above 8 are surrendered at end of phase (rule
        // 31.71), so holding back is rarely worth it. Cautious / non-tech players
        // are only slightly more willing to stop.
        AcquireCivilizationCardsMove::DoneAcquiringCards => {
            0.18 + (1.0 - w.tech_focus) * 0.12 + (1.0 - w.risk) * 0.05
        }
        AcquireCivilizationCardsMove::AcquireCard(_)
        | AcquireCivilizationCardsMove::AcquireCards(_) => {
            let Some(opt) = option else {
                return f32::NEG_INFINITY;
            };
            if opt.wealth < opt.effective_cost {
                // Can't actually pay — keep it out of contention.
                return f32::NEG_INFINITY;
            }
            // Strong base for any affordable card so the AI actually invests in
            // its civilization, boosted by tech focus and by credit synergy
            // (cards that discount a whole tech line).
            let synergy = saturating(opt.credit_value as f32, 3.0);
            let mut score = 0.55 + 0.35 * w.tech_focus + 0.35 * synergy;
            // Mild reserve discipline: a cautious player dislikes blowing its whole
            // hand on one card. Small, because excess cards are lost anyway.
            let spend_fraction = if opt.wealth == 0 {
                1.0
            } else {
                (opt.effective_cost as f32 / opt.wealth as f32).clamp(0.0, 1.0)
            };
            score -= (1.0 - w.risk) * spend_fraction * 0.2;
            score
        }
    }
}
