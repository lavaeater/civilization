use crate::civilization::concepts::trade::components::{
    CanTrade, InSettlement, NeedsTradeMove, PlayerSettlements, PublishedOffer, TradeOffer,
};
use crate::civilization::concepts::trade::events::SendTradingCardsCommand;
use crate::civilization::concepts::trade::functions::receiver_can_pay_for_offer;
use crate::civilization::concepts::trade::resources::{TradeCountdown, TradeUiState};
use crate::civilization::concepts::trade_cards::components::{PlayerTradeCards, TradeCard};
use crate::civilization::concepts::trade_cards::enums::Commodity;
use crate::stupid_ai::prelude::IsHuman;
use crate::GameActivity;
use bevy::prelude::{
    debug, Commands, Entity, EventReader, Has, Name, NextState, Query, Res, ResMut, Time, With,
    Without,
};
use bevy::utils::HashMap;
use bevy_egui::{egui, EguiContexts};

pub fn setup_trade(
    mut commands: Commands,
    trading_players_query: Query<(&PlayerTradeCards, Entity, Has<IsHuman>)>,
    mut trade_ui_state: ResMut<TradeUiState>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    let mut has_any_human = false;

    for (trade_cards, player, is_human) in trading_players_query.iter() {
        if trade_cards.can_trade() {
            if is_human {
                has_any_human = true;
                trade_ui_state.human_player = Some(player);
            }
            commands.entity(player).insert(CanTrade);
        }
    }
    if !has_any_human {
        debug!("No human player can trade. Skipping trade phase.");
        next_state.set(GameActivity::PopulationExpansion)
    }
}

pub fn remove_rejected_trades(
    trade_offers: Query<(Entity, &TradeOffer), With<PublishedOffer>>,
    mut commands: Commands,
) {
    for (entity, offer) in trade_offers.iter() {
        if offer.trade_rejected() {
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
            let receiver = offer.receiver.expect("There should be a receiver!");

            if !player_settlement_query.contains(initiator) {
                commands
                    .entity(initiator)
                    .insert(PlayerSettlements::default());
            }
            if !player_settlement_query.contains(receiver) {
                commands
                    .entity(receiver)
                    .insert(PlayerSettlements::default());
            }
            commands.entity(trade_entity).insert(InSettlement); //Makes sure we don't end up here again!
            let mut initiator_settlements = player_settlement_query
                .get_mut(initiator)
                .expect("Player should have settlements");
            initiator_settlements.trades.push_back(trade_entity);
            let mut receiver_settlements = player_settlement_query
                .get_mut(receiver)
                .expect("Player should have settlements");
            receiver_settlements.trades.push_back(trade_entity);
        }
    }
}

