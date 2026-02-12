use crate::civilization::concepts::acquire_trade_cards::PlayerTradeCards;
use crate::civilization::concepts::acquire_trade_cards::TradeCard;
use crate::civilization::concepts::trade::trade_components::{
    AvailableTradeOfferActions, CanTrade, CreateOfferButton, CreateOfferModal, DoneTradingButton,
    InSettlement, NeedsTradeMove, OpenOffersListContainer, OpenTradeOffer, PlayerSettlements,
    PlayerTradeInterests, PublishedOffer, SettlementModal, TradeButtonAction, TradeOffer,
};
use crate::civilization::concepts::trade::trade_events::SendTradingCardsCommand;
use crate::civilization::concepts::trade::trade_resources::{CreateOfferState, TradeCountdown, TradePhaseState, TradeUiState};
use crate::civilization::game_moves::RecalculatePlayerMoves;
use crate::civilization::game_moves::{AvailableMoves, GameMove, TradeMove};
use crate::civilization::{TradeCardTrait, TradePhaseUiRoot};
use crate::stupid_ai::IsHuman;
use crate::GameActivity;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use lava_ui_builder::{UIBuilder, UiTheme};

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

pub fn check_trade_gate(
    players_can_trade: Query<(Entity, &CanTrade)>,
    trade_offers: Query<(Entity, &TradeOffer)>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    // Only end trading when NO players remain. Individual players leaving
    // (including the human) should not end the phase — the countdown timer
    // or all players finishing will eventually bring this to zero.
    if players_can_trade.iter().len() == 0 {
        debug!("No players left trading. Ending trade phase.");
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
    _ui_theme: Res<UiTheme>,
    mut trade_ui_state: ResMut<TradeUiState>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    let mut players_that_can_trade_count: usize = 0;
    for (trade_cards, player, is_human) in trading_players_query.iter() {
        if trade_cards.can_trade() {
            if is_human {
                trade_ui_state.human_player = Some(player);
            }
            commands.entity(player).insert(CanTrade);
            commands
                .entity(player)
                .insert(PlayerTradeInterests::from_trade_cards(trade_cards));
            players_that_can_trade_count += 1;
        }
    }
    // Skip trade phase entirely if fewer than 2 players can trade at the start
    if players_that_can_trade_count < 2 {
        debug!("Not enough players can trade at start. Skipping trade phase.");
        next_state.set(GameActivity::PopulationExpansion);
    }
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
/// This queue is needed because a player can only accept one trade at a time. We need to
/// prevent them from accepting multiple trades, and prevent other players from accepting trades.
/// While another player is in the middle of a trade.
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
            Needs a queue number? Needs to be put in a resource? 

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
    So, what is a trade move? How do we define it so it can be chosen by an AI player?
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
                    GameMove::Trade(TradeMove::SettleTrade(current_trade)),
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
                        GameMove::Trade(TradeMove::ProposeTrade(
                            receiver,
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
                    //                                 current_player_wants.get_trade_thing(&mut rng),
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
                    //                                 current_player_wants.get_trade_thing(&mut rng),
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
                    //                                 current_player_wants.get_trade_thing(&mut rng),
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
                    //                         current_player_wants.get_trade_thing(&mut rng),
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
                    //                         current_player_wants.get_trade_thing(&mut rng),
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
                                            GameMove::Trade(TradeMove::AcceptOrDeclineTrade(
                                                trade_offer_entity,
                                            )),
                                        );
                                    }
                                    AvailableTradeOfferActions::Decline => {
                                        command_index += 1;
                                        moves.insert(
                                            command_index,
                                            GameMove::Trade(TradeMove::AutoDeclineTrade(
                                                trade_offer_entity,
                                            )),
                                        );
                                    }
                                }
                            }
                        }
                    }
                    command_index += 1;
                    moves.insert(command_index, GameMove::Trade(TradeMove::StopTrading));
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

/// Marker component for the countdown timer text
#[derive(bevy::prelude::Component, Default)]
pub struct TradeCountdownText;

/// Marker component for the player's cards display
#[derive(bevy::prelude::Component, Default)]
pub struct PlayerCardsDisplay;

/// Set up the trade phase UI overlay
pub fn setup_trade_phase_ui(
    mut commands: Commands,
    trade_ui_state: Res<TradeUiState>,
    human_player_query: Query<(&Name, &PlayerTradeCards), With<IsHuman>>,
    mut trade_phase_state: ResMut<TradePhaseState>,
) {
    trade_phase_state.countdown_seconds = 90.0; // 90 seconds
    trade_phase_state.human_done = false;
    
    // Get human player's cards for display - handle case where no human player exists
    let human_data = if let Some(human) = trade_ui_state.human_player {
        human_player_query.get(human).ok()
    } else {
        human_player_query.iter().next()
    };
    
    let Some((player_name, player_cards)) = human_data else {
        // No human player - skip trade UI setup (AI-only game)
        debug!("No human player found, skipping trade phase UI");
        return;
    };
    
    // Build the trade UI overlay
    commands
        .spawn((
            TradePhaseUiRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(20.0),
                top: Val::Percent(10.0),
                width: Val::Percent(60.0),
                height: Val::Percent(80.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                ..Default::default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.95)),
        ))
        .with_children(|parent| {
            // Header row with title and countdown
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(50.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    margin: UiRect::bottom(Val::Px(16.0)),
                    ..Default::default()
                })
                .with_children(|header| {
                    // Title
                    header.spawn((
                        Text::new("⚖️ TRADE PHASE"),
                        TextFont {
                            font_size: 28.0,
                            ..Default::default()
                        },
                        TextColor(Color::srgb(0.9, 0.8, 0.3)),
                    ));
                    
                    // Countdown timer
                    header.spawn((
                        TradeCountdownText,
                        Text::new("5:00"),
                        TextFont {
                            font_size: 24.0,
                            ..Default::default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    ));
                });
            
            // Player trading status row
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..Default::default()
                })
                .with_children(|status_row| {
                    status_row.spawn((
                        Text::new("Trading: "),
                        TextFont { font_size: 12.0, ..Default::default() },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                    ));
                    status_row.spawn((
                        PlayerTradingStatusDisplay,
                        Text::new("Loading..."),
                        TextFont { font_size: 12.0, ..Default::default() },
                        TextColor(Color::srgb(0.7, 0.9, 0.7)),
                    ));
                });
            
            // Player's cards section
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Auto,
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::bottom(Val::Px(16.0)),
                    padding: UiRect::all(Val::Px(12.0)),
                    ..Default::default()
                })
                .insert(BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.8)))
                .with_children(|cards_section| {
                    // Section header
                    cards_section.spawn((
                        Text::new(format!("Your Cards ({})", player_name)),
                        TextFont {
                            font_size: 16.0,
                            ..Default::default()
                        },
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    ));
                    
                    // Cards display
                    cards_section
                        .spawn((
                            PlayerCardsDisplay,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Auto,
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                margin: UiRect::top(Val::Px(8.0)),
                                ..Default::default()
                            },
                        ))
                        .with_children(|cards_row| {
                            // Display commodity cards
                            for (card, count) in player_cards.commodity_cards() {
                                for _ in 0..count {
                                    let color = commodity_color(&card);
                                    cards_row
                                        .spawn((
                                            Node {
                                                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                                                margin: UiRect::all(Val::Px(2.0)),
                                                ..Default::default()
                                            },
                                            BackgroundColor(color),
                                        ))
                                        .with_child((
                                            Text::new(format!("{}", card)),
                                            TextFont { font_size: 12.0, ..Default::default() },
                                            TextColor(Color::WHITE),
                                        ));
                                }
                            }
                            // Display calamity cards
                            for card in player_cards.calamity_cards() {
                                cards_row
                                    .spawn((
                                        Node {
                                            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                                            margin: UiRect::all(Val::Px(2.0)),
                                            ..Default::default()
                                        },
                                        BackgroundColor(Color::srgb(0.8, 0.2, 0.2)),
                                    ))
                                    .with_child((
                                        Text::new(format!("{}", card)),
                                        TextFont { font_size: 12.0, ..Default::default() },
                                        TextColor(Color::WHITE),
                                    ));
                            }
                        });
                });
            
            // Open offers section
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::bottom(Val::Px(16.0)),
                    ..Default::default()
                })
                .with_children(|offers_section| {
                    // Section header
                    offers_section.spawn((
                        Text::new("📜 Open Offers"),
                        TextFont {
                            font_size: 18.0,
                            ..Default::default()
                        },
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    ));
                    
                    // Offers list container (will be populated dynamically)
                    offers_section
                        .spawn((
                            OpenOffersListContainer,
                            Node {
                                width: Val::Percent(100.0),
                                flex_grow: 1.0,
                                flex_direction: FlexDirection::Column,
                                overflow: Overflow::clip(),
                                margin: UiRect::top(Val::Px(8.0)),
                                padding: UiRect::all(Val::Px(8.0)),
                                ..Default::default()
                            },
                            BackgroundColor(Color::srgba(0.1, 0.1, 0.12, 0.5)),
                        ))
                        .with_child((
                            Text::new("No open offers yet. Create one or wait for others!"),
                            TextFont { font_size: 14.0, ..Default::default() },
                            TextColor(Color::srgb(0.5, 0.5, 0.5)),
                        ));
                });
            
            // Bottom buttons row
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(50.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..Default::default()
                })
                .with_children(|buttons| {
                    // Create Offer button
                    buttons
                        .spawn((
                            Button,
                            CreateOfferButton,
                            Node {
                                padding: UiRect::axes(Val::Px(20.0), Val::Px(12.0)),
                                justify_content: JustifyContent::Center,
                                ..Default::default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.5, 0.3)),
                        ))
                        .with_child((
                            Text::new("➕ Create Offer"),
                            TextFont { font_size: 16.0, ..Default::default() },
                            TextColor(Color::WHITE),
                        ));
                    
                    // Done Trading button
                    buttons
                        .spawn((
                            Button,
                            DoneTradingButton,
                            Node {
                                padding: UiRect::axes(Val::Px(20.0), Val::Px(12.0)),
                                justify_content: JustifyContent::Center,
                                ..Default::default()
                            },
                            BackgroundColor(Color::srgb(0.5, 0.3, 0.2)),
                        ))
                        .with_child((
                            Text::new("✓ Done Trading"),
                            TextFont { font_size: 16.0, ..Default::default() },
                            TextColor(Color::WHITE),
                        ));
                });
        });
}

