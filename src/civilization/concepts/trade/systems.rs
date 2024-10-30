use crate::civilization::concepts::trade::components::CanTrade;
use crate::civilization::concepts::trade::resources::TradeResources;
use crate::civilization::concepts::trade_cards::components::PlayerTradeCards;
use crate::civilization::concepts::trade_cards::enums::Commodity;
use crate::stupid_ai::prelude::IsHuman;
use bevy::prelude::{Commands, Entity, Has, Name, Query, ResMut, With};
use bevy::utils::HashMap;
use bevy_egui::{egui, EguiContexts};

pub fn setup_trade(
    mut commands: Commands,
    trading_players_query: Query<(&PlayerTradeCards, Entity)>,
) {
    for (trade_cards, player) in trading_players_query.iter() {
        if trade_cards.can_trade() {
            commands.entity(player).insert(CanTrade);
        }
    }
}

pub fn trade_ui(
    mut egui_context: EguiContexts,
    mut trade_resources: ResMut<TradeResources>,
    trading_players_query: Query<(&Name, Entity, &PlayerTradeCards, Has<IsHuman>), With<CanTrade>>,
    human_player: Query<(Entity, &Name, &IsHuman)>
) {
    egui::Window::new("Trade Interface").show(egui_context.ctx_mut(), |ui| {
        // Section: Player List with trading capabilities
        ui.heading("Players Available for Trade");
        for (player_name, player_entity, _, is_human) in trading_players_query.iter() {
            ui.horizontal(|ui| {
                ui.label(format!("Player: {}, {}", player_name.as_str(), if is_human { "Human" } else { "AI" }));
                if ui.button("Propose Trade").clicked() {
                    if let Ok((human_entity, human_name, _)) = human_player.get_single() {
                        trade_resources.create_new_offer(human_entity, human_name.clone(), player_entity, player_name.clone());
                    }
                }
            });
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
                    display_commodities(ui, &new_offer.initiator_commodities);
                });
                ui.horizontal(|ui| {
                    ui.label("Requested Commodities:");
                    display_commodities(ui, &new_offer.receiver_commodities);
                });

                ui.horizontal(|ui| {
                    for commodity in Commodity::iter() {
                        if ui.button(format!("Add {:?}", commodity)).clicked() {
                            // Logic to add this commodity to the offer
                        }
                        if ui.button(format!("Remove {:?}", commodity)).clicked() {
                            // Logic to remove this commodity from the offer
                        }
                    }
                });
            });
        } else {
            ui.label("No new offers. Select a player to start trading.");
        }
    });
}

// Helper function to display commodities in a trade offer
fn display_commodities(ui: &mut egui::Ui, commodities: &HashMap<Commodity, usize>) {
    for (commodity, amount) in commodities {
        ui.label(format!("{:?}: {}", commodity, amount));
    }
}