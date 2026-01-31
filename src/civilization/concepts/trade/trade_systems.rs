use crate::civilization::concepts::acquire_trade_cards::trade_card_components::PlayerTradeCards;
use crate::civilization::concepts::acquire_trade_cards::trade_card_enums::TradeCard;
use crate::civilization::concepts::trade::trade_components::{
    AvailableTradeOfferActions, CanTrade, InSettlement, NeedsTradeMove, PlayerSettlements,
    PlayerTradeInterests, PublishedOffer, TradeButtonAction, TradeOffer,
};
use crate::civilization::concepts::trade::trade_events::SendTradingCardsCommand;
use crate::civilization::concepts::trade::trade_resources::{TradeCountdown, TradeUiState};
use crate::civilization::game_moves::game_moves_components::{AvailableMoves, Move, TradeMove};
use crate::civilization::game_moves::game_moves_events::RecalculatePlayerMoves;
use crate::civilization::ui::ui_builder::UiBuilderDefaults;
use crate::stupid_ai::prelude::IsHuman;
use crate::GameActivity;
use bevy::platform::collections::HashMap;
use bevy::prelude::{
    debug, Changed, Color, Commands, Entity, MessageReader, MessageWriter, Has, Interaction, NextState,
    Query, Res, ResMut, Time, With, Without,
};
use bevy::ui::BackgroundColor;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

pub fn check_trade_gate(
    players_can_trade: Query<(Entity, &CanTrade)>,
    trade_offers: Query<(Entity, &TradeOffer)>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    if players_can_trade.iter().len() < 2 {
        debug!("Not enough players can trade. Skipping trade phase.");
        for (entity, _) in players_can_trade.iter() {
            commands.entity(entity).remove::<CanTrade>();
            commands.entity(entity).remove::<PlayerTradeInterests>();
        }
        for (entity, _) in trade_offers.iter() {
            commands.entity(entity).despawn();
        }
        next_state.set(GameActivity::PopulationExpansion);
    }
}

pub fn settle_trades(
    mut commands: Commands,
    mut trades_query: Query<(Entity, &mut TradeOffer)>,
    mut command_writer: MessageWriter<SendTradingCardsCommand>,
) {
    for (trade_offer_entity, mut offer) in trades_query.iter_mut() {
        if offer.is_player_settled() {
            debug!("Players have settled this trade, let's send out some cards!");
            command_writer.write(SendTradingCardsCommand::new(
                offer.initiator,
                offer.receiver,
                offer.initiator_cards.clone(),
            ));
            command_writer.write(SendTradingCardsCommand::new(
                offer.receiver,
                offer.initiator,
                offer.receiver_cards.clone(),
            ));
            offer.system_settled = true;
        } else if offer.is_system_settled() {
            commands.entity(trade_offer_entity).despawn();
        }
    }
}

pub fn button_action(
    mut interaction_query: Query<
        (&Interaction, &TradeButtonAction, &mut BackgroundColor),
        Changed<Interaction>,
    >,
) {
    for (interaction, menu_button_action, mut bg_color) in &mut interaction_query {
        match interaction {
            Interaction::Pressed => {
                *bg_color = PRESSED_BUTTON.into();
                match menu_button_action {
                    TradeButtonAction::Ok => {}
                    TradeButtonAction::Cancel => {}
                    TradeButtonAction::TradeAction(trade_move) => match trade_move {
                        TradeMove::AcceptOrDeclineTrade(_trade_offer) => {}
                        TradeMove::AutoDeclineTrade(_trade_offer) => {}
                        TradeMove::StopTrading => {}
                        TradeMove::SettleTrade(_trade_offer) => {}
                        TradeMove::ProposeTrade(_, _) => {}
                    },
                }
            }
            Interaction::Hovered => {
                *bg_color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *bg_color = NORMAL_BUTTON.into();
            }
        }
    }
}