fn commodity_color(card: &TradeCard) -> Color {
    match card {
        TradeCard::Hides => Color::srgb(0.6, 0.4, 0.2),
        TradeCard::Ochre => Color::srgb(0.8, 0.4, 0.2),
        TradeCard::Iron => Color::srgb(0.5, 0.5, 0.5),
        TradeCard::Papyrus => Color::srgb(0.9, 0.9, 0.7),
        TradeCard::Salt => Color::srgb(0.9, 0.9, 0.9),
        TradeCard::Grain => Color::srgb(0.8, 0.7, 0.3),
        TradeCard::Cloth => Color::srgb(0.6, 0.3, 0.6),
        TradeCard::Bronze => Color::srgb(0.8, 0.5, 0.2),
        TradeCard::Spices => Color::srgb(0.7, 0.5, 0.3),
        TradeCard::Resin => Color::srgb(0.5, 0.3, 0.1),
        TradeCard::Gems => Color::srgb(0.3, 0.7, 0.9),
        TradeCard::Dye => Color::srgb(0.8, 0.2, 0.5),
        TradeCard::Wine => Color::srgb(0.6, 0.1, 0.2),
        TradeCard::Oil => Color::srgb(0.3, 0.4, 0.2),
        TradeCard::Silver => Color::srgb(0.7, 0.7, 0.8),
        TradeCard::Ivory => Color::srgb(0.95, 0.95, 0.9),
        TradeCard::Gold => Color::srgb(0.9, 0.8, 0.2),
        _ => Color::srgb(0.4, 0.4, 0.4), // Calamities and unknown
    }
}

/// Clean up the trade phase UI when leaving trade
pub fn cleanup_trade_phase_ui(
    mut commands: Commands,
    trade_ui_query: Query<Entity, With<TradePhaseUiRoot>>,
) {
    for entity in trade_ui_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Update the countdown timer display and end trade when timer expires
pub fn update_trade_countdown_display(
    time: Res<Time>,
    mut trade_phase_state: ResMut<TradePhaseState>,
    mut countdown_text: Query<&mut Text, With<TradeCountdownText>>,
    can_trade_query: Query<Entity, With<CanTrade>>,
    mut commands: Commands,
) {
    trade_phase_state.countdown_seconds -= time.delta_secs();
    if trade_phase_state.countdown_seconds < 0.0 {
        trade_phase_state.countdown_seconds = 0.0;
    }
    
    let minutes = (trade_phase_state.countdown_seconds / 60.0) as u32;
    let seconds = (trade_phase_state.countdown_seconds % 60.0) as u32;
    
    for mut text in countdown_text.iter_mut() {
        **text = format!("{}:{:02}", minutes, seconds);
    }
    
    // End trading for all players when countdown expires
    if trade_phase_state.countdown_seconds <= 0.0 {
        debug!("Trade countdown expired — removing CanTrade from all remaining players");
        for entity in can_trade_query.iter() {
            commands.entity(entity).remove::<CanTrade>();
        }
    }
}

/// Update the player trading status display
pub fn update_player_trading_status(
    all_players: Query<(Entity, &Name), With<PlayerTradeCards>>,
    trading_players: Query<Entity, With<CanTrade>>,
    mut status_display: Query<(&mut Text, &mut TextColor), With<PlayerTradingStatusDisplay>>,
) {
    let Ok((mut text, mut color)) = status_display.single_mut() else {
        return;
    };
    
    let trading_set: bevy::platform::collections::HashSet<Entity> = trading_players.iter().collect();
    
    let mut still_trading: Vec<String> = Vec::new();
    let mut done_trading: Vec<String> = Vec::new();
    
    for (entity, name) in all_players.iter() {
        if trading_set.contains(&entity) {
            still_trading.push(name.to_string());
        } else {
            done_trading.push(name.to_string());
        }
    }
    
    if still_trading.is_empty() {
        **text = "Everyone done!".to_string();
        *color = TextColor(Color::srgb(0.5, 0.5, 0.5));
    } else {
        let trading_text = still_trading.join(", ");
        let done_text = if done_trading.is_empty() {
            String::new()
        } else {
            format!(" | Done: {}", done_trading.join(", "))
        };
        **text = format!("{}{}", trading_text, done_text);
        *color = TextColor(Color::srgb(0.7, 0.9, 0.7));
    }
}

/// Update the open offers display with actual offers
pub fn update_open_offers_display(
    mut commands: Commands,
    offers_query: Query<(Entity, &OpenTradeOffer), Changed<OpenTradeOffer>>,
    all_offers_query: Query<(Entity, &OpenTradeOffer)>,
    container_query: Query<Entity, With<OpenOffersListContainer>>,
    children_query: Query<&Children>,
    trade_ui_state: Res<TradeUiState>,
) {
    // Only update if offers changed
    if offers_query.is_empty() {
        return;
    }
    
    let Ok(container) = container_query.single() else {
        return;
    };
    
    // Clear existing children
    if let Ok(children) = children_query.get(container) {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }
    
    // Get all active offers (not withdrawn, not fully settled)
    let active_offers: Vec<_> = all_offers_query
        .iter()
        .filter(|(_, o)| !o.withdrawn)
        .collect();
    
    if active_offers.is_empty() {
        // Show "no offers" message
        commands.entity(container).with_children(|parent| {
            parent.spawn((
                Text::new("No open offers yet. Create one or wait for others!"),
                TextFont { font_size: 14.0, ..Default::default() },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
        });
        return;
    }
    
    let human_player = trade_ui_state.human_player;
    
    // Display each offer
    commands.entity(container).with_children(|parent| {
        for (offer_entity, offer) in active_offers.iter() {
            let is_my_offer = human_player.map(|h| h == offer.creator).unwrap_or(false);
            let is_accepted = offer.accepted_by.is_some();
            
            // Offer card
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(8.0)),
                    margin: UiRect::bottom(Val::Px(8.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..Default::default()
                })
                .insert(BackgroundColor(if is_accepted {
                    Color::srgba(0.2, 0.4, 0.2, 0.8)
                } else if is_my_offer {
                    Color::srgba(0.2, 0.3, 0.4, 0.8)
                } else {
                    Color::srgba(0.15, 0.15, 0.2, 0.8)
                }))
                .insert(BorderColor::all(if is_accepted {
                    Color::srgb(0.3, 0.8, 0.3)
                } else {
                    Color::srgb(0.3, 0.3, 0.4)
                }))
                .with_children(|card| {
                    // Header: Creator name and status
                    card.spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        margin: UiRect::bottom(Val::Px(4.0)),
                        ..Default::default()
                    }).with_children(|header| {
                        header.spawn((
                            Text::new(format!("📤 {}", offer.creator_name)),
                            TextFont { font_size: 14.0, ..Default::default() },
                            TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        ));
                        
                        if is_accepted {
                            header.spawn((
                                Text::new(format!("✅ Accepted by {}", offer.accepted_by_name.as_deref().unwrap_or("?"))),
                                TextFont { font_size: 12.0, ..Default::default() },
                                TextColor(Color::srgb(0.5, 0.9, 0.5)),
                            ));
                        } else if is_my_offer {
                            header.spawn((
                                Text::new("(Your offer)"),
                                TextFont { font_size: 12.0, ..Default::default() },
                                TextColor(Color::srgb(0.6, 0.7, 0.9)),
                            ));
                        }
                    });
                    
                    // Offering section
                    card.spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        margin: UiRect::bottom(Val::Px(2.0)),
                        ..Default::default()
                    }).with_children(|row| {
                        row.spawn((
                            Text::new("Offers: "),
                            TextFont { font_size: 12.0, ..Default::default() },
                            TextColor(Color::srgb(0.7, 0.7, 0.7)),
                        ));
                        
                        let offering_text: Vec<String> = offer.offering_guaranteed
                            .iter()
                            .map(|(card, count)| {
                                if *count > 1 { format!("{}x{}", count, card) } else { format!("{}", card) }
                            })
                            .collect();
                        let hidden_text = if offer.offering_hidden_count > 0 {
                            format!(" +{} hidden", offer.offering_hidden_count)
                        } else {
                            String::new()
                        };
                        
                        row.spawn((
                            Text::new(format!("{}{}", offering_text.join(", "), hidden_text)),
                            TextFont { font_size: 12.0, ..Default::default() },
                            TextColor(Color::srgb(0.5, 0.9, 0.5)),
                        ));
                    });
                    
                    // Wanting section
                    card.spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        margin: UiRect::bottom(Val::Px(4.0)),
                        ..Default::default()
                    }).with_children(|row| {
                        row.spawn((
                            Text::new("Wants: "),
                            TextFont { font_size: 12.0, ..Default::default() },
                            TextColor(Color::srgb(0.7, 0.7, 0.7)),
                        ));
                        
                        let wanting_text: Vec<String> = offer.wanting_guaranteed
                            .iter()
                            .map(|(card, count)| {
                                if *count > 1 { format!("{}x{}", count, card) } else { format!("{}", card) }
                            })
                            .collect();
                        let hidden_text = if offer.wanting_hidden_count > 0 {
                            format!(" +{} hidden", offer.wanting_hidden_count)
                        } else {
                            String::new()
                        };
                        
                        row.spawn((
                            Text::new(format!("{}{}", wanting_text.join(", "), hidden_text)),
                            TextFont { font_size: 12.0, ..Default::default() },
                            TextColor(Color::srgb(0.9, 0.6, 0.3)),
                        ));
                    });
                    
                    // Action buttons (only for offers we can interact with)
                    if !is_my_offer && !is_accepted && human_player.is_some() {
                        card.spawn(Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::FlexEnd,
                            margin: UiRect::top(Val::Px(4.0)),
                            ..Default::default()
                        }).with_children(|buttons| {
                            buttons
                                .spawn((
                                    Button,
                                    AcceptOfferButton { offer: *offer_entity },
                                    Node {
                                        padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                                        ..Default::default()
                                    },
                                    BackgroundColor(Color::srgb(0.2, 0.5, 0.3)),
                                ))
                                .with_child((
                                    Text::new("Accept"),
                                    TextFont { font_size: 12.0, ..Default::default() },
                                    TextColor(Color::WHITE),
                                ));
                        });
                    }
                });
        }
    });
}

