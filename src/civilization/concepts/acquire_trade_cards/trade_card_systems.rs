use crate::civilization::components::Faction;
use crate::civilization::components::{PlayerCities, TokenStock, Treasury};
use crate::civilization::concepts::acquire_trade_cards::trade_card_components::{
    CivilizationTradeCards, PlayerTradeCards,
};
use crate::civilization::concepts::acquire_trade_cards::trade_card_events::{
    CheckIfWeCanTrade, HumanPlayerTradeCardsUpdated,
};
use crate::civilization::plugins::DebugOptions;
use crate::stupid_ai::IsHuman;
use crate::GameActivity;
use bevy::prelude::{debug, info, Entity, MessageReader, MessageWriter, Has, NextState, Query, Res, ResMut};

/// The ninth trade-card pile holds Gold, Ivory and Piracy shuffled together
/// (all three have `TradeCard::value() == 9`), so "the ninth stack" in rule
/// 27.5 is exactly `CivilizationTradeCards::pull_card_from(NINTH_STACK_PILE)`.
pub const NINTH_STACK_PILE: usize = 9;
/// Rule 27.51: 18 tokens from treasury per card bought from the ninth stack.
pub const NINTH_STACK_COST: usize = 18;

/// Rule 27.51: buy up to `max_cards` cards from the ninth (Gold/Ivory/Piracy)
/// stack at `NINTH_STACK_COST` tokens from treasury each; spent tokens are
/// returned to stock. Stops early (without spending) once the treasury can no
/// longer afford a card or the stack is empty. Returns how many were bought.
pub fn buy_from_ninth_stack(
    treasury: &mut Treasury,
    token_stock: &mut TokenStock,
    trade_card_resource: &mut CivilizationTradeCards,
    player_trade_cards: &mut PlayerTradeCards,
    max_cards: usize,
) -> usize {
    let mut bought = 0;
    for _ in 0..max_cards {
        if treasury.tokens_in_treasury() < NINTH_STACK_COST {
            break;
        }
        let Some(card) = trade_card_resource.pull_card_from(NINTH_STACK_PILE) else {
            break;
        };
        for _ in 0..NINTH_STACK_COST {
            if let Some(token) = treasury.remove_token_from_treasury() {
                token_stock.return_token_to_stock(token);
            }
        }
        player_trade_cards.add_trade_card(card);
        bought += 1;
    }
    bought
}

pub fn acquire_trade_cards(
    mut player_query: Query<(
        Entity,
        &Faction,
        &PlayerCities,
        &mut PlayerTradeCards,
        &mut Treasury,
        &mut TokenStock,
        Has<IsHuman>,
    )>,
    mut trade_card_resource: ResMut<CivilizationTradeCards>,
    mut check_if_we_can_trade: MessageWriter<CheckIfWeCanTrade>,
    mut pulled_card_event_writer: MessageWriter<HumanPlayerTradeCardsUpdated>,
    debug_options: Res<DebugOptions>,
) {
    info!("[TRADE_CARDS] Starting acquire trade cards phase");
    let mut total_players = 0;
    let mut players_with_cities = 0;
    
    for (player_entity, faction, player_cities, mut player_trade_cards, mut treasury, mut token_stock, is_human) in player_query
        .iter_mut()
        .sort_by::<&PlayerCities>(|v1, v2| v1.number_of_cities().cmp(&v2.number_of_cities()))
    {
        total_players += 1;
        let num_cities = player_cities.number_of_cities();
        if num_cities > 0 {
            players_with_cities += 1;
        }
        
        let mut pulled_cards = false;
        if is_human && debug_options.human_always_pulls_trade_cards {
            (1..=num_cities + 1).for_each(|pile| {
                if let Some(pulled_card) = trade_card_resource.pull_card_from(pile) {
                    pulled_cards = true;
                    player_trade_cards.add_trade_card(pulled_card);
                } else {
                    debug!("No more trade cards in pile: {}", pile);
                }
            });
        } else if !is_human && debug_options.ai_always_pulls_trade_cards {
            debug!("AI ALWAYS PULLS TRADE CARDS");
            (1..=num_cities + 3).for_each(|pile| {
                if let Some(pulled_card) = trade_card_resource.pull_card_from(pile) {
                    pulled_cards = true;
                    player_trade_cards.add_trade_card(pulled_card);
                } else {
                    debug!("No more trade cards in pile: {}", pile);
                }
            });
        } else {
            (1..=num_cities).for_each(|pile| {
                if let Some(pulled_card) = trade_card_resource.pull_card_from(pile) {
                    pulled_cards = true;
                    player_trade_cards.add_trade_card(pulled_card);
                } else {
                    debug!("No more trade cards in pile: {}", pile);
                }
            });
        }
        if is_human && pulled_cards {
            pulled_card_event_writer.write(HumanPlayerTradeCardsUpdated::new(player_entity));
        }
        if pulled_cards {
            info!(
                "[TRADE_CARDS] {} ({}) pulled cards, can_trade={}",
                faction.faction,
                if is_human { "human" } else { "AI" },
                player_trade_cards.can_trade()
            );
        }

        // Rule 27.51: immediately after this player collects their normal
        // cards (above), and before the next player collects theirs, they
        // may buy from the ninth (Gold/Ivory/Piracy) stack. No human UI for
        // this decision yet (tracked in docs/outline.md, same pattern as the
        // Coinage-rate gap) -- for now AI auto-buys at most one card per turn
        // when it can afford it, a deliberately conservative placeholder
        // policy pending real AI strategy (see the "Improved AI" item).
        if !is_human {
            let bought = buy_from_ninth_stack(
                &mut treasury,
                &mut token_stock,
                &mut trade_card_resource,
                &mut player_trade_cards,
                1,
            );
            if bought > 0 {
                info!(
                    "[TRADE_CARDS] {} bought {} card(s) from the ninth stack (rule 27.51)",
                    faction.faction, bought
                );
            }
        }
    }
    
    info!("[TRADE_CARDS] Processed {} players, {} have cities, sending CheckIfWeCanTrade", total_players, players_with_cities);
    check_if_we_can_trade.write(CheckIfWeCanTrade);
}

pub fn transition_to_trade(
    mut check_if_we_can_trade: MessageReader<CheckIfWeCanTrade>,
    players_can_trade_query: Query<(&PlayerTradeCards, Has<IsHuman>)>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    for _ in check_if_we_can_trade.read() {
        let can_trade_count = players_can_trade_query
            .iter()
            .filter(|(trade, _)| trade.can_trade())
            .count();
        
        info!("[TRADE_CARDS] CheckIfWeCanTrade received: {} players can trade", can_trade_count);
        
        if can_trade_count >= 2 {
            info!("[TRADE_CARDS] Transitioning to Trade phase");
            next_state.set(GameActivity::Trade);
        } else {
            info!("[TRADE_CARDS] Not enough traders, transitioning to CollectTaxes");
            next_state.set(GameActivity::CollectTaxes);
        }
    }
}
