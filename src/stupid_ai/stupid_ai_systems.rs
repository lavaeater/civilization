use crate::stupid_ai::*;
use bevy::platform::collections::HashMap;
use bevy::prelude::{
    debug, error, warn, Commands, Entity, Has, MessageReader, MessageWriter, Name, Query, Res,
    ResMut,
};
use rand::prelude::SliceRandom;
use crate::civilization::*;

pub fn setup_stupid_ai(mut stupid_ai_event: MessageReader<StupidAiMessage>, mut commands: Commands) {
    for e in stupid_ai_event.read() {
        commands.entity(e.player).insert(StupidAi);
    }
}

pub fn assign_some_foolish_cards() {}

/// Clear the per-player movement selection counts when a new movement phase begins.
pub fn reset_movement_loop_guard(mut loop_guard: ResMut<MovementLoopGuard>) {
    loop_guard.counts.clear();
}

pub fn select_stupid_pop_exp(
    mut event_reader: MessageReader<SelectStupidMove>,
    player_moves: Query<(&Name, &AvailableMoves, &Personality)>,
    area_info_query: Query<(Entity, &Population, &LandPassage, Option<&BuiltCity>, Has<CitySite>)>,
    mut expand_writer: MessageWriter<ExpandPopulationManuallyCommand>,
    debug_options: Res<DebugOptions>,
) {
    for event in event_reader.read() {
        if let Ok((player_name, available_moves, personality)) = player_moves.get(event.player) {
            let areas = gather_area_summaries(event.player, &area_info_query);
            let scored: Vec<(usize, f32)> = available_moves
                .moves
                .iter()
                .map(|(i, m)| (*i, score_population_expansion(m, &areas, &personality.weights)))
                .collect();
            let mut rng = rand::rng();
            if let Some(chosen) = pick(&scored, personality.picker, &mut rng) {
                let selected_move = &available_moves.moves[&chosen];
                if debug_options.print_selected_moves {
                    debug!("{} selects {:#?}", player_name, selected_move);
                }
                match selected_move {
                    GameMove::PopulationExpansion(pop_exp_move) => {
                        expand_writer.write(ExpandPopulationManuallyCommand::new(
                            event.player,
                            pop_exp_move.area,
                            pop_exp_move.max_tokens,
                        ));
                    }
                    _ => {
                        debug!("In pop exp, move was: {:#?}", selected_move);
                    }
                }
            }
        }
    }
}