/// Handle "Done Trading" button click - ends trade phase for human player
pub fn handle_done_trading_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<DoneTradingButton>),
    >,
    mut trade_phase_state: ResMut<TradePhaseState>,
    trade_ui_state: Res<TradeUiState>,
    mut commands: Commands,
) {
    if trade_phase_state.human_done {
        return;
    }
    for (interaction, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.6, 0.3));
                trade_phase_state.human_done = true;
                
                // Remove CanTrade from human player to signal they're done
                if let Some(human) = trade_ui_state.human_player {
                    commands.entity(human).remove::<CanTrade>();
                }
                debug!("Human player finished trading");
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.6, 0.4, 0.3));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.5, 0.3, 0.2));
            }
        }
    }
}

/// Handle "Create Offer" button click - opens the offer creation modal
pub fn handle_create_offer_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<CreateOfferButton>),
    >,
    mut trade_phase_state: ResMut<TradePhaseState>,
) {
    if trade_phase_state.human_done {
        return;
    }
    for (interaction, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.6, 0.4));
                trade_phase_state.create_offer_modal_open = true;
                debug!("Opening Create Offer modal");
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.6, 0.4));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.5, 0.3));
            }
        }
    }
}

/// Visually disable buttons and show feedback when human is done trading
pub fn update_human_done_ui(
    trade_phase_state: Res<TradePhaseState>,
    mut done_btn_query: Query<(&mut BackgroundColor, &Children), With<DoneTradingButton>>,
    mut create_btn_query: Query<&mut BackgroundColor, (With<CreateOfferButton>, Without<DoneTradingButton>)>,
    mut text_query: Query<&mut Text>,
) {
    if !trade_phase_state.human_done {
        return;
    }
    let grayed = Color::srgb(0.3, 0.3, 0.3);
    for (mut bg, children) in done_btn_query.iter_mut() {
        *bg = BackgroundColor(grayed);
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                **text = "Waiting for AI players...".to_string();
            }
        }
    }
    for mut bg in create_btn_query.iter_mut() {
        *bg = BackgroundColor(grayed);
    }
}

/// Marker for the close button on the create offer modal
#[derive(Component, Default)]
pub struct CloseCreateOfferModalButton;

/// Button to adjust the count of a card in the create offer modal.
/// Used for both offering and wanting sections.
#[derive(Component)]
pub struct CardCountAdjustButton {
    pub card: TradeCard,
    pub delta: i32,         // +1 or -1
    pub is_offering: bool,  // true = offering side, false = wanting side
}

/// Display showing the current count for a card in the create offer modal.
#[derive(Component)]
pub struct CardCountDisplay {
    pub card: TradeCard,
    pub is_offering: bool,
}

/// Marker for hidden card count adjustment buttons
#[derive(Component)]
pub struct HiddenCountButton {
    pub is_offering: bool,  // true = offering side, false = wanting side
    pub delta: i32,         // +1 or -1
}

/// Marker for accept offer button in the offers list
#[derive(Component)]
pub struct AcceptOfferButton {
    pub offer: Entity,
}

/// Marker for the player trading status display
#[derive(Component, Default)]
pub struct PlayerTradingStatusDisplay;

/// Marker for the offer summary display
#[derive(Component, Default)]
pub struct OfferSummaryDisplay;

/// Marker for the validation status display
#[derive(Component, Default)]
pub struct OfferValidationDisplay;

/// Marker for the publish offer button
#[derive(Component, Default)]
pub struct PublishOfferButton;

/// Helper to spawn a `< - > CARD_NAME (count) < + >` row using UIBuilder
fn spawn_card_adjust_row(
    ui: &mut UIBuilder,
    card: TradeCard,
    is_offering: bool,
    label_suffix: &str,
) {
    let color = commodity_color(&card);
    let suffix = label_suffix.to_string();

    ui.add_row(|row| {
        row.width_percent(100.0).align_items_center().margin(UiRect::vertical(Val::Px(2.0)));

        // < button (decrement)
        row.add_button(
            "<", 30.0, 24.0,
            Color::srgb(0.4, 0.3, 0.3), 14.0, 4.0,
            CardCountAdjustButton { card, delta: -1, is_offering },
        );

        // Card name label with commodity color
        row.with_child(|label| {
            label.set_node(Node {
                padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                margin: UiRect::horizontal(Val::Px(4.0)),
                min_width: Val::Px(100.0),
                justify_content: JustifyContent::Center,
                ..Default::default()
            });
            label.bg_color(color);
            label.add_text_child(format!("{}", card), None, Some(12.0), Some(Color::WHITE));
        });

        // Count display
        row.with_child(|count| {
            count.set_node(Node {
                min_width: Val::Px(30.0),
                justify_content: JustifyContent::Center,
                ..Default::default()
            });
            count.insert(CardCountDisplay { card, is_offering });
            count.with_text("0", None, Some(14.0), Some(Color::WHITE), None, None);
        });

        // > button (increment)
        row.add_button(
            ">", 30.0, 24.0,
            Color::srgb(0.3, 0.4, 0.3), 14.0, 4.0,
            CardCountAdjustButton { card, delta: 1, is_offering },
        );

        // Suffix label (e.g. "/ 3 owned")
        if !suffix.is_empty() {
            row.add_text_child(format!("  {}", suffix), None, Some(11.0), Some(Color::srgb(0.5, 0.5, 0.5)));
        }
    });
}

/// Helper to spawn a hidden count row: label  < - >  count  < + >
fn spawn_hidden_count_row(ui: &mut UIBuilder, label_text: &str, is_offering: bool) {
    ui.add_row(|row| {
        row.width_percent(100.0).align_items_center().margin(UiRect::top(Val::Px(8.0)));

        row.add_text_child(label_text, None, Some(12.0), Some(Color::srgb(0.7, 0.7, 0.7)));

        row.add_button(
            "-", 30.0, 24.0,
            Color::srgb(0.4, 0.3, 0.3), 14.0, 4.0,
            HiddenCountButton { is_offering, delta: -1 },
        );

        row.with_child(|count| {
            count.insert(OfferHiddenCountDisplay { is_offering });
            count.with_text("0", None, Some(14.0), Some(Color::WHITE), None, None);
        });

        row.add_button(
            "+", 30.0, 24.0,
            Color::srgb(0.3, 0.4, 0.3), 14.0, 4.0,
            HiddenCountButton { is_offering, delta: 1 },
        );
    });
}

