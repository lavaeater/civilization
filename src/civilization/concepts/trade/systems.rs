use crate::civilization::concepts::trade::resources::TradeResources;
use crate::civilization::concepts::trade_cards::enums::Commodity;
use bevy::prelude::{Entity, Name, Res};
use bevy::utils::HashMap;
use bevy_egui::{egui, EguiContexts};

#[derive(Clone, PartialEq)]
struct UserTradeMenu {
    player: Entity,
    player_name: Name,
}

pub fn setup_trade(
    mut trade_resources: Res<TradeResources>,
    player_names_query: Query<>,
) {
    trade_resources.players_that_can_trade.clear();
    for (player, player_name) in player_names.iter() {
        trade_resources.players_that_can_trade.push(UserTradeMenu {
            player,
            player_name: player_name.clone(),
        });
    }
}
) {
    
}

pub fn trade_ui(
    mut egui_context: EguiContexts,
    trade_resources: Res<TradeResources>,
) {
    egui::Window::new("Trade Interface").show(egui_context.ctx_mut(), |ui| {
        // Section: Player List with trading capabilities
        ui.heading("Players Available for Trade");
        for (player, player_name) in &trade_resources.players_that_can_trade {
            ui.horizontal(|ui| {
                ui.label(format!("Player: {}", player_name.as_str()));
                if ui.button("Trade").clicked() {
                    // Logic to initiate a trade with this player
                    // E.g., set them as the receiver in `TradeOffer`
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