pub fn select_stupid_movement(
    mut event_reader: MessageReader<SelectStupidMove>,
    player_moves: Query<(&Name, &AvailableMoves, &PlayerAreas, &Personality)>,
    mut move_tokens_writer: MessageWriter<MoveTokenFromAreaToAreaCommand>,
    mut ship_ferry_writer: MessageWriter<ShipFerryCommand>,
    mut end_movement_writer: MessageWriter<PlayerMovementEnded>,
    area_info_query: Query<(Entity, &Population, &LandPassage, Option<&BuiltCity>, Has<CitySite>)>,
    debug_options: Res<DebugOptions>,
    mut loop_guard: ResMut<MovementLoopGuard>,
) {
    for event in event_reader.read() {
        if let Ok((player_name, available_moves, _player_areas, personality)) =
            player_moves.get(event.player)
        {
            let areas = gather_area_summaries(event.player, &area_info_query);

            let scored: Vec<(usize, f32)> = available_moves
                .moves
                .iter()
                .map(|(i, m)| {
                    (
                        *i,
                        score_movement(m, event.player, &areas, &personality.weights),
                    )
                })
                .collect();

            let mut rng = rand::rng();
            let Some(chosen) = pick(&scored, personality.picker, &mut rng) else {
                continue;
            };
            let selected_move = &available_moves.moves[&chosen];

            // Loop guard: count consecutive selections this player has made without
            // ending movement. A no-op move (one the executor silently drops) would
            // otherwise spin forever under a deterministic picker.
            let count = loop_guard.counts.entry(event.player).or_insert(0);
            *count += 1;
            let count = *count;
            if count == MovementLoopGuard::WARN_AT {
                warn!(
                    "{} has made {} consecutive movement selections without ending. \
                     Looping move (score {:.3}): {:#?}",
                    player_name,
                    count,
                    scored.iter().find(|(i, _)| *i == chosen).map_or(f32::NAN, |(_, s)| *s),
                    selected_move,
                );
            }
            if count >= MovementLoopGuard::FORCE_END_AT
                && !matches!(selected_move, GameMove::EndMovement)
            {
                error!(
                    "{} exceeded {} movement selections — forcing end of movement. \
                     Last move would have been: {:#?}",
                    player_name,
                    MovementLoopGuard::FORCE_END_AT,
                    selected_move,
                );
                loop_guard.counts.remove(&event.player);
                end_movement_writer.write(PlayerMovementEnded::new(event.player));
                continue;
            }

            {
                if debug_options.print_selected_moves {
                    debug!("{} selects {:#?}", player_name, selected_move);
                }
                match selected_move {
                    GameMove::Movement(movement_move) => {
                        //A little complexity here: If possible, leave two, but also, always make a move
                        send_movement_move(&mut move_tokens_writer, event, movement_move, false);
                    }
                    GameMove::ShipFerry(ferry_move) => {
                        // Ferry up to max_tokens (≤5) tokens by ship. Keep 1 back if possible.
                        let n = if ferry_move.max_tokens > 1 { ferry_move.max_tokens - 1 } else { 1 };
                        ship_ferry_writer.write(ShipFerryCommand::new(
                            ferry_move.source,
                            ferry_move.target,
                            n,
                            event.player,
                        ));
                    }
                    GameMove::EndMovement => {
                        loop_guard.counts.remove(&event.player);
                        end_movement_writer.write(PlayerMovementEnded::new(event.player));
                    }
                    GameMove::AttackArea(movement_move) | GameMove::AttackCity(movement_move) => {
                        //A little complexity here: If possible, leave two, but also, always make a move
                        send_movement_move(&mut move_tokens_writer, event, movement_move, true);
                    }
                    _ => {
                        debug!("In Movement, move was: {:#?}", selected_move);
                    }
                }
            }
        }
    }
}

pub fn select_stupid_city_building(
    mut event_reader: MessageReader<SelectStupidMove>,
    player_moves: Query<(&Name, &AvailableMoves, &Personality)>,
    area_info_query: Query<(Entity, &Population, &LandPassage, Option<&BuiltCity>, Has<CitySite>)>,
    mut build_city_writer: MessageWriter<BuildCityCommand>,
    mut end_player_city_construction: MessageWriter<EndPlayerCityConstruction>,
    debug_options: Res<DebugOptions>,
) {
    for event in event_reader.read() {
        if let Ok((player_name, available_moves, personality)) = player_moves.get(event.player) {
            let areas = gather_area_summaries(event.player, &area_info_query);
            let scored: Vec<(usize, f32)> = available_moves
                .moves
                .iter()
                .map(|(i, m)| (*i, score_city_construction(m, &areas, &personality.weights)))
                .collect();
            let mut rng = rand::rng();

            if let Some(chosen) = pick(&scored, personality.picker, &mut rng) {
                let selected_move = &available_moves.moves[&chosen];
                if debug_options.print_selected_moves {
                    debug!("{} selects {:#?}", player_name, selected_move);
                }
                match selected_move {
                    GameMove::CityConstruction(build_city_move) => {
                        build_city_writer
                            .write(BuildCityCommand::new(event.player, build_city_move.target));
                    }
                    GameMove::EndCityConstruction => {
                        end_player_city_construction
                            .write(EndPlayerCityConstruction::new(event.player));
                    }
                    _ => {
                        debug!("In City Construction, move was: {:#?}", selected_move);
                    }
                }
            }
        }
    }
}