/// Spawn the create offer modal when the flag is set
pub fn spawn_create_offer_modal(
    commands: Commands,
    trade_phase_state: Res<TradePhaseState>,
    modal_query: Query<Entity, With<CreateOfferModal>>,
    trade_ui_state: Res<TradeUiState>,
    human_player_query: Query<(&Name, &PlayerTradeCards), With<IsHuman>>,
    mut create_offer_state: ResMut<CreateOfferState>,
    theme: Res<UiTheme>,
) {
    // Only spawn if modal should be open and doesn't exist yet
    if !trade_phase_state.create_offer_modal_open || !modal_query.is_empty() {
        return;
    }
    
    // Reset offer state when opening modal
    create_offer_state.reset();
    
    // Get human player's cards
    let (player_name, player_cards) = if let Some(human) = trade_ui_state.human_player {
        human_player_query.get(human).ok()
    } else {
        human_player_query.iter().next()
    }.unwrap_or_else(|| {
        panic!("No human player found for create offer modal");
    });
    
    // Commodity cards the player owns (sorted by value)
    let mut owned_commodities: Vec<(TradeCard, usize)> = player_cards.commodity_cards().into_iter().collect();
    owned_commodities.sort_by_key(|(card, _)| card.value());
    
    // All commodity types (for wanting section)
    let mut all_commodity_types: Vec<TradeCard> = TradeCard::iter()
        .filter(|c| c.is_commodity())
        .collect();
    all_commodity_types.sort_by_key(|c| c.value());
    
    // Calamity cards the player owns (tradeable ones)
    let owned_calamities: Vec<TradeCard> = player_cards.calamity_cards()
        .into_iter()
        .filter(|c| c.is_tradeable())
        .collect();
    
    let player_name = player_name.to_string();
    let mut ui = UIBuilder::new(commands, Some(theme.clone()));
    
    ui.insert(CreateOfferModal)
        .insert(GlobalZIndex(100))
        .set_node(Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(10.0),
            top: Val::Percent(5.0),
            width: Val::Percent(80.0),
            height: Val::Percent(90.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(16.0)),
            overflow: Overflow::scroll_y(),
            ..Default::default()
        })
        .bg_color(Color::srgba(0.12, 0.12, 0.18, 0.98));
    
    // Header row
    ui.add_row(|header| {
        header.width_percent(100.0).height_px(40.0)
            .justify_space_between().align_items_center()
            .margin_btm_px(12.0);
        
        header.add_text_child("Create Trade Offer", None, Some(22.0), Some(Color::srgb(0.9, 0.8, 0.3)));
        
        header.add_button(
            "X", 40.0, 30.0,
            Color::srgb(0.5, 0.2, 0.2), 16.0, 4.0,
            CloseCreateOfferModalButton,
        );
    });
    
    // Instructions
    ui.add_text_child(
        "Rules: Exactly 2 guaranteed cards each side. Minimum 3 total cards each side.",
        None, Some(12.0), Some(Color::srgb(0.6, 0.6, 0.6)),
    );
    
    // === WHAT YOU OFFER SECTION ===
    ui.add_column(|section| {
        section.width_percent(100.0)
            .margin(UiRect::top(Val::Px(12.0)))
            .padding_all_px(10.0)
            .bg_color(Color::srgba(0.15, 0.2, 0.15, 0.6));
        
        section.add_text_child(
            format!("WHAT YOU OFFER (Your Cards - {})", player_name),
            None, Some(15.0), Some(Color::srgb(0.5, 0.9, 0.5)),
        );
        section.add_text_child(
            "Use < > to adjust guaranteed card counts (max 2 total):",
            None, Some(11.0), Some(Color::srgb(0.6, 0.6, 0.6)),
        );
        
        // One row per commodity type the player owns
        for (card, count) in &owned_commodities {
            spawn_card_adjust_row(section, *card, true, &format!("/ {} owned", count));
        }
        
        // Calamity cards (can be hidden in trades)
        for card in &owned_calamities {
            spawn_card_adjust_row(section, *card, true, "(calamity)");
        }
        
        // Hidden cards count for offering
        spawn_hidden_count_row(section, "Hidden cards to offer: ", true);
    });
    
    // === WHAT YOU WANT SECTION ===
    ui.add_column(|section| {
        section.width_percent(100.0)
            .margin(UiRect::top(Val::Px(12.0)))
            .padding_all_px(10.0)
            .bg_color(Color::srgba(0.2, 0.15, 0.15, 0.6));
        
        section.add_text_child(
            "WHAT YOU WANT (Guaranteed cards you request)",
            None, Some(15.0), Some(Color::srgb(0.9, 0.6, 0.5)),
        );
        section.add_text_child(
            "Use < > to adjust guaranteed card counts (max 2 total):",
            None, Some(11.0), Some(Color::srgb(0.6, 0.6, 0.6)),
        );
        
        // One row per commodity type
        for card_type in &all_commodity_types {
            spawn_card_adjust_row(section, *card_type, false, "");
        }
        
        // Hidden cards count for wanting
        spawn_hidden_count_row(section, "Hidden cards wanted: ", false);
    });
    
    // === OFFER SUMMARY ===
    ui.add_column(|section| {
        section.width_percent(100.0)
            .margin(UiRect::top(Val::Px(12.0)))
            .padding_all_px(10.0)
            .bg_color(Color::srgba(0.1, 0.1, 0.15, 0.8));
        
        section.add_text_child("OFFER SUMMARY", None, Some(14.0), Some(Color::srgb(0.8, 0.8, 0.8)));
        
        section.with_child(|summary| {
            summary.insert(OfferSummaryDisplay);
            summary.with_text(
                "Offering: nothing | Wanting: nothing",
                None, Some(12.0), Some(Color::srgb(0.6, 0.6, 0.6)), None, None,
            );
        });
        
        section.with_child(|validation| {
            validation.insert(OfferValidationDisplay);
            validation.with_text(
                "Invalid: Need 2 guaranteed cards each side, 3+ total each side",
                None, Some(12.0), Some(Color::srgb(0.8, 0.4, 0.4)), None, None,
            );
        });
    });
    
    // === PUBLISH BUTTON ===
    ui.add_row(|btn_row| {
        btn_row.width_percent(100.0).height_px(50.0)
            .justify_center().align_items_center()
            .margin(UiRect::top(Val::Px(12.0)));
        
        btn_row.add_button(
            "Publish Offer", 200.0, 40.0,
            Color::srgb(0.3, 0.3, 0.3), 16.0, 6.0,
            PublishOfferButton,
        );
    });
    
    ui.build();
}

/// Marker for hidden count display text
#[derive(Component)]
pub struct OfferHiddenCountDisplay {
    pub is_offering: bool,
}

/// Handle close button on create offer modal
pub fn handle_close_create_offer_modal(
    mut interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<CloseCreateOfferModalButton>),
    >,
    mut trade_phase_state: ResMut<TradePhaseState>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            trade_phase_state.create_offer_modal_open = false;
            debug!("Closing Create Offer modal");
        }
    }
}

/// Despawn the create offer modal when the flag is cleared
pub fn despawn_create_offer_modal(
    mut commands: Commands,
    trade_phase_state: Res<TradePhaseState>,
    modal_query: Query<Entity, With<CreateOfferModal>>,
) {
    if trade_phase_state.create_offer_modal_open {
        return;
    }
    
    for entity in modal_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Handle < - > and < + > buttons for card counts in the create offer modal
pub fn handle_card_count_adjust(
    mut interaction_query: Query<
        (&Interaction, &CardCountAdjustButton),
        Changed<Interaction>,
    >,
    mut create_offer_state: ResMut<CreateOfferState>,
) {
    for (interaction, btn) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        
        if btn.is_offering {
            let current = create_offer_state.offering_guaranteed
                .get(&btn.card).copied().unwrap_or(0);
            let total = create_offer_state.guaranteed_offering_count();
            
            if btn.delta > 0 && total < 2 {
                *create_offer_state.offering_guaranteed.entry(btn.card).or_insert(0) += 1;
            } else if btn.delta < 0 && current > 0 {
                if current > 1 {
                    *create_offer_state.offering_guaranteed.get_mut(&btn.card).unwrap() -= 1;
                } else {
                    create_offer_state.offering_guaranteed.remove(&btn.card);
                }
            }
        } else {
            let current = create_offer_state.wanting_guaranteed
                .get(&btn.card).copied().unwrap_or(0);
            let total = create_offer_state.guaranteed_wanting_count();
            
            if btn.delta > 0 && total < 2 {
                *create_offer_state.wanting_guaranteed.entry(btn.card).or_insert(0) += 1;
            } else if btn.delta < 0 && current > 0 {
                if current > 1 {
                    *create_offer_state.wanting_guaranteed.get_mut(&btn.card).unwrap() -= 1;
                } else {
                    create_offer_state.wanting_guaranteed.remove(&btn.card);
                }
            }
        }
    }
}