pub fn setup_trade(
    mut commands: Commands,
    trading_players_query: Query<(&PlayerTradeCards, Entity, Has<IsHuman>)>,
    _ui_builder_defaults: Res<UiBuilderDefaults>,
    mut trade_ui_state: ResMut<TradeUiState>,
) {
    let mut _has_any_human = false;
    let mut _players_that_can_trade_count: usize = 0;
    for (trade_cards, player, is_human) in trading_players_query.iter() {
        if trade_cards.can_trade() {
            if is_human {
                _has_any_human = true;
                trade_ui_state.human_player = Some(player);
            }
            commands.entity(player).insert(CanTrade);
            commands
                .entity(player)
                .insert(PlayerTradeInterests::from_trade_cards(trade_cards));
            _players_that_can_trade_count += 1;
        }
    }
    // This is checked and handled in the gate checker instead,
    // if players_that_can_trade_count < 2 {
    //     debug!("Not enough players can trade. Skipping trade phase.");
    //     next_state.set(GameActivity::PopulationExpansion)
    // }
}

pub fn remove_rejected_trades(
    trade_offers: Query<(Entity, &TradeOffer), With<PublishedOffer>>,
    mut commands: Commands,
) {
    for (entity, offer) in trade_offers.iter() {
        if offer.trade_rejected() || offer.is_system_settled() {
            commands.entity(entity).despawn();
        }
    }
}

/// Reset the trade countdown timer for every published trade offer that has been accepted.
/// This queue is needed because a player can only accept one trade at a time, and we need to
/// prevent them from accepting multiple trades, and prevent other players from accepting trades
/// while another player is in the middle of a trade.
/// Good description, but does it make any sense whatsoever?
/// Yes, it kinda does, actually. This makes everyone on hold until the trade is DONE. Huh.
///
pub fn delay_trade_moves_if_offers_are_accepted(
    trade_offers: Query<&TradeOffer, With<PublishedOffer>>,
    mut trade_countdown: ResMut<TradeCountdown>,
) {
    for offer in trade_offers.iter() {
        if offer.trade_accepted() {
            trade_countdown.trade_timer.reset();
        }
    }
}

pub fn begin_trade_settlement(
    trade_offers: Query<(Entity, &TradeOffer), (With<PublishedOffer>, Without<InSettlement>)>,
    mut player_settlement_query: Query<&mut PlayerSettlements>,
    mut commands: Commands,
) {
    for (trade_entity, offer) in trade_offers.iter() {
        if offer.trade_accepted() {
            /* So much needs to happen here! */
            /*
            First, this is actually interactive. Say for instance that we are doing a trade with
            only three cards each. Ok, fine, but we still need to figure out what the hidden card is.
            Also, we need to "lock" that player from accepting other trades right now, until this
            particular trade is done. We need some kind of settlement order for trades. We could end
            up with a previously accepted trade that no longer is viable for either party.

            So a trade, when ending up here, needs to be... prioritized?
            Needs a queue number? Needs to be put in a resource? AAAAH!!

            No, we use components, as per usual. Use the ECS.

            So what we do is we check if a player already has a trade settlement in progress.

            But that trade settlement needs to have a queue of sorts to keep track of all trades that needs settling.

            So the trade itself needs to be marked as "in settlement" and then added to a queue for the
            players involved with it.
             */
            let initiator = offer.initiator;
            let receiver = offer.receiver;

            // `commands.entity(..).insert(..)` is deferred, so we must not insert and then immediately
            // expect the component to be visible via `player_settlement_query` in the same system run.
            // Instead, if the component is missing, insert an initialized value that already includes
            // this trade.
            if let Ok(mut initiator_settlements) = player_settlement_query.get_mut(initiator) {
                initiator_settlements.trades.push_back(trade_entity);
            } else {
                let mut initiator_settlements = PlayerSettlements::default();
                initiator_settlements.trades.push_back(trade_entity);
                commands.entity(initiator).insert(initiator_settlements);
            }

            if let Ok(mut receiver_settlements) = player_settlement_query.get_mut(receiver) {
                receiver_settlements.trades.push_back(trade_entity);
            } else {
                let mut receiver_settlements = PlayerSettlements::default();
                receiver_settlements.trades.push_back(trade_entity);
                commands.entity(receiver).insert(receiver_settlements);
            }

            commands.entity(trade_entity).insert(InSettlement); //Makes sure we don't end up here again!
        }
    }
}

