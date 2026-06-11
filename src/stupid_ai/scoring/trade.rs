//! Personality knobs for the (countdown-driven) AI trade systems in
//! `concepts/trade`. Trade is hidden-information and multi-step, so rather than a
//! single move-score we expose small decision helpers the existing `ai_*` trade
//! systems consult. See `docs/utility-ai-design.md` §4 (Trade).

use crate::stupid_ai::Weights;

/// Extra stack-value gain (beyond break-even) an AI demands before accepting an
/// offer. Eager traders (high `trade_drive`) take marginal trades; reluctant ones
/// hold out for a clear profit.
pub fn trade_accept_margin(w: &Weights) -> f32 {
    // trade_drive 1.0 -> 0 margin (accept any non-losing trade)
    // trade_drive 0.0 -> require +3 stack value
    (1.0 - w.trade_drive) * 3.0
}

/// Whether this AI bothers to take a trade purely to offload a calamity card.
/// Calamity-averse players jump at it; the carefree ignore the risk.
pub fn accepts_calamity_offload(w: &Weights) -> bool {
    w.calamity_aversion >= 0.4
}

/// Probability [0,1] that the AI creates a fresh offer this tick. Merchants flood
/// the table with offers; turtles/warlords mostly sit out the trade phase.
pub fn offer_creation_chance(w: &Weights) -> f32 {
    (0.25 + 0.75 * w.trade_drive).clamp(0.0, 1.0)
}

/// Trade-phase countdown value below which this AI stops trading. The phase counts
/// down from 90; a higher threshold = stops sooner. Eager traders keep going.
pub fn stop_trading_threshold(w: &Weights) -> f32 {
    // trade_drive 1.0 -> 80 (trade until the last ~10s)
    // trade_drive 0.0 -> 88 (bail after ~2s)
    88.0 - 8.0 * w.trade_drive
}