/// Update the count displays next to each card in the create offer modal
pub fn update_card_count_displays(
    create_offer_state: Res<CreateOfferState>,
    mut displays: Query<(&CardCountDisplay, &mut Text)>,
) {
    for (display, mut text) in displays.iter_mut() {
        let count = if display.is_offering {
            create_offer_state.offering_guaranteed.get(&display.card).copied().unwrap_or(0)
        } else {
            create_offer_state.wanting_guaranteed.get(&display.card).copied().unwrap_or(0)
        };
        **text = count.to_string();
    }
}

/// Handle hidden count +/- buttons
pub fn handle_hidden_count_buttons(
    mut interaction_query: Query<
        (&Interaction, &HiddenCountButton),
        Changed<Interaction>,
    >,
    mut create_offer_state: ResMut<CreateOfferState>,
) {
    for (interaction, btn) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            if btn.is_offering {
                let new_val = create_offer_state.offering_hidden_count as i32 + btn.delta;
                if new_val >= 0 {
                    create_offer_state.offering_hidden_count = new_val as usize;
                }
            } else {
                let new_val = create_offer_state.wanting_hidden_count as i32 + btn.delta;
                if new_val >= 0 {
                    create_offer_state.wanting_hidden_count = new_val as usize;
                }
            }
        }
    }
}

/// Update hidden count displays
pub fn update_hidden_count_displays(
    create_offer_state: Res<CreateOfferState>,
    mut displays: Query<(&OfferHiddenCountDisplay, &mut Text)>,
) {
    for (display, mut text) in displays.iter_mut() {
        let count = if display.is_offering {
            create_offer_state.offering_hidden_count
        } else {
            create_offer_state.wanting_hidden_count
        };
        **text = count.to_string();
    }
}

/// Handle clicking the Accept button on an offer
pub fn handle_accept_offer_button(
    mut interaction_query: Query<
        (&Interaction, &AcceptOfferButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut offers_query: Query<&mut OpenTradeOffer>,
    trade_ui_state: Res<TradeUiState>,
    player_names: Query<&Name>,
    player_cards_query: Query<&PlayerTradeCards>,
) {
    for (interaction, accept_btn, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.7, 0.4));
                
                if let Some(human) = trade_ui_state.human_player {
                    if let Ok(mut offer) = offers_query.get_mut(accept_btn.offer) {
                        if offer.can_accept(human) {
                            // Validate that human can fulfill the trade requirements
                            if let Ok(player_cards) = player_cards_query.get(human) {
                                let player_commodities: HashMap<TradeCard, usize> = 
                                    player_cards.commodity_cards().into_iter().collect();
                                
                                // Check if player has the required guaranteed cards
                                let mut can_fulfill = true;
                                for (card, count) in offer.wanting_guaranteed.iter() {
                                    let player_has = player_commodities.get(card).copied().unwrap_or(0);
                                    if player_has < *count {
                                        can_fulfill = false;
                                        debug!("Cannot accept: need {} {:?} but only have {}", count, card, player_has);
                                        break;
                                    }
                                }
                                
                                // Check total card count
                                let total_needed = offer.wanting_guaranteed.values().sum::<usize>() + offer.wanting_hidden_count;
                                let total_have = player_cards.number_of_trade_cards();
                                if total_have < total_needed {
                                    can_fulfill = false;
                                    debug!("Cannot accept: need {} total cards but only have {}", total_needed, total_have);
                                }
                                
                                if can_fulfill {
                                    let human_name = player_names
                                        .get(human)
                                        .map(|n| n.to_string())
                                        .unwrap_or_else(|_| "Human".to_string());
                                    offer.accept(human, human_name);
                                    debug!("Human accepted offer from {}", offer.creator_name);
                                } else {
                                    debug!("Human cannot fulfill trade requirements");
                                }
                            }
                        }
                    }
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.55, 0.35));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.5, 0.3));
            }
        }
    }
}

/// Update offer summary and validation displays
pub fn update_offer_summary_display(
    create_offer_state: Res<CreateOfferState>,
    mut summary_query: Query<&mut Text, (With<OfferSummaryDisplay>, Without<OfferValidationDisplay>)>,
    mut validation_query: Query<(&mut Text, &mut TextColor), (With<OfferValidationDisplay>, Without<OfferSummaryDisplay>)>,
    mut publish_btn: Query<(&mut BackgroundColor, &Children), With<PublishOfferButton>>,
    mut text_color_query: Query<&mut TextColor, (Without<OfferValidationDisplay>, Without<OfferSummaryDisplay>)>,
) {
    // Build offering text
    let offering_text = if create_offer_state.offering_guaranteed.is_empty() && create_offer_state.offering_hidden_count == 0 {
        "nothing".to_string()
    } else {
        let mut parts = Vec::new();
        for (card, count) in create_offer_state.offering_guaranteed.iter() {
            parts.push(format!("{}x {}", count, card));
        }
        if create_offer_state.offering_hidden_count > 0 {
            parts.push(format!("+{} hidden", create_offer_state.offering_hidden_count));
        }
        parts.join(", ")
    };
    
    // Build wanting text
    let wanting_text = if create_offer_state.wanting_guaranteed.is_empty() && create_offer_state.wanting_hidden_count == 0 {
        "nothing".to_string()
    } else {
        let mut parts = Vec::new();
        for (card, count) in create_offer_state.wanting_guaranteed.iter() {
            parts.push(format!("{}x {}", count, card));
        }
        if create_offer_state.wanting_hidden_count > 0 {
            parts.push(format!("+{} hidden", create_offer_state.wanting_hidden_count));
        }
        parts.join(", ")
    };
    
    // Update summary
    for mut text in summary_query.iter_mut() {
        **text = format!("Offering: {} | Wanting: {}", offering_text, wanting_text);
    }
    
    // Update validation
    let is_valid = create_offer_state.is_valid();
    for (mut text, mut color) in validation_query.iter_mut() {
        if is_valid {
            **text = "✅ Valid offer - ready to publish!".to_string();
            *color = TextColor(Color::srgb(0.4, 0.8, 0.4));
        } else {
            let mut issues = Vec::new();
            if create_offer_state.guaranteed_offering_count() != 2 {
                issues.push(format!("need 2 guaranteed offering (have {})", create_offer_state.guaranteed_offering_count()));
            }
            if create_offer_state.guaranteed_wanting_count() != 2 {
                issues.push(format!("need 2 guaranteed wanting (have {})", create_offer_state.guaranteed_wanting_count()));
            }
            if create_offer_state.total_offering() < 3 {
                issues.push(format!("need 3+ total offering (have {})", create_offer_state.total_offering()));
            }
            if create_offer_state.total_wanting() < 3 {
                issues.push(format!("need 3+ total wanting (have {})", create_offer_state.total_wanting()));
            }
            **text = format!("❌ Invalid: {}", issues.join(", "));
            *color = TextColor(Color::srgb(0.8, 0.4, 0.4));
        }
    }
    
    // Update publish button appearance
    for (mut bg, children) in publish_btn.iter_mut() {
        if is_valid {
            *bg = BackgroundColor(Color::srgb(0.2, 0.6, 0.3));
            for child in children.iter() {
                if let Ok(mut text_color) = text_color_query.get_mut(child) {
                    *text_color = TextColor(Color::WHITE);
                }
            }
        } else {
            *bg = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));
            for child in children.iter() {
                if let Ok(mut text_color) = text_color_query.get_mut(child) {
                    *text_color = TextColor(Color::srgb(0.5, 0.5, 0.5));
                }
            }
        }
    }
}

/// Handle publish offer button - creates the OpenTradeOffer entity
pub fn handle_publish_offer(
    mut commands: Commands,
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<PublishOfferButton>)>,
    create_offer_state: Res<CreateOfferState>,
    mut trade_phase_state: ResMut<TradePhaseState>,
    trade_ui_state: Res<TradeUiState>,
    human_query: Query<&Name, With<IsHuman>>,
) {
    for interaction in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        
        if !create_offer_state.is_valid() {
            debug!("Cannot publish invalid offer");
            continue;
        }
        
        let Some(human_entity) = trade_ui_state.human_player else {
            debug!("No human player to create offer");
            continue;
        };
        
        let human_name = human_query.get(human_entity)
            .map(|n| n.to_string())
            .unwrap_or_else(|_| "Player".to_string());
        
        // Create the offer entity
        let mut offer = OpenTradeOffer::new(human_entity, human_name, None, None);
        offer.offering_guaranteed = create_offer_state.offering_guaranteed.clone();
        offer.offering_hidden_count = create_offer_state.offering_hidden_count;
        offer.wanting_guaranteed = create_offer_state.wanting_guaranteed.clone();
        offer.wanting_hidden_count = create_offer_state.wanting_hidden_count;
        
        commands.spawn(offer);
        
        debug!("Published trade offer");
        
        // Close the modal
        trade_phase_state.create_offer_modal_open = false;
    }
}

// ============================================================================
// SETTLEMENT MODAL
// ============================================================================

/// Marker for the close button on the settlement modal
#[derive(Component, Default)]
pub struct CloseSettlementModalButton;

/// Marker for the confirm settlement button
#[derive(Component, Default)]
pub struct ConfirmSettlementButton;

