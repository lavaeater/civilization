use crate::civilization::concepts::trade::components::CanTrade;
use crate::civilization::concepts::trade::resources::TradeResources;
use crate::civilization::concepts::trade_cards::components::PlayerTradeCards;
use crate::civilization::concepts::trade_cards::enums::Commodity;
use bevy::prelude::{Commands, Entity, Name, Query, ResMut, With};
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
    trading_players_query: Query<(&Name, Entity, &PlayerTradeCards), With<CanTrade>>
) {
    egui::Window::new("Trade Interface").show(egui_context.ctx_mut(), |ui| {
        // Section: Player List with trading capabilities
        ui.heading("Players Available for Trade");
        for (player_name, _, _) in trading_players_query.iter() {
            ui.horizontal(|ui| {
                ui.label(format!("Player: {}", player_name.as_str()));
                if ui.button("Propose Trade").clicked() {
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