pub fn select_stupid_city_elimination(
    mut event_reader: MessageReader<SelectStupidMove>,
    player_moves: Query<(&Name, &AvailableMoves, &Personality)>,
    area_info_query: Query<(Entity, &Population, &LandPassage, Option<&BuiltCity>, Has<CitySite>)>,
    mut eliminate_city: MessageWriter<EliminateCity>,
    debug_options: Res<DebugOptions>,
) {
    for event in event_reader.read() {
        if let Ok((player_name, available_moves, personality)) = player_moves.get(event.player) {
            let areas = gather_area_summaries(event.player, &area_info_query);
            let scored: Vec<(usize, f32)> = available_moves
                .moves
                .iter()
                .map(|(i, m)| (*i, score_city_elimination(m, &areas, &personality.weights)))
                .collect();
            let mut rng = rand::rng();

            if let Some(chosen) = pick(&scored, personality.picker, &mut rng) {
                let selected_move = &available_moves.moves[&chosen];
                if debug_options.print_selected_moves {
                    debug!("{} selects {:#?}", player_name, selected_move);
                }
                match selected_move {
                    GameMove::EliminateCity(el_move) => {
                        eliminate_city.write(EliminateCity::new(
                            el_move.player,
                            el_move.city,
                            el_move.area,
                            false,
                        ));
                    }
                    _ => {
                        debug!("In City Elimination, move was: {:#?}", selected_move);
                    }
                }
            }
        }
    }
}

pub fn select_stupid_trade_move(
    mut event_reader: MessageReader<SelectStupidMove>,
    player_moves: Query<(&Name, &AvailableMoves, &PlayerAreas)>,
    _commands: Commands,
    _trade_offer_query: Query<&mut TradeOffer>,
    player_trade_cards: Query<(&PlayerTradeCards, &PlayerTradeInterests)>,
    player_wants_query: Query<(&PlayerTradeInterests, &Name)>,
) {
    for event in event_reader.read() {
        // //debug!("Selecting stupid AI move for player {:#?}", event.player);
        if let Ok((player_name, available_moves, _player_areas)) = player_moves.get(event.player) {
            let trade_moves = available_moves
                .moves
                .values()
                .filter_map(|m| match m { GameMove::Trade(trade_move) => Some(trade_move), _ => None })
                .cloned()
                .collect::<Vec<_>>();
            for trade_move in &trade_moves {
                let mut rng = rand::rng();
                match trade_move {
                    TradeMove::ProposeTrade(receiver, matching_cards) => {
                        if let Ok((player_trade_cards, player_wants)) =
                            player_trade_cards.get(event.player)
                            && let Ok((_receiver_wants, receiver_name)) =
                                player_wants_query.get(*receiver)
                        {
                                match matching_cards.len() {
                                    1 => {
                                        /*
                                        This means we have ONE card in common with the other player.
                                         */
                                        let (card, no_of_cards) =
                                            matching_cards.iter().next().unwrap();
                                        // if this card is NOT our top commodity, we can offer it as a trade.
                                        if !player_trade_cards.is_top_commodity(*card) {
                                            let mut offer = TradeOffer::propose_trade(
                                                event.player,
                                                player_name,
                                                *receiver,
                                                receiver_name,
                                            );
                                            if let Some(lowest_commodity) =
                                                player_trade_cards.worst_commodity()
                                            {
                                                match no_of_cards {
                                                    0 => {
                                                        //Shouldn't happen, but could happen. We should not trade for a card we don't have.
                                                    }
                                                    1 => {
                                                        offer.pay_even_more(*card, 1);
                                                        offer.pay_even_more(lowest_commodity, 2);
                                                    }
                                                    _ => {
                                                        offer.pay_even_more(*card, 2);
                                                        offer.pay_even_more(lowest_commodity, 1);
                                                    }
                                                }
                                                //Now add what we want
                                                for (card, count) in player_wants.get_trade_thingie(&mut rng) {
                                                    offer.get_even_more(card, count);
                                                }
                                            }
                                        }
                                    }

                                    2 => {
                                        /*
                                        This means we have TWO cards in common with the other player.
                                        We exclude any match that is our top commodity, then 
                                        we add any remainders in order of number of cards.
                                         */
                                        let mut offer = TradeOffer::propose_trade(
                                            event.player,
                                            player_name,
                                            *receiver,
                                            receiver_name,
                                        );
                                        for (card, count) in player_wants.get_trade_thingie(&mut rng) {
                                            offer.get_even_more(card, count);
                                        }
                                    }
                                    3 => {
                                        // Suggest two random cards from these three and then the lowest commodity
                                        let mut cards: Vec<TradeCard> =
                                            matching_cards.keys().copied().collect();
                                        // Shuffle to get random selection
                                        cards.shuffle(&mut rng);
                                    }
                                    _ => {}
                                }
                        }
                    }
                    TradeMove::AcceptOrDeclineTrade(_trade_offer) => {}
                    TradeMove::AutoDeclineTrade(_trade_offer) => {}
                    TradeMove::StopTrading => {}
                    TradeMove::SettleTrade(_trade_offer) => {}
                }
            }
        }
    }
}