/// Marker for a card button in the settlement modal that can be selected
#[derive(Component)]
pub struct SettlementCardButton {
    pub card: TradeCard,
    pub selected: bool,
}

/// Container for selected cards display in settlement modal
#[derive(Component, Default)]
pub struct SettlementSelectedCardsDisplay;

/// Resource to track selected cards during settlement
#[derive(Resource, Default)]
pub struct SettlementSelection {
    pub selected_cards: HashMap<TradeCard, usize>,
}

/// Check if any accepted offer needs settlement and open the modal
pub fn check_for_settlement_needed(
    offers_query: Query<(Entity, &OpenTradeOffer)>,
    trade_ui_state: Res<TradeUiState>,
    mut trade_phase_state: ResMut<TradePhaseState>,
) {
    // Don't open if already open or no human player
    if trade_phase_state.settlement_modal_open {
        return;
    }
    
    let Some(human) = trade_ui_state.human_player else {
        return;
    };
    
    // Find an accepted offer where human is involved and hasn't settled yet
    for (offer_entity, offer) in offers_query.iter() {
        if !offer.is_settling() {
            continue;
        }
        
        // Check if human is creator and hasn't settled
        if offer.creator == human && offer.creator_actual_cards.is_none() {
            trade_phase_state.settlement_modal_open = true;
            trade_phase_state.settling_offer_entity = Some(offer_entity);
            debug!("Opening settlement modal for human as creator");
            return;
        }
        
        // Check if human is acceptor and hasn't settled
        if offer.accepted_by == Some(human) && offer.acceptor_actual_cards.is_none() {
            trade_phase_state.settlement_modal_open = true;
            trade_phase_state.settling_offer_entity = Some(offer_entity);
            debug!("Opening settlement modal for human as acceptor");
            return;
        }
    }
}

/// Spawn the settlement modal when the flag is set
pub fn spawn_settlement_modal(
    mut commands: Commands,
    trade_phase_state: Res<TradePhaseState>,
    modal_query: Query<Entity, With<SettlementModal>>,
    trade_ui_state: Res<TradeUiState>,
    human_player_query: Query<(&Name, &PlayerTradeCards), With<IsHuman>>,
    offers_query: Query<&OpenTradeOffer>,
) {
    // Only spawn if modal should be open and doesn't exist yet
    if !trade_phase_state.settlement_modal_open || !modal_query.is_empty() {
        return;
    }
    
    let Some(offer_entity) = trade_phase_state.settling_offer_entity else {
        return;
    };
    
    let Ok(offer) = offers_query.get(offer_entity) else {
        return;
    };
    
    // Get human player's cards
    let (player_name, player_cards) = if let Some(human) = trade_ui_state.human_player {
        human_player_query.get(human).ok()
    } else {
        human_player_query.iter().next()
    }.unwrap_or_else(|| {
        panic!("No human player found for settlement modal");
    });
    
    // Determine if human is creator or acceptor
    let is_creator = trade_ui_state.human_player == Some(offer.creator);
    let (required_guaranteed, required_hidden) = if is_creator {
        (&offer.offering_guaranteed, offer.offering_hidden_count)
    } else {
        (&offer.wanting_guaranteed, offer.wanting_hidden_count)
    };
    
    let total_required = required_guaranteed.values().sum::<usize>() + required_hidden;
    let role_text = if is_creator { "You offered" } else { "You must provide" };
    
    // Initialize settlement selection resource
    commands.insert_resource(SettlementSelection::default());
    
    commands
        .spawn((
            SettlementModal,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(20.0),
                top: Val::Percent(10.0),
                width: Val::Percent(60.0),
                height: Val::Percent(80.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                ..Default::default()
            },
            BackgroundColor(Color::srgba(0.12, 0.15, 0.2, 0.98)),
            GlobalZIndex(150),
        ))
        .with_children(|modal| {
            // Header with title and close button
            modal
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(50.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(16.0)),
                    ..Default::default()
                })
                .with_children(|header| {
                    header.spawn((
                        Text::new("🤝 Settle Trade"),
                        TextFont { font_size: 26.0, ..Default::default() },
                        TextColor(Color::srgb(0.3, 0.8, 0.5)),
                    ));
                    
                    // Close button (cancels settlement)
                    header
                        .spawn((
                            Button,
                            CloseSettlementModalButton,
                            Node {
                                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                                ..Default::default()
                            },
                            BackgroundColor(Color::srgb(0.5, 0.2, 0.2)),
                        ))
                        .with_child((
                            Text::new("✕ Cancel"),
                            TextFont { font_size: 14.0, ..Default::default() },
                            TextColor(Color::WHITE),
                        ));
                });
            
            // Trade summary
            modal
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(12.0)),
                    margin: UiRect::bottom(Val::Px(16.0)),
                    ..Default::default()
                })
                .insert(BackgroundColor(Color::srgba(0.1, 0.12, 0.15, 0.8)))
                .with_children(|summary| {
                    // Role description
                    summary.spawn((
                        Text::new(format!("{}: {} cards total", role_text, total_required)),
                        TextFont { font_size: 16.0, ..Default::default() },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    ));
                    
                    // Guaranteed cards requirement
                    let guaranteed_text: Vec<String> = required_guaranteed
                        .iter()
                        .map(|(card, count)| format!("{}x {}", count, card))
                        .collect();
                    
                    summary.spawn((
                        Text::new(format!("Guaranteed (must include): {}", guaranteed_text.join(", "))),
                        TextFont { font_size: 14.0, ..Default::default() },
                        TextColor(Color::srgb(0.9, 0.7, 0.3)),
                    ));
                    
                    if required_hidden > 0 {
                        summary.spawn((
                            Text::new(format!("+ {} hidden cards (your choice)", required_hidden)),
                            TextFont { font_size: 14.0, ..Default::default() },
                            TextColor(Color::srgb(0.6, 0.6, 0.6)),
                        ));
                    }
                });
            
            // Instructions
            modal.spawn((
                Text::new("Click cards below to select them for this trade:"),
                TextFont { font_size: 14.0, ..Default::default() },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
            
            // Your cards section (clickable)
            modal
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::top(Val::Px(8.0)),
                    ..Default::default()
                })
                .with_children(|section| {
                    section.spawn((
                        Text::new(format!("Your Cards ({}):", player_name)),
                        TextFont { font_size: 14.0, ..Default::default() },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                    ));
                    
                    section
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            margin: UiRect::top(Val::Px(8.0)),
                            padding: UiRect::all(Val::Px(8.0)),
                            ..Default::default()
                        })
                        .insert(BackgroundColor(Color::srgba(0.08, 0.08, 0.1, 0.6)))
                        .with_children(|cards_row| {
                            // Commodity cards
                            for (card, count) in player_cards.commodity_cards() {
                                for _ in 0..count {
                                    let color = commodity_color(&card);
                                    cards_row
                                        .spawn((
                                            Button,
                                            SettlementCardButton { card, selected: false },
                                            Node {
                                                padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                                                margin: UiRect::all(Val::Px(3.0)),
                                                border: UiRect::all(Val::Px(2.0)),
                                                ..Default::default()
                                            },
                                            BackgroundColor(color),
                                            BorderColor::all(Color::NONE),
                                        ))
                                        .with_child((
                                            Text::new(format!("{}", card)),
                                            TextFont { font_size: 12.0, ..Default::default() },
                                            TextColor(Color::WHITE),
                                        ));
                                }
                            }
                            // Calamity cards
                            for card in player_cards.calamity_cards() {
                                cards_row
                                    .spawn((
                                        Button,
                                        SettlementCardButton { card, selected: false },
                                        Node {
                                            padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                                            margin: UiRect::all(Val::Px(3.0)),
                                            border: UiRect::all(Val::Px(2.0)),
                                            ..Default::default()
                                        },
                                        BackgroundColor(Color::srgb(0.7, 0.15, 0.15)),
                                        BorderColor::all(Color::NONE),
                                    ))
                                    .with_child((
                                        Text::new(format!("{}", card)),
                                        TextFont { font_size: 12.0, ..Default::default() },
                                        TextColor(Color::WHITE),
                                    ));
                            }
                        });
                });
            
            // Selected cards display
            modal
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::top(Val::Px(16.0)),
                    padding: UiRect::all(Val::Px(12.0)),
                    flex_grow: 1.0,
                    ..Default::default()
                })
                .insert(BackgroundColor(Color::srgba(0.1, 0.15, 0.1, 0.6)))
                .with_children(|selected_section| {
                    selected_section.spawn((
                        Text::new("Selected Cards (0):"),
                        TextFont { font_size: 14.0, ..Default::default() },
                        TextColor(Color::srgb(0.5, 0.8, 0.5)),
                    ));
                    
                    selected_section.spawn((
                        SettlementSelectedCardsDisplay,
                        Text::new("None selected yet"),
                        TextFont { font_size: 13.0, ..Default::default() },
                        TextColor(Color::srgb(0.5, 0.5, 0.5)),
                    ));
                });
            
            // Bottom buttons row
            modal
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(50.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(16.0)),
                    ..Default::default()
                })
                .with_children(|btn_row| {
                    // Cancel button (prominent, on the left)
                    btn_row
                        .spawn((
                            Button,
                            CloseSettlementModalButton,
                            Node {
                                padding: UiRect::axes(Val::Px(30.0), Val::Px(14.0)),
                                ..Default::default()
                            },
                            BackgroundColor(Color::srgb(0.6, 0.2, 0.2)),
                        ))
                        .with_child((
                            Text::new("✕ Cancel Trade"),
                            TextFont { font_size: 16.0, ..Default::default() },
                            TextColor(Color::WHITE),
                        ));
                    
                    // Confirm button (on the right)
                    btn_row
                        .spawn((
                            Button,
                            ConfirmSettlementButton,
                            Node {
                                padding: UiRect::axes(Val::Px(30.0), Val::Px(14.0)),
                                ..Default::default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                        ))
                        .with_child((
                            Text::new("✓ Confirm Settlement"),
                            TextFont { font_size: 16.0, ..Default::default() },
                            TextColor(Color::srgb(0.6, 0.6, 0.6)),
                        ));
                });
        });
}

