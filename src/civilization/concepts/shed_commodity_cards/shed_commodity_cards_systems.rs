use crate::GameActivity;
use crate::civilization::concepts::acquire_trade_cards::{
    CivilizationTradeCards, PlayerTradeCards, TradeCard, TradeCardTrait,
};
use crate::civilization::concepts::shed_commodity_cards::shed_commodity_cards_components::*;
use crate::civilization::concepts::shed_commodity_cards::shed_commodity_cards_ui_components::*;
use crate::civilization::cheapest_cards_to_give;
use crate::stupid_ai::IsHuman;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

/// House rule (not in the base Advanced Civilization rulebook): a player's
/// commodity hand may not exceed this many cards once the Trade phase ends.
pub const COMMODITY_CARD_HAND_LIMIT: usize = 8;

/// Marks every player whose commodity hand exceeds [`COMMODITY_CARD_HAND_LIMIT`]
/// with how many cards they must discard. Calamity cards don't count toward
/// the limit -- they're already capped per-turn by rule 29.5.
pub fn start_shed_commodity_cards(mut commands: Commands, players: Query<(Entity, &PlayerTradeCards)>) {
    for (player, hand) in players.iter() {
        let total: usize = hand.commodity_cards().values().sum();
        if total > COMMODITY_CARD_HAND_LIMIT {
            let must_discard = total - COMMODITY_CARD_HAND_LIMIT;
            commands
                .entity(player)
                .insert(NeedsToShedCommodityCards { must_discard });
        }
    }
}

/// Drives commodity shedding to completion and owns the phase's exit, mirroring
/// `apply_monotheism_conversions`: AI holders are resolved immediately; a human
/// holder gets `AwaitingCommodityShedSelection` and the UI populates
/// `CommodityShedSelectionState`, and this system applies the result once the
/// UI clears that marker.
pub fn apply_commodity_shed(
    mut commands: Commands,
    mut shedders: Query<(Entity, &mut PlayerTradeCards, &NeedsToShedCommodityCards, Has<IsHuman>)>,
    awaiting_query: Query<Entity, With<AwaitingCommodityShedSelection>>,
    mut shed_state: ResMut<CommodityShedSelectionState>,
    mut trade_card_piles: ResMut<CivilizationTradeCards>,
    names: Query<&Name>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    if shedders.is_empty() && awaiting_query.is_empty() {
        next_state.set(GameActivity::ResolveCalamities);
        return;
    }

    for (player_entity, mut hand, needs, is_human) in shedders.iter_mut() {
        let is_waiting = awaiting_query.get(player_entity).is_ok();
        let must_discard = needs.must_discard;
        let player_label = names.get(player_entity).map_or_else(
            |_| format!("{player_entity:?}"),
            std::string::ToString::to_string,
        );

        if is_human {
            if is_waiting {
                // Still deciding.
                continue;
            }
            if shed_state.player == Some(player_entity) {
                // Human just confirmed (AwaitingCommodityShedSelection removed by button handler).
                let chosen = shed_state.take_result();
                discard_commodity_cards(&mut hand, &chosen, &mut trade_card_piles);
                commands
                    .entity(player_entity)
                    .remove::<NeedsToShedCommodityCards>();
                info!(
                    "[SHED] {} discarded {} commodity card(s): {:?}",
                    player_label,
                    chosen.values().sum::<usize>(),
                    chosen
                );
            } else if shed_state.player.is_none() {
                // First time for this human: set up UI.
                let holdings: Vec<(TradeCard, usize)> = hand
                    .cards_with_counts()
                    .into_iter()
                    .filter(|(card, _)| card.is_commodity())
                    .collect();
                shed_state.populate(player_entity, must_discard, holdings);
                commands
                    .entity(player_entity)
                    .insert(AwaitingCommodityShedSelection);
                info!(
                    "[SHED] Human player {} must discard {} commodity card(s)",
                    player_label, must_discard
                );
            }
        } else {
            // AI: favor keeping high-value cards and sets by shedding
            // whichever individual card is currently cheapest to give up
            // (`value_of_giving`) -- this discourages breaking up a set
            // (giving up the Nth copy of a stack costs `value * (2n-1)`, so a
            // large stack is expensive to break) more than it discourages
            // giving up a low-value singleton.
            let chosen = cheapest_cards_to_give(&hand.commodity_cards(), must_discard);
            discard_commodity_cards(&mut hand, &chosen, &mut trade_card_piles);
            commands
                .entity(player_entity)
                .remove::<NeedsToShedCommodityCards>();
            info!(
                "[SHED] {} (AI) discarded {} commodity card(s): {:?}",
                player_label,
                chosen.values().sum::<usize>(),
                chosen
            );
        }
    }
}

fn discard_commodity_cards(
    hand: &mut PlayerTradeCards,
    chosen: &HashMap<TradeCard, usize>,
    piles: &mut CivilizationTradeCards,
) {
    for (&card, &count) in chosen {
        if count == 0 {
            continue;
        }
        if hand.remove_n_trade_cards(count, card).is_some() {
            piles
                .card_piles
                .entry(card.value())
                .or_default()
                .extend(std::iter::repeat(card).take(count));
        }
    }
}
