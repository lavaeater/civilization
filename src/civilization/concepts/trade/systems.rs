use crate::civilization::concepts::trade::components::{CanTrade, PublishedOffer, TradeOffer};
use crate::civilization::concepts::trade::resources::{receiver_can_accept_trade_offer, TradeUiState};
use crate::civilization::concepts::trade_cards::components::PlayerTradeCards;
use crate::civilization::concepts::trade_cards::enums::Commodity;
use crate::stupid_ai::prelude::IsHuman;
use crate::GameActivity;
use bevy::prelude::{debug, Commands, Entity, Has, Name, NextState, Query, ResMut, With, Without};
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
            commands.entity(player).insert(CanTrade::default());
        }
    }
    if !has_any_human {
        debug!("No human player can trade. Skipping trade phase.");
        next_state.set(GameActivity::PopulationExpansion)
    }
}

pub fn trade_ui(
    mut egui_context: EguiContexts,
    mut trade_offers: Query<&mut TradeOffer, With<PublishedOffer>>,
    trading_players_query: Query<(&Name, Entity, &PlayerTradeCards, Has<IsHuman>), With<CanTrade>>,
    mut human_player: Query<(Entity, &Name, &IsHuman, Option<&mut TradeOffer>), Without<PublishedOffer>>,
    mut ui_state: ResMut<TradeUiState>,
    mut next_state: ResMut<NextState<GameActivity>>,
    mut commands: Commands,
) {
    let ctx = egui_context.ctx_mut();
    egui::Window::new("Trade Interface")
        .show(ctx, |ui| {
            // Section: Player List with trading capabilities
            ui.heading("Players Available for Trade");
            for (player_name, player_entity, _, is_human) in trading_players_query.iter() {
                ui.horizontal(|ui| {
                    ui.label(format!("Player: {}, {}", player_name.as_str(), if is_human { "Human" } else { "AI" }));
                    if ui.button("Propose Trade").clicked() {
                        if let Ok((human_entity, human_name, _, offer)) = human_player.get_single() {
                            if offer.is_none() {
                                commands.entity(human_entity).insert(TradeOffer::propose_trade_to(human_entity, human_name.clone(), player_entity, player_name.clone()));
                            }
                        }
                    }
                });
            }
            if let Ok((human_entity, human_name, _, offer)) = human_player.get_single() {
                if offer.is_none() && ui.button("Open Proposal").clicked() {
                    commands.entity(human_entity).insert(TradeOffer::new(human_entity, human_name.clone()));
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
                        display_commodities(ui, &offer.initiator_commodities);
                    });
                    if let Some(receiver_name) = &offer.receiver_name {
                        ui.label(format!("To: {}", receiver_name.as_str()));
                    } else {
                        ui.label("To: Anyone");
                    }
                    ui.horizontal(|ui| {
                        ui.label("Requested Commodities:");
                        display_commodities(ui, &offer.receiver_commodities);
                    });

                    if offer.receiver.is_some() && offer.receiver == ui_state.human_player {
                        let player = ui_state.human_player.unwrap();
                        if let Ok((_, _, trade_cards, _)) = trading_players_query.get(player) {
                            if receiver_can_accept_trade_offer(&offer, trade_cards) && ui.button("Accept").clicked() {
                                offer.accept(player);
                            }
                            if ui.button("Reject").clicked() {
                                offer.reject(player);
                            }
                        }
                    }

                    if let Ok((h_entity, _, _, existing_offer)) = human_player.get_single() {
                        if offer.initiator != h_entity && existing_offer.is_none() && ui.button("Counter Offer").clicked() {
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
                        ui.label(format!("New Offer from {}", new_offer.initiator_name.as_str()));
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label("Offered Commodities:");
                                if ui.button("Offer commodity").clicked() {
                                    ui_state.add_offered_commodity_open = true;
                                }
                                display_commodities(ui, &new_offer.initiator_commodities);
                            });
                            ui.vertical(|ui| {
                                ui.label("Requested Commodities:");
                                if ui.button("Request commodity").clicked() {
                                    ui_state.add_requested_commodity_open = true;
                                }
                                display_commodities(ui, &new_offer.receiver_commodities);
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
                                                    if new_offer.receiver_number_of_cards() < trade_cards.number_of_tradeable_cards() && ui.button(format!("Add {:?}", commodity)).clicked() {
                                                        new_offer.pay_more(commodity);
                                                    }
                                                    if ui.button(format!("Remove {:?}", commodity)).clicked() {
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
                                                new_offer.get_more(commodity);
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