/// Handle card selection in settlement modal
pub fn handle_settlement_card_selection(
    mut interaction_query: Query<
        (&Interaction, &mut SettlementCardButton, &mut BorderColor),
        Changed<Interaction>,
    >,
    mut settlement_selection: Option<ResMut<SettlementSelection>>,
) {
    let Some(ref mut selection) = settlement_selection else {
        return;
    };
    
    for (interaction, mut card_btn, mut border) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            card_btn.selected = !card_btn.selected;
            
            if card_btn.selected {
                *border = BorderColor::all(Color::srgb(0.3, 0.9, 0.4));
                *selection.selected_cards.entry(card_btn.card).or_insert(0) += 1;
            } else {
                *border = BorderColor::all(Color::NONE);
                if let Some(count) = selection.selected_cards.get_mut(&card_btn.card) {
                    if *count > 1 {
                        *count -= 1;
                    } else {
                        selection.selected_cards.remove(&card_btn.card);
                    }
                }
            }
        }
    }
}

/// Update the selected cards display and confirm button state
pub fn update_settlement_display(
    settlement_selection: Option<Res<SettlementSelection>>,
    trade_phase_state: Res<TradePhaseState>,
    offers_query: Query<&OpenTradeOffer>,
    trade_ui_state: Res<TradeUiState>,
    mut selected_display: Query<&mut Text, With<SettlementSelectedCardsDisplay>>,
    mut confirm_btn: Query<(&mut BackgroundColor, &Children), With<ConfirmSettlementButton>>,
    mut text_query: Query<&mut TextColor, Without<SettlementSelectedCardsDisplay>>,
) {
    let Some(ref selection) = settlement_selection else {
        return;
    };
    
    let Some(offer_entity) = trade_phase_state.settling_offer_entity else {
        return;
    };
    
    let Ok(offer) = offers_query.get(offer_entity) else {
        return;
    };
    
    // Determine requirements
    let is_creator = trade_ui_state.human_player == Some(offer.creator);
    let (required_guaranteed, required_hidden) = if is_creator {
        (&offer.offering_guaranteed, offer.offering_hidden_count)
    } else {
        (&offer.wanting_guaranteed, offer.wanting_hidden_count)
    };
    
    let total_required = required_guaranteed.values().sum::<usize>() + required_hidden;
    let total_selected: usize = selection.selected_cards.values().sum();
    
    // Check if selection is valid
    let mut is_valid = total_selected == total_required;
    
    // Check guaranteed cards are included
    for (card, required_count) in required_guaranteed.iter() {
        let selected_count = selection.selected_cards.get(card).copied().unwrap_or(0);
        if selected_count < *required_count {
            is_valid = false;
            break;
        }
    }
    
    // Update display text
    for mut text in selected_display.iter_mut() {
        if selection.selected_cards.is_empty() {
            **text = "None selected yet".to_string();
        } else {
            let cards_text: Vec<String> = selection.selected_cards
                .iter()
                .map(|(card, count)| format!("{}x {}", count, card))
                .collect();
            **text = format!("Selected ({}/{}): {}", total_selected, total_required, cards_text.join(", "));
        }
    }
    
    // Update confirm button appearance based on validity
    for (mut bg, children) in confirm_btn.iter_mut() {
        if is_valid {
            *bg = BackgroundColor(Color::srgb(0.2, 0.6, 0.3));
            for child in children.iter() {
                if let Ok(mut text_color) = text_query.get_mut(child) {
                    *text_color = TextColor(Color::WHITE);
                }
            }
        } else {
            *bg = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));
            for child in children.iter() {
                if let Ok(mut text_color) = text_query.get_mut(child) {
                    *text_color = TextColor(Color::srgb(0.6, 0.6, 0.6));
                }
            }
        }
    }
}

/// Handle confirm settlement button
pub fn handle_confirm_settlement(
    mut interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<ConfirmSettlementButton>),
    >,
    settlement_selection: Option<Res<SettlementSelection>>,
    mut trade_phase_state: ResMut<TradePhaseState>,
    trade_ui_state: Res<TradeUiState>,
    mut offers_query: Query<&mut OpenTradeOffer>,
) {
    let Some(ref selection) = settlement_selection else {
        return;
    };
    
    for interaction in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        
        let Some(offer_entity) = trade_phase_state.settling_offer_entity else {
            continue;
        };
        
        let Ok(mut offer) = offers_query.get_mut(offer_entity) else {
            continue;
        };
        
        // Validate selection
        let is_creator = trade_ui_state.human_player == Some(offer.creator);
        let (required_guaranteed, required_hidden) = if is_creator {
            (&offer.offering_guaranteed, offer.offering_hidden_count)
        } else {
            (&offer.wanting_guaranteed, offer.wanting_hidden_count)
        };
        
        let total_required = required_guaranteed.values().sum::<usize>() + required_hidden;
        let total_selected: usize = selection.selected_cards.values().sum();
        
        if total_selected != total_required {
            debug!("Settlement invalid: wrong card count");
            continue;
        }
        
        // Check guaranteed cards
        let mut valid = true;
        for (card, required_count) in required_guaranteed.iter() {
            let selected_count = selection.selected_cards.get(card).copied().unwrap_or(0);
            if selected_count < *required_count {
                valid = false;
                break;
            }
        }
        
        if !valid {
            debug!("Settlement invalid: missing guaranteed cards");
            continue;
        }
        
        // Apply settlement
        if is_creator {
            offer.settle_creator(selection.selected_cards.clone());
            debug!("Creator settled trade");
        } else {
            offer.settle_acceptor(selection.selected_cards.clone());
            debug!("Acceptor settled trade");
        }
        
        // Close modal
        trade_phase_state.settlement_modal_open = false;
        trade_phase_state.settling_offer_entity = None;
    }
}

/// Handle close button on settlement modal
pub fn handle_close_settlement_modal(
    mut interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<CloseSettlementModalButton>),
    >,
    mut trade_phase_state: ResMut<TradePhaseState>,
    mut offers_query: Query<&mut OpenTradeOffer>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            // Cancel the trade by withdrawing the offer
            if let Some(offer_entity) = trade_phase_state.settling_offer_entity {
                if let Ok(mut offer) = offers_query.get_mut(offer_entity) {
                    offer.withdrawn = true;
                    debug!("Trade cancelled during settlement");
                }
            }
            
            trade_phase_state.settlement_modal_open = false;
            trade_phase_state.settling_offer_entity = None;
            debug!("Closing Settlement modal");
        }
    }
}

/// Despawn the settlement modal when the flag is cleared
pub fn despawn_settlement_modal(
    mut commands: Commands,
    trade_phase_state: Res<TradePhaseState>,
    modal_query: Query<Entity, With<SettlementModal>>,
) {
    if trade_phase_state.settlement_modal_open {
        return;
    }
    
    for entity in modal_query.iter() {
        commands.entity(entity).despawn();
    }
    
    // Clean up selection resource
    commands.remove_resource::<SettlementSelection>();
}

// ============================================================================
// AI TRADE BEHAVIOR
// ============================================================================