/// On `NeedsTradeMove` components on AI players after a trade countdown finishes.
///
/// This system ticks the `TradeCountdown` timer every frame and checks if it has just finished. If it
/// has, all AI players with a `CanTrade` component but without a `NeedsTradeMove` or `IsHuman` component
/// are given a `NeedsTradeMove` component, triggering the AI to make a trade move.
pub fn trigger_trade_moves(
    time: Res<Time>,
    mut trade_countdown: ResMut<TradeCountdown>,
    can_trade_query: Query<Entity, (With<CanTrade>, Without<NeedsTradeMove>, Without<IsHuman>)>,
    mut commands: Commands,
) {
    if trade_countdown
        .trade_timer
        .tick(time.delta())
        .just_finished()
    {
        for entity in can_trade_query.iter() {
            debug!("This guy needs a trade move!");
            commands.entity(entity).insert(NeedsTradeMove);
        }
    }
}

pub fn handle_send_trading_cards_command(
    mut command_reader: MessageReader<SendTradingCardsCommand>,
    mut player_trading_cards: Query<&mut PlayerTradeCards>,
) {
    for event in command_reader.read() {
        debug!("Sending trading cards!");
        let mut sender_trade_cards = player_trading_cards.get_mut(event.sending_player).unwrap();
        let mut cards_to_send: HashMap<TradeCard, usize> = HashMap::new();
        for (trade_card, count) in event.cards_to_send.iter() {
            if let Some(cards) = sender_trade_cards.remove_n_trade_cards(*count, *trade_card) {
                cards_to_send.insert(*trade_card, cards);
            }
        }
        if !cards_to_send.is_empty() {
            let mut target_trade_cards = player_trading_cards
                .get_mut(event.receiving_player)
                .unwrap();
            for (card, count) in cards_to_send {
                target_trade_cards.add_trade_cards(card, count);
            }
        }
    }
}