fn _debug_trade_move_info(trade_move: &TradeMove, trade_offer: &TradeOffer) {
    debug!(
        "Move: {:#?} on offer from {:#?} to {:#?}, gets: {:#?}, pays {:#?}",
        trade_move,
        trade_offer.initiator_name,
        trade_offer.receiver_name,
        trade_offer.initiator_gets,
        trade_offer.initiator_pays,
    );
}

pub fn select_stupid_civ_card_move(
    mut event_reader: MessageReader<SelectStupidMove>,
    player_moves: Query<(&Name, &AvailableMoves, &Personality)>,
    player_cards_query: Query<(
        &PlayerTradeCards,
        &PlayerCivilizationCards,
        Option<&crate::civilization::resolve_calamities::resolve_calamities_components::GrainLockedForPurchase>,
        Option<&crate::civilization::CardsHeldBeforePurchasing>,
    )>,
    cards: Res<AvailableCivCards>,
    mut done_writer: MessageWriter<PlayerDoneAcquiringCivilizationCards>,
    mut purchase_writer: MessageWriter<ConfirmCivCardPurchase>,
    debug_options: Res<DebugOptions>,
) {
    for event in event_reader.read() {
        if let Ok((player_name, available_moves, personality)) = player_moves.get(event.player) {
            // Player wealth + existing credits, used to score each card option.
            // Rule 31.53: credits are computed against what was held BEFORE
            // this turn's acquiring phase began, not the live hand -- see
            // CardsHeldBeforePurchasing's doc comment. This matters here
            // specifically because this system re-runs iteratively as the AI
            // buys one card per move, so without the snapshot a card bought
            // earlier this turn would wrongly discount a later one.
            let (wealth, credits) = match player_cards_query.get(event.player) {
                Ok((trade_cards, civ_cards, _, cards_held_before)) => (
                    trade_cards.total_stack_value() as u32,
                    cards.total_credits(cards_held_before.map_or(&civ_cards.cards, |c| &c.0)),
                ),
                Err(_) => (0, Vec::new()),
            };

            let scored: Vec<(usize, f32)> = available_moves
                .moves
                .iter()
                .filter_map(|(i, m)| match m {
                    GameMove::AcquireCivilizationCards(civ_move) => {
                        let option = match civ_move {
                            AcquireCivilizationCardsMove::AcquireCard(name) => cards
                                .cards
                                .iter()
                                .find(|c| c.name == *name)
                                .map(|def| CivCardOption {
                                    effective_cost: def.calculate_cost(&credits),
                                    credit_value: civ_card_credit_value(def),
                                    wealth,
                                }),
                            _ => None,
                        };
                        Some((*i, score_civ_card(civ_move, option, &personality.weights)))
                    }
                    _ => None,
                })
                .collect();

            if scored.is_empty() {
                return;
            }

            let mut rng = rand::rng();
            if let Some(chosen) = pick(&scored, personality.picker, &mut rng) {
                let GameMove::AcquireCivilizationCards(selected_move) =
                    &available_moves.moves[&chosen]
                else {
                    continue;
                };
                if debug_options.print_selected_moves {
                    debug!("{} selects {:#?}", player_name, selected_move);
                }
                match selected_move {
                    AcquireCivilizationCardsMove::DoneAcquiringCards => {
                        done_writer.write(PlayerDoneAcquiringCivilizationCards(event.player));
                    }
                    AcquireCivilizationCardsMove::AcquireCard(card_name) => {
                        if let Ok((trade_cards, civ_cards, grain_locked, cards_held_before)) = player_cards_query.get(event.player) {
                            let credits = cards.total_credits(cards_held_before.map_or(&civ_cards.cards, |c| &c.0));
                            let card_def = cards.cards.iter().find(|c| c.name == *card_name);
                            if let Some(def) = card_def {
                                let cost = def.calculate_cost(&credits) as usize;
                                let payment = compute_ai_payment(trade_cards, cost, grain_locked.map_or(0, |l| l.0));
                                purchase_writer.write(ConfirmCivCardPurchase {
                                    player: event.player,
                                    cards_to_buy: vec![*card_name],
                                    payment,
                                });
                            } else {
                                done_writer.write(PlayerDoneAcquiringCivilizationCards(event.player));
                            }
                        } else {
                            done_writer.write(PlayerDoneAcquiringCivilizationCards(event.player));
                        }
                    }
                    AcquireCivilizationCardsMove::AcquireCards(_) => {
                        // Not generated by recalculate_civ_card_moves_for_player; skip.
                        done_writer.write(PlayerDoneAcquiringCivilizationCards(event.player));
                    }
                }
            }
        }
    }
}

