use crate::stupid_ai::*;
use bevy::prelude::{
    debug, Commands, Has, MessageReader, MessageWriter, Name, Query, Res,
};
use rand::prelude::{IteratorRandom, SliceRandom};
use crate::civilization::*;

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
                    GameMove::Movement(move_ment) => {
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
                    GameMove::Movement(movement_move) => {
                        //A little complexity here: If possible, leave two, but also, always make a move
                        send_movement_move(&mut move_tokens_writer, event, movement_move, false);
                    }
                    GameMove::EndMovement => {
                        end_movement_writer.write(PlayerMovementEnded::new(event.player));
                    }
                    GameMove::AttackArea(movement_move) => {
                        //A little complexity here: If possible, leave two, but also, always make a move
                        send_movement_move(&mut move_tokens_writer, event, movement_move, true);
                    }
                    GameMove::AttackCity(movement_move) => {
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
    player_moves: Query<(&Name, &AvailableMoves)>,
    mut done_writer: MessageWriter<PlayerDoneAcquiringCivilizationCards>,
    debug_options: Res<DebugOptions>,
) {
    for event in event_reader.read() {
        if let Ok((player_name, available_moves)) = player_moves.get(event.player) {
            let civ_moves: Vec<_> = available_moves
                .moves
                .values()
                .filter_map(|m| match m {
                    GameMove::AcquireCivilizationCards(civ_move) => Some(civ_move),
                    _ => None,
                })
                .collect();

            if civ_moves.is_empty() {
                return;
            }

            let mut rng = rand::rng();
            if let Some(selected_move) = civ_moves.into_iter().choose(&mut rng) {
                if debug_options.print_selected_moves {
                    debug!("{} selects {:#?}", player_name, selected_move);
                }
                match selected_move {
                    AcquireCivilizationCardsMove::DoneAcquiringCards => {
                        done_writer.write(PlayerDoneAcquiringCivilizationCards(event.player));
                    }
                    AcquireCivilizationCardsMove::AcquireCard(_)
                    | AcquireCivilizationCardsMove::AcquireCards(_) => {
                        done_writer.write(PlayerDoneAcquiringCivilizationCards(event.player));
                    }
                }
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