pub fn recalculate_trade_moves_for_player(
    mut recalc_player_reader: MessageReader<RecalculatePlayerMoves>,
    player_cards_query: Query<&PlayerTradeCards, With<CanTrade>>,
    player_wants_query: Query<(Entity, &PlayerTradeInterests)>,
    trade_offer_query: Query<(Entity, &TradeOffer)>,
    mut player_settlements_query: Query<&mut PlayerSettlements>,
    mut commands: Commands,
) {
    /*
    So, what is a trade move? How do we define it so it can be chosen by an ai player?
     */

    for event in recalc_player_reader.read() {
        commands.entity(event.player).remove::<AvailableMoves>();
        let mut moves = HashMap::default();
        let mut command_index = 0;
        let mut is_not_settling_trade = true;
        // we cannot create new trades or stop trading while we need to settle a trade.
        if let Ok(mut player_settlement) = player_settlements_query.get_mut(event.player) {
            if !player_settlement.trades.is_empty() && player_settlement.current_trade.is_none() {
                player_settlement.current_trade =
                    Some(player_settlement.trades.pop_front().unwrap());
            }
            if let Some(current_trade) = player_settlement.current_trade {
                debug!("Player needs to settle a trade");
                is_not_settling_trade = false;
                command_index += 1;
                moves.insert(
                    command_index,
                    Move::Trade(TradeMove::SettleTrade(current_trade)),
                );
            } else {
                commands.entity(event.player).remove::<PlayerSettlements>();
            }
        }
        if is_not_settling_trade {
            if let Ok(trading_cards) = player_cards_query.get(event.player) {
                for (receiver, player_wants) in player_wants_query
                    .iter()
                    .filter(|(receiver, _)| *receiver != event.player)
                {
                    // We could make this happen only for computer players, not human players.
                    let matching_cards = trading_cards
                        .commodity_cards()
                        .iter()
                        .filter(|(card, _)| player_wants.wants.contains(card))
                        .map(|(card, size)| (*card, *size))
                        .collect::<HashMap<TradeCard, usize>>();

                    command_index += 1;
                    moves.insert(
                        command_index,
                        Move::Trade(TradeMove::ProposeTrade(
                            receiver.clone(),
                            matching_cards.clone(),
                        )),
                    );

                    //     let (_, current_player_wants) = player_wants_query.get(event.player).unwrap();
                    //     match matching_cards.len() {
                    //         1 => {
                    //             /*
                    //             This means we have ONE card in common with the other player.
                    //              */
                    //             let (card, no_of_cards) = matching_cards.iter().next().unwrap();
                    //             // if this card is NOT our top commodity, we can offer it as a trade.
                    //             if !trading_cards.is_top_commodity(*card) {
                    //                 match no_of_cards {
                    //                     0 => {
                    //                         //Shouldn't happen, but could happen. We should not trade for a card we don't have.
                    //                     }
                    //                     2 => {
                    //                         // Suggest two of these cards
                    //                         let lowest_commodity = trading_cards.worst_commodity().unwrap();
                    //                         command_index += 1;
                    //                         moves.insert(
                    //                             command_index,
                    //                             Move::Trade(TradeMove::propose_trade(
                    //                                 current_player_wants.get_trade_thingie(&mut rng),
                    //                                 HashMap::from([(*card, 2),(lowest_commodity, 1)]),
                    //                                 receiver,
                    //                             )),
                    //                         );
                    //                     }
                    //                     3 => {
                    //                         // Suggest all three cards
                    //                         command_index += 1;
                    //                         moves.insert(
                    //                             command_index,
                    //                             Move::Trade(TradeMove::propose_trade(
                    //                                 current_player_wants.get_trade_thingie(&mut rng),
                    //                                 HashMap::from([(*card, 3)]),
                    //                                 receiver,
                    //                             )),
                    //                         );
                    //                     }
                    //                     _ => {
                    //                         // Suggest one of these and then the lowest commodity
                    //                         let lowest_commodity = trading_cards.worst_commodity().unwrap();
                    //
                    //                         command_index += 1;
                    //                         moves.insert(
                    //                             command_index,
                    //                             Move::Trade(TradeMove::propose_trade(
                    //                                 current_player_wants.get_trade_thingie(&mut rng),
                    //                                 HashMap::from([(*card, 1),(lowest_commodity, 2)]), // we can always offer two of the lowest commodity - even if we only have one
                    //                                 receiver,
                    //                             )),
                    //                         );
                    //                     }
                    //                 }
                    //             }
                    //         }
                    //         2 => {
                    //             // Suggest one of each of these two
                    //             let cards: Vec<TradeCard> = matching_cards.keys().copied().collect();
                    //             if cards.len() == 2 {
                    //                 command_index += 1;
                    //                 moves.insert(
                    //                     command_index,
                    //                     Move::Trade(TradeMove::propose_trade(
                    //                         current_player_wants.get_trade_thingie(&mut rng),
                    //                         HashMap::from([(cards[0], 1),(cards[1], 2)]),
                    //                         receiver,
                    //                     )),
                    //                 );
                    //             }
                    //         }
                    //         3 => {
                    //             // Suggest two random cards from these three and then the lowest commodity
                    //             let mut cards: Vec<TradeCard> = matching_cards.keys().copied().collect();
                    //             // Shuffle to get random selection
                    //             cards.shuffle(&mut rng);
                    //
                    //             if cards.len() >= 2 {
                    //                 command_index += 1;
                    //                 let lowest_commodity = trading_cards.worst_commodity().unwrap();
                    //                 moves.insert(
                    //                     command_index,
                    //                     Move::Trade(TradeMove::propose_trade(
                    //                         current_player_wants.get_trade_thingie(&mut rng),
                    //                         HashMap::from([(cards[0], 1),(cards[1], 1), (lowest_commodity, 1)]),
                    //                         receiver,
                    //                     )),
                    //                 );
                    //
                    //             }
                    //         }
                    //         _ => {}
                    //     }
                    // }

                    for (trade_offer_entity, trade_offer) in trade_offer_query.iter() {
                        if let Some(offer_actions) =
                            trade_offer.get_trade_offer_actions(event.player)
                        {
                            for action in offer_actions {
                                match action {
                                    AvailableTradeOfferActions::CanAccept => {
                                        command_index += 1;
                                        moves.insert(
                                            command_index,
                                            Move::Trade(TradeMove::AcceptOrDeclineTrade(
                                                trade_offer_entity,
                                            )),
                                        );
                                    }
                                    AvailableTradeOfferActions::Decline => {
                                        command_index += 1;
                                        moves.insert(
                                            command_index,
                                            Move::Trade(TradeMove::AutoDeclineTrade(
                                                trade_offer_entity,
                                            )),
                                        );
                                    }
                                }
                            }
                        }
                    }
                    command_index += 1;
                    moves.insert(command_index, Move::Trade(TradeMove::StopTrading));
                }
            }
            commands.entity(event.player).remove::<NeedsTradeMove>();
        }
        if moves.is_empty() {
            commands.entity(event.player).remove::<CanTrade>();
        } else {
            commands
                .entity(event.player)
                .insert(AvailableMoves::new(moves));
        }
    }
}