/// Sum the credit (future-discount) value a civ card hands out, regardless of which
/// card type / card it applies to — a coarse proxy for "how much tech synergy".
fn civ_card_credit_value(def: &CivCardDefinition) -> u32 {
    def.credits
        .iter()
        .map(|credit| match credit {
            Credits::ToType(_, c) | Credits::ToAll(c) | Credits::ToSpecificCard(_, c) => *c,
        })
        .sum()
}

/// Greedy minimum payment: take stacks sorted by descending face value, using just
/// enough cards from each to cover `cost`.  Returns a HashMap<TradeCard, count>.
///
/// `grain_locked` (rule 30.312) is how many of the player's Grain cards are
/// tied up face-up from a Famine/Pottery reduction this turn and therefore
/// unusable -- the Grain stack's effective count (and its count²×face
/// suite_value, recomputed here since the raw stack's is based on the full
/// count) is capped before scoring, so the AI never even considers spending
/// locked Grain.
pub fn compute_ai_payment(
    trade_cards: &PlayerTradeCards,
    cost: usize,
    grain_locked: usize,
) -> bevy::platform::collections::HashMap<TradeCard, usize> {
    let mut payment = bevy::platform::collections::HashMap::default();
    let mut remaining = cost;

    let mut stacks = trade_cards.as_card_stacks_sorted_by_value();
    if grain_locked > 0
        && let Some(grain_stack) = stacks.iter_mut().find(|s| s.card_type == TradeCard::Grain)
    {
        let usable = crate::civilization::resolve_calamities::resolve_calamities_components::usable_grain_count(
            grain_stack.count,
            grain_locked,
        );
        grain_stack.count = usable;
        grain_stack.suite_value = usable * usable * grain_stack.card_type.value();
    }
    stacks.retain(|s| s.is_commodity && s.count > 0);

    for stack in stacks {
        if remaining == 0 {
            break;
        }
        let face = stack.card_type.value();
        let full_value = stack.suite_value; // count² × face

        if full_value <= remaining {
            // The whole stack still doesn't cover what's left — use it all.
            payment.insert(stack.card_type, stack.count);
            remaining -= full_value;
        } else {
            // Find the minimum N (≥1) such that N² × face ≥ remaining.
            let mut n = 1usize;
            while n * n * face < remaining {
                n += 1;
            }
            let n = n.min(stack.count);
            payment.insert(stack.card_type, n);
            remaining = 0;
        }
    }

    payment
}