pub fn trade_ui(
    mut egui_context: EguiContexts,
    mut trade_offers: Query<&mut TradeOffer, With<PublishedOffer>>,
    trading_players_query: Query<(&Name, Entity, &PlayerTradeCards, Has<IsHuman>), With<CanTrade>>,
    mut human_player: Query<
        (Entity, &Name, &IsHuman, Option<&mut TradeOffer>),
        Without<PublishedOffer>,
    >,
    mut ui_state: ResMut<TradeUiState>,
    mut next_state: ResMut<NextState<GameActivity>>,
    mut commands: Commands,
) {
    let ctx = egui_context.ctx_mut();
    egui::Window::new("Trade Interface").show(ctx, |ui| {
        // Section: Player List with trading capabilities
        ui.heading("Players Available for Trade");
        for (player_name, player_entity, _, is_human) in trading_players_query.iter() {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Player: {}, {}",
                    player_name.as_str(),
                    if is_human { "Human" } else { "AI" }
                ));
                if ui.button("Propose Trade").clicked() {
                    if let Ok((human_entity, human_name, _, offer)) = human_player.get_single() {
                        if offer.is_none() {
                            commands
                                .entity(human_entity)
                                .insert(TradeOffer::propose_trade_to(
                                    human_entity,
                                    human_name.clone(),
                                    player_entity,
                                    player_name.clone(),
                                ));
                        }
                    }
                }
            });
        }
        if let Ok((human_entity, human_name, _, offer)) = human_player.get_single() {
            if offer.is_none() && ui.button("Open Proposal").clicked() {
                commands
                    .entity(human_entity)
                    .insert(TradeOffer::new(human_entity, human_name.clone()));
            }
        }

        // Section: Trade Offers
        ui.separator();
        ui.heading("Current Trade Offers");

        for mut offer in trade_offers.iter_mut() {
            ui.group(|ui| {
                ui.label(format!("From: {}", offer.initiator_name.as_str()));
                ui.horizontal(|ui| {
                    ui.label("Offered Commodities:");
                    display_commodities(ui, &offer.initiator_pays);
                });
                if let Some(receiver_name) = &offer.receiver_name {
                    ui.label(format!("To: {}", receiver_name.as_str()));
                } else {
                    ui.label("To: Anyone");
                }
                ui.horizontal(|ui| {
                    ui.label("Requested Commodities:");
                    display_commodities(ui, &offer.initiator_gets);
                });

                if offer.receiver.is_some() && offer.receiver == ui_state.human_player {
                    let player = ui_state.human_player.unwrap();
                    if let Ok((_, _, trade_cards, _)) = trading_players_query.get(player) {
                        if receiver_can_pay_for_offer(&offer, trade_cards)
                            && ui.button("Accept").clicked()
                        {
                            offer.accept(player);
                        }
                        if ui.button("Reject").clicked() {
                            offer.reject(player);
                        }
                    }
                }

                if let Ok((h_entity, _, _, existing_offer)) = human_player.get_single() {
                    if offer.initiator != h_entity
                        && existing_offer.is_none()
                        && ui.button("Counter Offer").clicked()
                    {
                        let countered_offer = offer.prepare_counter_offer(h_entity);
                        commands.entity(h_entity).insert(countered_offer);
                    }
                }
            });
        }

        // 2. New Trade Offer Section
        if let Ok((human_player, _, _, potential_offer)) = human_player.get_single_mut() {
            if let Some(mut new_offer) = potential_offer {
                ui.group(|ui| {
                    ui.label(format!(
                        "New Offer from {}",
                        new_offer.initiator_name.as_str()
                    ));
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label("Offered Commodities:");
                            if ui.button("Offer commodity").clicked() {
                                ui_state.add_offered_commodity_open = true;
                            }
                            display_commodities(ui, &new_offer.initiator_pays);
                        });
                        ui.vertical(|ui| {
                            ui.label("Requested Commodities:");
                            if ui.button("Request commodity").clicked() {
                                ui_state.add_requested_commodity_open = true;
                            }
                            display_commodities(ui, &new_offer.initiator_gets);
                        });
                    });

                    if let Some(player) = ui_state.human_player {
                        if let Ok((_, _, trade_cards, _)) = trading_players_query.get(player) {
                            egui::Window::new("Add Offered Commodity")
                                .vscroll(true)
                                .open(&mut ui_state.add_offered_commodity_open)
                                .show(ctx, |ui| {
                                    ui.label("Add another commodity to the offer");
                                    ui.vertical(|ui| {
                                        for commodity in Commodity::iter() {
                                            ui.horizontal(|ui| {
                                                if new_offer.gets_number_of_cards()
                                                    < trade_cards.number_of_tradeable_cards()
                                                    && ui
                                                        .button(format!("Add {:?}", commodity))
                                                        .clicked()
                                                {
                                                    new_offer.initiator_pays_more(commodity);
                                                }
                                                if ui
                                                    .button(format!("Remove {:?}", commodity))
                                                    .clicked()
                                                {
                                                    new_offer.pay_less(commodity);
                                                }
                                            });
                                        }
                                    });
                                });

                            egui::Window::new("Add Requested Commodity")
                                .vscroll(true)
                                .open(&mut ui_state.add_requested_commodity_open)
                                .show(ctx, |ui| {
                                    ui.label("Request another commodity");
                                    for commodity in Commodity::iter() {
                                        if ui.button(format!("Request {:?}", commodity)).clicked() {
                                            new_offer.initiator_gets_more(commodity);
                                        }
                                        if ui.button(format!("Remove {:?}", commodity)).clicked() {
                                            new_offer.get_less(commodity);
                                        }
                                    }
                                });
                        }
                    }

                    if ui.button("Send Offer").clicked() {
                        commands.entity(human_player).remove::<TradeOffer>();
                        commands.spawn((new_offer.clone(), PublishedOffer));
                    }
                });
            } else {
                ui.label("No new offers. Select a player to start trading.");
            }
        }

        if ui.button("Stop Trading").clicked() {
            next_state.set(GameActivity::PopulationExpansion);
        }
    });
}

// Helper function to display commodities in a trade offer
fn display_commodities(ui: &mut egui::Ui, commodities: &HashMap<Commodity, usize>) {
    ui.vertical(|ui| {
        for (commodity, amount) in commodities {
            ui.label(format!("{:?}: {}", commodity, amount));
        }
    });
}

/// Trigger `NeedsTradeMove` components on AI players after a trade countdown finishes.
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
            commands.entity(entity).insert(NeedsTradeMove);
        }
    }
}

pub fn handle_send_trading_cards_command(
    mut command_reader: EventReader<SendTradingCardsCommand>,
    mut player_trading_cards: Query<&mut PlayerTradeCards>,
) {
    for event in command_reader.read() {
        let mut sender_trade_cards = player_trading_cards.get_mut(event.sending_player).unwrap();
        let mut cards_to_send: Vec<TradeCard> = Vec::new();
        for (card_type, count) in event.cards_to_send.iter() {
            if let Some(cards) = sender_trade_cards.remove_n_trade_cards(*count, *card_type) {
                cards_to_send.extend(cards);
            }
        }
        if !cards_to_send.is_empty() {
            let mut target_trade_cards = player_trading_cards.get_mut(event.receiving_player).unwrap();
            for card in cards_to_send {
                target_trade_cards.add_trade_card(card);
            }
        }
    }
}
