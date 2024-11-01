use crate::civilization::concepts::trade::components::CanTrade;
use crate::civilization::concepts::trade::resources::{TradeResources, TradeUiState};
use crate::civilization::concepts::trade_cards::components::PlayerTradeCards;
use crate::civilization::concepts::trade_cards::enums::Commodity;
use crate::stupid_ai::prelude::IsHuman;
use crate::GameActivity;
use bevy::prelude::{debug, Commands, Entity, Has, Name, NextState, Query, ResMut, With};
use bevy::utils::HashMap;
use bevy_egui::{egui, EguiContexts};

pub fn setup_trade(
    mut commands: Commands,
    trading_players_query: Query<(&PlayerTradeCards, Entity, Has<IsHuman>)>,
    mut trade_ui_state: ResMut<TradeUiState>,
    mut next_state: ResMut<NextState<GameActivity>>
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

pub fn trade_ui(
    mut egui_context: EguiContexts,
    mut trade_resources: ResMut<TradeResources>,
    trading_players_query: Query<(&Name, Entity, &PlayerTradeCards, Has<IsHuman>), With<CanTrade>>,
    human_player: Query<(Entity, &Name, &IsHuman)>,
    mut ui_state: ResMut<TradeUiState>,
    mut commands: Commands,
) {
    let mut ctx = egui_context.ctx_mut();
    egui::Window::new("Trade Interface")
        .show(ctx, |ui| {
            // Section: Player List with trading capabilities
            ui.heading("Players Available for Trade");
            for (player_name, player_entity, _, is_human) in trading_players_query.iter() {
                ui.horizontal(|ui| {
                    ui.label(format!("Player: {}, {}", player_name.as_str(), if is_human { "Human" } else { "AI" }));
                    if ui.button("Propose Trade").clicked() {
                        if let Ok((human_entity, human_name, _)) = human_player.get_single() {
                            trade_resources.create_new_offer(human_entity, human_name.clone(), Some(player_entity), Some(player_name.clone()));
                        }
                    }
                });
            }
            if ui.button("Open Proposal").clicked() {
                if let Ok((human_entity, human_name, _)) = human_player.get_single() {
                    trade_resources.create_new_offer(human_entity, human_name.clone(), None, None);
                }
            }

            // Section: Trade Offers
            ui.separator();
            ui.heading("Current Trade Offers");
            for offer in &trade_resources.offers {
                ui.group(|ui| {
                    ui.label(format!(
                        "Offer from {} to {}",
                        offer.initiator_name.as_str(),
                        offer.receiver_name.as_ref().map(|name| name.as_str()).unwrap_or("Pending")
                    ));
                    ui.horizontal(|ui| {
                        ui.label("Initiator Commodities:");
                        display_commodities(ui, &offer.initiator_commodities);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Receiver Commodities:");
                        display_commodities(ui, &offer.receiver_commodities);
                    });

                    // Offer actions
                    ui.horizontal(|ui| {
                        if ui.button("Accept").clicked() {
                            // Logic to accept the trade
                        }
                        if ui.button("Reject").clicked() {
                            // Logic to reject the trade
                        }
                    });
                });
            }

            // Section: New Trade Offer
            ui.separator();
            ui.heading("Create a New Trade Offer");
            if let Some(new_offer) = &trade_resources.new_offer {
                ui.group(|ui| {
                    ui.label(format!("New Offer from {}", new_offer.initiator_name.as_str()));
                    ui.horizontal(|ui| {
                        ui.label("Offered Commodities:");
                        if ui.button("Offer commodity").clicked() {
                            ui_state.add_offered_commodity_open = true;
                        }
                        display_commodities(ui, &new_offer.initiator_commodities);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Requested Commodities:");
                        if ui.button("Request commodity").clicked() {
                            ui_state.add_requested_commodity_open = true;
                        }
                        display_commodities(ui, &new_offer.receiver_commodities);
                    });
                });
            } else {
                ui.label("No new offers. Select a player to start trading.");
            }
        });
    if let Some(player) = ui_state.human_player {
        if let Ok((_, _, trade_cards, _)) = trading_players_query.get(player) {
            egui::Window::new("Add Offered Commodity")
                .vscroll(true)
                .open(&mut ui_state.add_offered_commodity_open)
                .show(ctx, |ui| {
                    ui.label("Add another commodity to the offer");
                    for commodity in Commodity::iter() {
                        if ui.button(format!("Offer {:?}", commodity)).clicked() {
                            trade_resources.new_offer.as_mut().unwrap().pay_more(commodity);
                        }
                        if ui.button(format!("Remove {:?}", commodity)).clicked() {
                            trade_resources.new_offer.as_mut().unwrap().pay_less(commodity);
                        }
                    }
                });

            egui::Window::new("Add Requested Commodity")
                .vscroll(true)
                .open(&mut ui_state.add_requested_commodity_open)
                .show(ctx, |ui| {
                    ui.label("Request another commodity");
                    for commodity in Commodity::iter() {
                        if ui.button(format!("Request {:?}", commodity)).clicked() {
                            trade_resources.new_offer.as_mut().unwrap().get_more(commodity);
                        }
                        if ui.button(format!("Remove {:?}", commodity)).clicked() {
                            trade_resources.new_offer.as_mut().unwrap().get_less(commodity);
                        }
                    }
                });
        }
    }
}

// Helper function to display commodities in a trade offer
fn display_commodities(ui: &mut egui::Ui, commodities: &HashMap<Commodity, usize>) {
    for (commodity, amount) in commodities {
        ui.label(format!("{:?}: {}", commodity, amount));
    }
}