/// Flatten every area into an [`AreaSummary`] keyed by area entity, from `player`'s
/// point of view (my_pop vs enemy_pop). Gathered once per movement decision so the
/// scoring functions stay pure.
fn gather_area_summaries(
    player: Entity,
    area_info_query: &Query<(Entity, &Population, &LandPassage, Option<&BuiltCity>, Has<CitySite>)>,
) -> HashMap<Entity, AreaSummary> {
    let mut areas = HashMap::default();
    for (entity, population, land_passage, built_city, is_city_site) in area_info_query.iter() {
        let my_pop = population.population_for_player(player);
        let enemy_pop = population.total_population() - my_pop;
        areas.insert(
            entity,
            AreaSummary {
                max_population: population.max_population,
                is_city_site,
                has_city: built_city.is_some(),
                city_is_mine: built_city.is_some_and(|c| c.player == player),
                my_pop,
                enemy_pop,
                neighbours: land_passage.to_areas.clone(),
            },
        );
    }
    areas
}

fn send_movement_move(
    move_tokens_writer: &mut MessageWriter<MoveTokenFromAreaToAreaCommand>,
    event: &SelectStupidMove,
    movement_move: &MovementMove,
    is_attack: bool,
) {
    if is_attack {
        // Commit the attack with at least one token: keep one back if we can, but
        // never send a 0-token move (which is a no-op the AI would re-pick forever).
        let n = movement_move.max_tokens.saturating_sub(1).max(1);
        move_tokens_writer.write(MoveTokenFromAreaToAreaCommand::new(
            movement_move.source,
            movement_move.target,
            n,
            event.player,
        ));
    } else {
        match movement_move.max_tokens {
            1 | 2 => {
                move_tokens_writer.write(MoveTokenFromAreaToAreaCommand::new(
                    movement_move.source,
                    movement_move.target,
                    1,
                    event.player,
                ));
            }
            _ => {
                move_tokens_writer.write(MoveTokenFromAreaToAreaCommand::new(
                    movement_move.source,
                    movement_move.target,
                    2,
                    event.player,
                ));
            }
        }
    }
}

#[cfg(test)]
mod grain_lock_tests {
    use super::*;

    // ── Rule 30.312: locked Grain is never offered as AI payment ───────────

    #[test]
    fn no_lock_can_still_spend_all_grain() {
        let mut hand = PlayerTradeCards::default();
        for _ in 0..3 {
            hand.add_trade_card(TradeCard::Grain);
        }
        // 3 Grain -> suite_value 3*3*4 = 36, plenty to cover a cost of 20.
        let payment = compute_ai_payment(&hand, 20, 0);
        assert_eq!(payment.get(&TradeCard::Grain).copied(), Some(3));
    }

    #[test]
    fn fully_locked_grain_is_never_selected() {
        let mut hand = PlayerTradeCards::default();
        for _ in 0..3 {
            hand.add_trade_card(TradeCard::Grain);
        }
        // All 3 held Grain are locked -> usable count 0 -> can't cover any cost.
        let payment = compute_ai_payment(&hand, 20, 3);
        assert_eq!(payment.get(&TradeCard::Grain), None);
    }

    #[test]
    fn partially_locked_grain_only_offers_the_unlocked_remainder() {
        let mut hand = PlayerTradeCards::default();
        for _ in 0..5 {
            hand.add_trade_card(TradeCard::Grain);
        }
        // 5 held, 3 locked -> 2 usable -> suite_value 2*2*4 = 16, not enough
        // for a cost of 20, so the whole usable stack is spent (not the full 5).
        let payment = compute_ai_payment(&hand, 20, 3);
        assert_eq!(payment.get(&TradeCard::Grain).copied(), Some(2));
    }

    #[test]
    fn locked_grain_does_not_affect_other_commodity_types() {
        let mut hand = PlayerTradeCards::default();
        for _ in 0..3 {
            hand.add_trade_card(TradeCard::Grain);
        }
        for _ in 0..3 {
            hand.add_trade_card(TradeCard::Salt);
        }
        // Grain fully locked; Salt is untouched and should still be offered.
        let payment = compute_ai_payment(&hand, 5, 3);
        assert_eq!(payment.get(&TradeCard::Grain), None);
        assert!(payment.get(&TradeCard::Salt).copied().unwrap_or(0) > 0);
    }
}
