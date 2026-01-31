use crate::civilization::components::population::Population;
use crate::civilization::components::*;
use crate::civilization::concepts::acquire_trade_cards::trade_card_components::PlayerTradeCards;
use crate::civilization::concepts::acquire_trade_cards::trade_card_enums::TradeCard;
use crate::civilization::concepts::check_city_support::check_city_support_events::EliminateCity;
use crate::civilization::concepts::city_construction::city_construction_events::{
    BuildCityCommand, EndPlayerCityConstruction,
};
use crate::civilization::concepts::movement::movement_events::{
    MoveTokenFromAreaToAreaCommand, PlayerMovementEnded,
};
use crate::civilization::concepts::population_expansion::population_expansion_events::ExpandPopulationManuallyCommand;
use crate::civilization::concepts::trade::trade_components::{
    PlayerTradeInterests, TradeOffer,
};
use crate::civilization::concepts::trade::trade_functions::{
    initiator_can_pay_for_offer, receiver_can_pay_for_offer,
};
use crate::civilization::game_moves::game_moves_components::{
    AvailableMoves, Move, MovementMove, TradeMove,
};
use crate::civilization::plugins::civilization_plugin::DebugOptions;
use crate::stupid_ai::stupid_ai_components::StupidAi;
use crate::stupid_ai::stupid_ai_events::{SelectStupidMove, StupidAiMessage};
use bevy::platform::collections::HashMap;
use bevy::prelude::{
    debug, Commands, Entity, MessageReader, MessageWriter, Has, Name, Query, Res,
};
use itertools::Itertools;
use rand::prelude::{IteratorRandom, SliceRandom};

pub fn setup_stupid_ai(mut stupid_ai_event: MessageReader<StupidAiMessage>, mut commands: Commands) {
    for e in stupid_ai_event.read() {
        commands.entity(e.player).insert(StupidAi);
    }
}

pub fn assign_some_foolish_cards() {}