/// AI creates trade offers based on their cards and needs
pub fn ai_create_trade_offers(
    mut commands: Commands,
    ai_players: Query<(Entity, &Name, &PlayerTradeCards, &CanTrade), Without<IsHuman>>,
    existing_offers: Query<&OpenTradeOffer>,
    time: Res<Time>,
    mut ai_offer_timer: Local<f32>,
) {
    // Only create offers periodically (every 3-5 seconds)
    *ai_offer_timer += time.delta_secs();
    if *ai_offer_timer < 3.0 {
        return;
    }
    *ai_offer_timer = 0.0;
    
    for (ai_entity, ai_name, ai_cards, _) in ai_players.iter() {
        // Check if AI already has an active offer
        let has_active_offer = existing_offers.iter().any(|o| {
            o.creator == ai_entity && !o.withdrawn && o.accepted_by.is_none()
        });
        
        if has_active_offer {
            continue;
        }
        
        // Get AI's commodity cards
        let commodities: Vec<(TradeCard, usize)> = ai_cards.commodity_cards().into_iter().collect();
        if commodities.len() < 2 {
            continue; // Not enough cards to trade
        }
        
        // Pick 2 random commodities to offer as guaranteed
        let mut offering_guaranteed: HashMap<TradeCard, usize> = HashMap::default();
        let mut cards_offered = 0;
        
        // Prefer offering lower-value cards
        let mut sorted_commodities: Vec<_> = commodities.iter()
            .filter(|(_, count)| *count > 0)
            .collect();
        sorted_commodities.sort_by_key(|(card, _)| card.value());
        
        for (card, count) in sorted_commodities.iter().take(3) {
            if cards_offered >= 2 {
                break;
            }
            let to_offer = (*count).min(2 - cards_offered);
            if to_offer > 0 {
                offering_guaranteed.insert(*card, to_offer);
                cards_offered += to_offer;
            }
        }
        
        if cards_offered < 2 {
            continue;
        }
        
        // Pick what AI wants — only request cards up to their own max card value.
        // An AI with max value 4 cards should never ask for Gold (9) or Ivory (9).
        let mut wanting_guaranteed: HashMap<TradeCard, usize> = HashMap::default();
        let mut cards_wanted = 0;
        
        let ai_card_counts: HashMap<TradeCard, usize> = commodities.iter().cloned().collect();
        let max_own_value = commodities.iter()
            .map(|(card, _)| card.value())
            .max()
            .unwrap_or(0);
        
        // Want commodities that: (a) are at or below our max value, (b) we have few of,
        // (c) prefer higher value within that range (to try completing sets)
        let mut desired: Vec<TradeCard> = TradeCard::iter()
            .filter(|c| c.is_commodity())
            .filter(|c| c.value() <= max_own_value)
            .filter(|c| ai_card_counts.get(c).copied().unwrap_or(0) < 2)
            .collect();
        desired.sort_by_key(|c| std::cmp::Reverse(c.value()));
        
        for card in desired.iter().take(3) {
            if cards_wanted >= 2 {
                break;
            }
            let to_want = 1.min(2 - cards_wanted);
            wanting_guaranteed.insert(*card, to_want);
            cards_wanted += to_want;
        }
        
        if cards_wanted < 2 {
            // Fallback: want any commodity at or below our max value
            for card in TradeCard::iter()
                .filter(|c| c.is_commodity())
                .filter(|c| c.value() <= max_own_value)
            {
                if cards_wanted >= 2 {
                    break;
                }
                if !wanting_guaranteed.contains_key(&card) {
                    wanting_guaranteed.insert(card, 1);
                    cards_wanted += 1;
                }
            }
        }
        
        // Add hidden cards to reach minimum 3 each side
        let offering_hidden = 3_usize.saturating_sub(cards_offered);
        let wanting_hidden = 3_usize.saturating_sub(cards_wanted);
        
        // Create the offer
        let mut offer = OpenTradeOffer::new(ai_entity, ai_name.to_string(), None, None);
        offer.offering_guaranteed = offering_guaranteed;
        offer.offering_hidden_count = offering_hidden;
        offer.wanting_guaranteed = wanting_guaranteed;
        offer.wanting_hidden_count = wanting_hidden;
        
        if offer.is_valid() {
            commands.spawn(offer);
            debug!("{} created a trade offer", ai_name);
        }
    }
}

/// AI accepts trade offers that benefit them.
/// Criteria: The trade must either increase the AI's total stack value OR enable trading away a calamity.
pub fn ai_accept_trade_offers(
    ai_players: Query<(Entity, &Name, &PlayerTradeCards, &CanTrade), Without<IsHuman>>,
    mut offers: Query<(Entity, &mut OpenTradeOffer)>,
) {
    for (ai_entity, ai_name, ai_cards, _) in ai_players.iter() {
        for (_offer_entity, mut offer) in offers.iter_mut() {
            // Skip if we can't accept
            if !offer.can_accept(ai_entity) {
                continue;
            }
            
            // Check if AI has the cards to fulfill the trade
            let ai_commodities: HashMap<TradeCard, usize> = ai_cards.commodity_cards().into_iter().collect();
            
            // Check if AI can provide the wanted guaranteed cards
            let mut can_fulfill = true;
            for (card, count) in offer.wanting_guaranteed.iter() {
                let ai_has = ai_commodities.get(card).copied().unwrap_or(0);
                if ai_has < *count {
                    can_fulfill = false;
                    break;
                }
            }
            
            if !can_fulfill {
                continue;
            }
            
            // Check if AI has enough total cards
            let total_ai_cards: usize = ai_commodities.values().sum();
            if total_ai_cards < offer.total_wanting() {
                continue;
            }
            
            // Calculate current stack value
            let current_stack_value = ai_cards.total_stack_value();
            
            // Simulate the trade to calculate new stack value
            // We receive: offering_guaranteed cards, we give: wanting_guaranteed cards
            let mut simulated_cards = ai_cards.clone();
            
            // Remove cards we would give away
            for (card, count) in offer.wanting_guaranteed.iter() {
                simulated_cards.remove_n_trade_cards(*count, *card);
            }
            
            // Add cards we would receive
            for (card, count) in offer.offering_guaranteed.iter() {
                simulated_cards.add_trade_cards(*card, *count);
            }
            
            let new_stack_value = simulated_cards.total_stack_value();
            
            // Check if this trade enables trading away a calamity (hidden card slot)
            let can_trade_away_calamity = ai_cards.has_tradeable_calamity() && offer.wanting_hidden_count > 0;
            
            // Accept if: trade increases stack value OR enables trading away a calamity
            if new_stack_value > current_stack_value || can_trade_away_calamity {
                offer.accept(ai_entity, ai_name.to_string());
                debug!("{} accepted trade offer from {} (value: {} -> {}, calamity: {})", 
                    ai_name, offer.creator_name, current_stack_value, new_stack_value, can_trade_away_calamity);
                break; // Only accept one offer per frame
            }
        }
    }
}

/// AI settles trades by selecting cards
pub fn ai_settle_trades(
    ai_players: Query<(Entity, &Name, &PlayerTradeCards), Without<IsHuman>>,
    mut offers: Query<&mut OpenTradeOffer>,
) {
    for (ai_entity, ai_name, ai_cards) in ai_players.iter() {
        for mut offer in offers.iter_mut() {
            if !offer.is_settling() {
                continue;
            }
            
            // Check if AI is creator and needs to settle
            if offer.creator == ai_entity && offer.creator_actual_cards.is_none() {
                let cards = ai_select_settlement_cards(
                    ai_cards,
                    &offer.offering_guaranteed,
                    offer.offering_hidden_count,
                );
                if let Some(cards) = cards {
                    offer.settle_creator(cards);
                    debug!("{} settled trade as creator", ai_name);
                }
            }
            
            // Check if AI is acceptor and needs to settle
            if offer.accepted_by == Some(ai_entity) && offer.acceptor_actual_cards.is_none() {
                let cards = ai_select_settlement_cards(
                    ai_cards,
                    &offer.wanting_guaranteed,
                    offer.wanting_hidden_count,
                );
                if let Some(cards) = cards {
                    offer.settle_acceptor(cards);
                    debug!("{} settled trade as acceptor", ai_name);
                }
            }
        }
    }
}

/// Helper: AI selects cards for settlement
fn ai_select_settlement_cards(
    player_cards: &PlayerTradeCards,
    required_guaranteed: &HashMap<TradeCard, usize>,
    hidden_count: usize,
) -> Option<HashMap<TradeCard, usize>> {
    let mut selected: HashMap<TradeCard, usize> = HashMap::default();
    let mut available: HashMap<TradeCard, usize> = player_cards.commodity_cards().into_iter().collect();
    
    // First, add all required guaranteed cards
    for (card, count) in required_guaranteed.iter() {
        let have = available.get(card).copied().unwrap_or(0);
        if have < *count {
            return None; // Can't fulfill
        }
        selected.insert(*card, *count);
        if have > *count {
            available.insert(*card, have - count);
        } else {
            available.remove(card);
        }
    }
    
    // Add hidden cards - prefer calamities first, then lowest value commodities
    let mut hidden_left = hidden_count;
    
    // Try to add tradeable calamities
    for calamity in player_cards.calamity_cards() {
        if hidden_left == 0 {
            break;
        }
        if calamity.is_tradeable() {
            *selected.entry(calamity).or_insert(0) += 1;
            hidden_left -= 1;
        }
    }
    
    // Add the lowest value commodities for remaining hidden
    let mut sorted_available: Vec<_> = available.iter().collect();
    sorted_available.sort_by_key(|(card, _)| card.value());
    
    for (card, count) in sorted_available {
        if hidden_left == 0 {
            break;
        }
        let to_add = (*count).min(hidden_left);
        *selected.entry(*card).or_insert(0) += to_add;
        hidden_left -= to_add;
    }
    
    if hidden_left > 0 {
        return None; // Couldn't fulfill hidden requirement
    }
    
    Some(selected)
}