pub fn select_stupid_pop_exp(
    mut event_reader: MessageReader<SelectStupidMove>,
    player_moves: Query<(&Name, &AvailableMoves, &PlayerAreas)>,
    mut expand_writer: MessageWriter<ExpandPopulationManuallyCommand>,
    debug_options: Res<DebugOptions>,
) {
    for event in event_reader.read() {
        if let Ok((player_name, available_moves, _player_areas)) = player_moves.get(event.player) {
            let available_moves = available_moves.moves.values().collect::<Vec<_>>();
            let mut rng = rand::rng();
            if let Some(selected_move) = available_moves.into_iter().choose(&mut rng) {
                if debug_options.print_selected_moves {
                    debug!("{} selects {:#?}", player_name, selected_move);
                }
                match selected_move {
                    Move::PopulationExpansion(pop_exp_move) => {
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
    player_moves: Query<(&Name, &AvailableMoves, &PlayerAreas)>,
    mut move_tokens_writer: MessageWriter<MoveTokenFromAreaToAreaCommand>,
    mut end_movement_writer: MessageWriter<PlayerMovementEnded>,
    target_area_info_query: Query<(&Population, Has<BuiltCity>)>,
    debug_options: Res<DebugOptions>,
) {
    for event in event_reader.read() {
        if let Ok((player_name, available_moves, _player_areas)) = player_moves.get(event.player) {
            let available_moves = available_moves
                .moves
                .values()
                .filter(|m| match m {
                    Move::Movement(move_ment) => {
                        let (_population, has_city) =
                            target_area_info_query.get(move_ment.target).unwrap();
                        !has_city
                    }
                    _ => true,
                })
                .collect::<Vec<_>>();

            let mut rng = rand::rng();

            if let Some(selected_move) = available_moves.into_iter().choose(&mut rng) {
                if debug_options.print_selected_moves {
                    debug!("{} selects {:#?}", player_name, selected_move);
                }
                match selected_move {
                    Move::Movement(movement_move) => {
                        //A little complexity here: If possible, leave two, but also, always make a move
                        send_movement_move(&mut move_tokens_writer, event, movement_move, false);
                    }
                    Move::EndMovement => {
                        end_movement_writer.write(PlayerMovementEnded::new(event.player));
                    }
                    Move::AttackArea(movement_move) => {
                        //A little complexity here: If possible, leave two, but also, always make a move
                        send_movement_move(&mut move_tokens_writer, event, movement_move, true);
                    }
                    Move::AttackCity(movement_move) => {
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
    player_moves: Query<(&Name, &AvailableMoves, &PlayerAreas)>,
    mut build_city_writer: MessageWriter<BuildCityCommand>,
    mut end_player_city_construction: MessageWriter<EndPlayerCityConstruction>,
    debug_options: Res<DebugOptions>,
) {
    for event in event_reader.read() {
        if let Ok((player_name, available_moves, _player_areas)) = player_moves.get(event.player) {
            let available_moves = available_moves.moves.values().collect::<Vec<_>>();

            let mut rng = rand::rng();

            if let Some(selected_move) = available_moves.into_iter().choose(&mut rng) {
                if debug_options.print_selected_moves {
                    debug!("{} selects {:#?}", player_name, selected_move);
                }
                match selected_move {
                    Move::CityConstruction(build_city_move) => {
                        build_city_writer
                            .write(BuildCityCommand::new(event.player, build_city_move.target));
                    }
                    Move::EndCityConstruction => {
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
    player_moves: Query<(&Name, &AvailableMoves, &PlayerAreas)>,
    mut eliminate_city: MessageWriter<EliminateCity>,
    debug_options: Res<DebugOptions>,
) {
    for event in event_reader.read() {
        if let Ok((player_name, available_moves, _player_areas)) = player_moves.get(event.player) {
            let available_moves = available_moves.moves.values().collect::<Vec<_>>();

            let mut rng = rand::rng();

            if let Some(selected_move) = available_moves.into_iter().choose(&mut rng) {
                if debug_options.print_selected_moves {
                    debug!("{} selects {:#?}", player_name, selected_move);
                }
                match selected_move {
                    Move::EliminateCity(el_move) => {
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
                .filter(|m| matches!(m, Move::Trade(_)))
                .map(|m| match m {
                    Move::Trade(trade_move) => trade_move,
                    _ => unreachable!(),
                })
                .cloned()
                .collect::<Vec<_>>();
            for trade_move in trade_moves.iter() {
                let mut rng = rand::rng();
                match trade_move {
                    TradeMove::ProposeTrade(receiver, matching_cards) => {
                        if let Ok((player_trade_cards, player_wants)) =
                            player_trade_cards.get(event.player)
                        {
                            if let Ok((_receiver_wants, receiver_name)) =
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

fn debug_trade_move_info(trade_move: &TradeMove, trade_offer: &TradeOffer) {
    debug!(
        "Move: {:#?} on offer from {:#?} to {:#?}, gets: {:#?}, pays {:#?}",
        trade_move,
        trade_offer.initiator_name,
        trade_offer.receiver_name,
        trade_offer.initiator_gets,
        trade_offer.initiator_pays,
    );
}

fn settle_trade(my_cards: &PlayerTradeCards, player: Entity, trade_offer: &mut TradeOffer) {
    let mut trade_valid = true;

    if trade_offer.initiator == player {
        debug!("Player initiated this trade, time to pay up!");
        if initiator_can_pay_for_offer(&trade_offer, my_cards) {
            let mut cards_left = trade_offer.pays_number_of_cards();

            let mut cards_to_use: HashMap<TradeCard, usize> = HashMap::new();

            for (card_type, count) in trade_offer.initiator_pays_guaranteed.iter() {
                cards_to_use.insert(*card_type, *count);
                cards_left -= count;
            }
            if cards_left > 0 {
                for calamity in my_cards.tradeable_calamity_cards_ranked() {
                    if cards_left > 0 {
                        cards_left -= 1;
                        cards_to_use.insert(calamity, 1);
                    }
                }
            }

            assign_cards(my_cards, cards_left, &mut cards_to_use);

            /*
            This is kind of crazy, but perhaps I added too much here, refactor later

            But what we need to do is now simply add all the cards to the trade offer as well
             */

            if trade_valid {
                trade_offer.settle(player, cards_to_use);
            }
        } else {
            debug!("Initiator couldn't pay, trade invalid!");
            trade_valid = false;
        }
    } else if trade_offer.receiver == player {
        debug!("Player is receiver of this trade, time to pay up!");
        if receiver_can_pay_for_offer(&trade_offer, my_cards) {
            let mut cards_left = trade_offer.gets_number_of_cards();

            let mut cards_to_use = HashMap::new();

            for (card_type, count) in trade_offer.initiator_gets_guaranteed.iter() {
                cards_to_use.insert(*card_type, *count);
                cards_left -= count;
            }
            if cards_left > 0 {
                for calamity in my_cards.tradeable_calamity_cards_ranked() {
                    if cards_left > 0 {
                        cards_left -= 1;
                        cards_to_use.insert(calamity, 1);
                    }
                }
            }
            assign_cards(my_cards, cards_left, &mut cards_to_use);

            if trade_valid {
                trade_offer.settle(player, cards_to_use);
            }
        } else {
            debug!("Receiver couldn't pay, trade invalid!");
            trade_valid = false;
        }
    }

    if !trade_valid {
        trade_offer.reject(player);
    }
}

fn assign_cards(
    my_cards: &PlayerTradeCards,
    mut cards_left: usize,
    cards_to_use: &mut HashMap<TradeCard, usize>,
) {
    for (commodity, _suite_value) in my_cards
        .commodity_card_suites()
        .iter()
        .sorted_by_key(|(_commodity, value)| *value)
        .rev()
    {
        if cards_left > 0 {
            if my_cards.number_of_cards_for_trade_card(*commodity) > cards_left {
                cards_to_use.insert(*commodity, cards_left);
                cards_left = 0;
            } else {
                cards_left -= my_cards.number_of_cards_for_trade_card(*commodity);
                cards_to_use.insert(
                    *commodity,
                    my_cards.number_of_cards_for_trade_card(*commodity),
                );
            }
        }
    }
}

fn send_movement_move(
    move_tokens_writer: &mut MessageWriter<MoveTokenFromAreaToAreaCommand>,
    event: &SelectStupidMove,
    movement_move: &MovementMove,
    is_attack: bool,
) {
    if is_attack {
        move_tokens_writer.write(MoveTokenFromAreaToAreaCommand::new(
            movement_move.source,
            movement_move.target,
            movement_move.max_tokens - 1,
            event.player,
        ));
    } else {
        match movement_move.max_tokens {
            1 => {
                move_tokens_writer.write(MoveTokenFromAreaToAreaCommand::new(
                    movement_move.source,
                    movement_move.target,
                    1,
                    event.player,
                ));
            }
            2 => {
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
