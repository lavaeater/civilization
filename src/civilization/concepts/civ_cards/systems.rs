use crate::civilization::{
    AvailableCivCards, BackToCardSelection, CardHandle, CivCardDefinition, CivCardPurchasePhase,
    CivCardSelectionState, CivCardType, CivCardsAcquisition, CivTradeUi, ConfirmCivCardPurchase,
    Credits, PaymentAdjustButton, PaymentSelectionPanel, PaymentState, PaymentValueDisplay,
    PlayerAcquiringCivilizationCards, PlayerCivilizationCards,
    PlayerDoneAcquiringCivilizationCards, ProceedToPayment, RefreshCivCardsUi, SelectedCardsSummary,
    ToggleCivCardSelection,
};
use crate::civilization::concepts::acquire_trade_cards::{CivilizationTradeCards, PlayerTradeCards, TradeCardTrait};
use crate::player::Player;
use crate::stupid_ai::IsHuman;
use crate::GameActivity;
use bevy::asset::{AssetServer, Assets};
use bevy::color::Color;
use bevy::prelude::{percent, Add, Button, Changed, Commands, Entity, Has, Interaction, MessageReader, MessageWriter, NextState, On, Query, Res, ResMut, Val, With};
use bevy::ui_widgets::Button as WidgetsButton;
use bevy::ui_widgets::Activate;
use lava_ui_builder::{LavaTheme, UIBuilder};

pub fn load_civ_cards(mut commands: Commands, asset_server: Res<AssetServer>) {
    let card_handle = asset_server.load("definitions/civilization.cards.ron");
    let h = CardHandle(card_handle);
    commands.insert_resource(h);
}

pub fn init_civ_cards(
    mut commands: Commands,
    card_handle: Res<CardHandle>,
    maps: Res<Assets<AvailableCivCards>>,
) {
    if let Some(cards) = maps.get(card_handle.0.id()) {
        commands.insert_resource(AvailableCivCards {
            cards: cards.cards.clone(),
        });
    }
}

pub fn on_add_player_acquiring_civilization_cards(
    trigger: On<Add, PlayerAcquiringCivilizationCards>,
    human_player_query: Query<(&Player, &IsHuman, &PlayerCivilizationCards, &PlayerTradeCards)>,
    ui_exists_query: Query<(), With<CivTradeUi>>,
    mut selection_state: ResMut<CivCardSelectionState>,
    commands: Commands,
    theme: Res<LavaTheme>,
    cards: Res<AvailableCivCards>,
) {
    if ui_exists_query.is_empty() {
        if let Ok((_, _, player_cards, player_trade_cards)) = human_player_query.get(trigger.entity) {
            selection_state.clear();
            selection_state.player_entity = Some(trigger.entity);
            
            build_civ_cards_ui(commands, &theme, &cards, player_cards, player_trade_cards, &selection_state);
        }
    }
}

fn build_civ_cards_ui(
    commands: Commands,
    theme: &LavaTheme,
    cards: &AvailableCivCards,
    player_cards: &PlayerCivilizationCards,
    player_trade_cards: &PlayerTradeCards,
    selection_state: &CivCardSelectionState,
) {
    let mut theme_to_use = theme.clone();
    theme_to_use.text.label_size = 16.0;
    theme_to_use.text.header_size = 20.0;
    let mut builder = UIBuilder::new(commands, Some(theme_to_use));
    
    builder.component::<CivTradeUi>().add_panel(|panel| {
        let panel_color = Color::srgba(0.1, 0.1, 0.1, 0.95);
        panel
            .display_flex()
            .flex_row()
            .size_scaled(90., 90.)
            .bg_color(panel_color)
            .padding_all_px(6.0)
            .gap_px(4.0);

        // Left side: Card columns
        panel.with_child(|cards_area| {
            cards_area
                .display_flex()
                .flex_row()
                .width_scaled(60.)
                .height(percent(100.))
                .gap_px(2.0)
                .overflow_scroll_y();

            cards_area.foreach_child(CivCardType::all_types().iter(), |col_builder, card_type| {
                let col_color = Color::srgba(0.15, 0.15, 0.15, 0.8);
                col_builder
                    .display_flex()
                    .flex_column()
                    .width(percent(20.0))
                    .height(percent(100.))
                    .bg_color(col_color)
                    .padding_all_px(3.0)
                    .row_gap_px(3.0)
                    .overflow_scroll_y()
                    .border_radius_all_px(2.0);

                col_builder.with_child(|header| {
                    header
                        .display_flex()
                        .justify_center()
                        .padding_all_px(4.0)
                        .margin_btm(Val::Px(4.0))
                        .default_text(card_type.to_string());
                });

                col_builder.foreach_child(&cards.get_cards(card_type), |card_builder, card| {
                    create_civ_card_panel(card_builder, card, player_cards, &cards, selection_state);
                });
            });
        });

        // Right side: Summary and actions
        panel.with_child(|sidebar| {
            sidebar
                .display_flex()
                .flex_column()
                .width_scaled(40.)
                .height(percent(100.))
                .bg_color(Color::srgba(0.12, 0.12, 0.15, 0.95))
                .padding_all_px(6.0)
                .row_gap_px(6.0)
                .border_radius_all_px(6.0);

            // Player's buying power
            let total_value = player_trade_cards.total_stack_value();
            sidebar.with_child(|info| {
                info.display_flex()
                    .flex_column()
                    .row_gap_px(4.0);
                info.default_text("Your Buying Power");
                info.default_text(format!("Commodity Value: {}", total_value));
            });

            // Selected cards summary
            sidebar.with_child(|summary| {
                summary
                    .component::<SelectedCardsSummary>()
                    .display_flex()
                    .flex_column()
                    .row_gap_px(4.0)
                    .padding_all_px(8.0)
                    .bg_color(Color::srgba(0.2, 0.2, 0.25, 0.9))
                    .border_radius_all_px(4.0);
                
                summary.default_text("Selected Cards:");
                
                if selection_state.selected_cards.is_empty() {
                    summary.default_text("(none)");
                } else {
                    let selected_defs = cards.cards_for_names(&selection_state.selected_cards);
                    let total_cost: u32 = selected_defs.iter().map(|c| {
                        let credits = cards.total_credits(&player_cards.cards);
                        c.calculate_cost(&credits)
                    }).sum();
                    
                    for card_def in &selected_defs {
                        let credits = cards.total_credits(&player_cards.cards);
                        let cost = card_def.calculate_cost(&credits);
                        summary.default_text(format!("• {} ({})", card_def.name, cost));
                    }
                    summary.default_text(format!("Total: {}", total_cost));
                    
                    let can_afford = total_cost as usize <= total_value;
                    if can_afford {
                        summary.with_child(|afford| {
                            afford.default_text("✓ Can afford");
                        });
                    } else {
                        summary.with_child(|cant| {
                            cant.default_text("✗ Cannot afford");
                        });
                    }
                }
            });

            // Action buttons
            sidebar.with_child(|buttons| {
                buttons
                    .display_flex()
                    .flex_column()
                    .row_gap_px(8.0)
                    .margin_top(Val::Auto)
                    .margin_btm(percent(5.));

                // Proceed to payment button
                if !selection_state.selected_cards.is_empty() {
                    buttons.add_button_observe(
                        "Proceed to Payment",
                        |_btn| {},
                        |_: On<Activate>, mut proceed_writer: MessageWriter<ProceedToPayment>| {
                            proceed_writer.write(ProceedToPayment);
                        },
                    );
                }

                // Done button (skip purchasing)
                let player_entity = selection_state.player_entity;
                buttons.add_button_observe(
                    "Done (Skip)",
                    |_btn| {},
                    move |_: On<Activate>, mut done_writer: MessageWriter<PlayerDoneAcquiringCivilizationCards>| {
                        if let Some(player) = player_entity {
                            done_writer.write(PlayerDoneAcquiringCivilizationCards(player));
                        }
                    },
                );
            });
        });
    });
}

fn create_civ_card_panel(
    card_builder: &mut UIBuilder, 
    card: &CivCardDefinition, 
    player_cards: &PlayerCivilizationCards, 
    cards: &AvailableCivCards,
    selection_state: &CivCardSelectionState,
) {
    let credits = cards.total_credits(&player_cards.cards);
    let actual_cost = card.calculate_cost(&credits);
    let owns_card = player_cards.owns(&card.name);
    let has_prerequisites = player_cards.has_prerequisites(&card.prerequisites);
    let is_selected = selection_state.is_selected(&card.name);
    
    // Determine card state and colors
    let (card_bg, status_text, is_purchasable): (Color, Option<String>, bool) = if owns_card {
        (Color::srgba(0.1, 0.3, 0.1, 1.0), Some("OWNED".to_string()), false)
    } else if !has_prerequisites {
        let missing: Vec<_> = card.prerequisites.iter()
            .filter(|p| !player_cards.owns(p))
            .map(|p| p.to_string())
            .collect();
        (Color::srgba(0.3, 0.15, 0.15, 1.0), Some(format!("Requires: {}", missing.join(", "))), false)
    } else if is_selected {
        (Color::srgba(0.2, 0.4, 0.2, 1.0), Some("SELECTED".to_string()), true)
    } else {
        (Color::srgba(0.2, 0.2, 0.25, 1.0), None, true)
    };

    card_builder
        .display_flex()
        .flex_column()
        .bg_color(card_bg)
        .padding_all_px(3.0)
        .border_radius_all_px(2.0)
        .row_gap_px(2.0);
    
    let card_name = card.name;
    if is_purchasable {
        card_builder.insert(Button);
        card_builder.insert(WidgetsButton);
        card_builder.observe(move |_: On<Activate>, mut toggle_writer: MessageWriter<ToggleCivCardSelection>| {
            toggle_writer.write(ToggleCivCardSelection(card_name));
        });
    }

    card_builder.with_child(|name_row| {
        name_row
            .display_flex()
            .flex_row()
            .justify_space_between();
        name_row.add_text_child(card.name.to_string(), None, None, None);
        if let Some(ref status) = status_text {
            name_row.add_text_child(format!("[{}]", status), None, None, None);
        }
    });
    card_builder.with_child(|cost_row| {
        cost_row
            .display_flex()
            .flex_row()
            .justify_space_between();
        if actual_cost < card.cost {
            cost_row.default_text(format!("Cost: {} (was {})", actual_cost, card.cost));
        } else {
            cost_row.default_text(format!("Cost: {}", actual_cost));
        }
    });
    if !card.credits.is_empty() {
        card_builder.with_child(|credits_row| {
            credits_row.default_text(format!("Gives {} credits", card.credits.len()));
        });
    }
}

#[allow(dead_code)]
fn format_credit(credit: &Credits) -> String {
    match credit {
        Credits::ToType(card_type, amount) => format!("+{} to {:?}", amount, card_type),
        Credits::ToAll(amount) => format!("+{} to all", amount),
        Credits::ToSpecificCard(card_name, amount) => format!("+{} to {}", amount, card_name),
    }
}

pub fn handle_toggle_card_selection(
    mut toggle_reader: MessageReader<ToggleCivCardSelection>,
    mut selection_state: ResMut<CivCardSelectionState>,
    mut refresh_writer: MessageWriter<RefreshCivCardsUi>,
) {
    for toggle in toggle_reader.read() {
        selection_state.toggle_card(toggle.0);
        refresh_writer.write(RefreshCivCardsUi);
    }
}

pub fn handle_proceed_to_payment_message(
    mut proceed_reader: MessageReader<ProceedToPayment>,
    mut selection_state: ResMut<CivCardSelectionState>,
    mut refresh_writer: MessageWriter<RefreshCivCardsUi>,
) {
    for _ in proceed_reader.read() {
        selection_state.phase = CivCardPurchasePhase::SelectingPayment;
        refresh_writer.write(RefreshCivCardsUi);
    }
}

pub fn handle_back_to_selection(
    mut back_reader: MessageReader<BackToCardSelection>,
    mut selection_state: ResMut<CivCardSelectionState>,
    mut payment_state: ResMut<PaymentState>,
    mut refresh_writer: MessageWriter<RefreshCivCardsUi>,
) {
    for _ in back_reader.read() {
        selection_state.phase = CivCardPurchasePhase::SelectingCards;
        payment_state.reset();
        refresh_writer.write(RefreshCivCardsUi);
    }
}

pub fn refresh_civ_cards_ui(
    mut refresh_reader: MessageReader<RefreshCivCardsUi>,
    mut commands: Commands,
    ui_query: Query<Entity, With<CivTradeUi>>,
    human_player_query: Query<(&PlayerCivilizationCards, &PlayerTradeCards), With<IsHuman>>,
    theme: Res<LavaTheme>,
    cards: Res<AvailableCivCards>,
    selection_state: Res<CivCardSelectionState>,
    payment_state: Res<PaymentState>,
) {
    for _ in refresh_reader.read() {
        // Despawn existing UI
        for entity in ui_query.iter() {
            commands.entity(entity).despawn();
        }
        
        // Rebuild UI based on current phase
        if let Ok((player_cards, player_trade_cards)) = human_player_query.single() {
            match selection_state.phase {
                CivCardPurchasePhase::SelectingCards => {
                    build_civ_cards_ui(
                        commands.reborrow(), 
                        &theme, 
                        &cards, 
                        player_cards, 
                        player_trade_cards, 
                        &selection_state
                    );
                }
                CivCardPurchasePhase::SelectingPayment => {
                    build_payment_ui(
                        commands.reborrow(),
                        &theme,
                        &cards,
                        player_cards,
                        player_trade_cards,
                        &selection_state,
                        &payment_state,
                    );
                }
            }
        }
    }
}

fn build_payment_ui(
    commands: Commands,
    theme: &LavaTheme,
    cards: &AvailableCivCards,
    player_cards: &PlayerCivilizationCards,
    player_trade_cards: &PlayerTradeCards,
    selection_state: &CivCardSelectionState,
    payment_state: &PaymentState,
) {
    let mut theme_to_use = theme.clone();
    theme_to_use.text.label_size = 14.0;
    let mut builder = UIBuilder::new(commands, Some(theme_to_use));

    let selected_defs = cards.cards_for_names(&selection_state.selected_cards);
    let credits = cards.total_credits(&player_cards.cards);
    let total_cost: u32 = selected_defs.iter().map(|c| c.calculate_cost(&credits)).sum();
    let chosen_value = payment_state.total_value();
    let can_confirm = chosen_value >= total_cost as usize;

    builder.component::<CivTradeUi>().add_panel(|panel| {
        let panel_color = Color::srgba(0.1, 0.1, 0.1, 0.95);
        panel
            .display_flex()
            .flex_column()
            .size(percent(80.), percent(80.))
            .bg_color(panel_color)
            .padding_all_px(20.0)
            .row_gap_px(16.0)
            .align_items_center();

        panel.default_text("Select Payment");

        // Show what we're buying
        panel.with_child(|buying| {
            buying
                .display_flex()
                .flex_column()
                .row_gap_px(4.0)
                .padding_all_px(12.0)
                .bg_color(Color::srgba(0.15, 0.15, 0.2, 0.9))
                .border_radius_all_px(4.0);

            buying.default_text("Purchasing:");
            for card_def in &selected_defs {
                let cost = card_def.calculate_cost(&credits);
                buying.default_text(format!("• {} ({})", card_def.name, cost));
            }
            buying.default_text(format!("Total Cost: {}", total_cost));
        });

        // Commodity card picker — one +/- row per stack
        panel.with_child(|commodities| {
            commodities
                .component::<PaymentSelectionPanel>()
                .display_flex()
                .flex_column()
                .gap_px(6.0)
                .padding_all_px(12.0)
                .bg_color(Color::srgba(0.15, 0.15, 0.2, 0.9))
                .border_radius_all_px(4.0)
                .width(percent(100.));

            commodities.default_text("Choose cards to pay with:");

            let stacks = player_trade_cards.as_card_stacks_sorted_by_value();
            for stack in stacks.iter().filter(|s| s.is_commodity) {
                let card_type = stack.card_type;
                let owned = stack.count;
                let chosen = payment_state.chosen.get(&card_type).copied().unwrap_or(0);
                let chosen_value_for_stack = chosen * chosen * card_type.value();

                commodities.add_row(|row| {
                    row.width(percent(100.0)).align_items_center().margin(bevy::prelude::UiRect::vertical(Val::Px(2.0)));

                    // Decrement button
                    row.add_button(
                        "<", 30.0, 24.0,
                        Color::srgb(0.4, 0.3, 0.3), 14.0, 4.0,
                        PaymentAdjustButton { card: card_type, delta: -1 },
                    );

                    // Card name label
                    row.with_child(|label| {
                        label.set_node(bevy::prelude::Node {
                            padding: bevy::prelude::UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                            margin: bevy::prelude::UiRect::horizontal(Val::Px(4.0)),
                            min_width: Val::Px(120.0),
                            justify_content: bevy::prelude::JustifyContent::Center,
                            ..Default::default()
                        });
                        label.add_text_child(
                            format!("{}", card_type),
                            None, Some(12.0), Some(Color::WHITE),
                        );
                    });

                    // Chosen / owned count
                    row.add_text_child(
                        format!("{}/{}", chosen, owned),
                        None, Some(13.0), Some(Color::WHITE),
                    );

                    // Increment button
                    row.add_button(
                        ">", 30.0, 24.0,
                        Color::srgb(0.3, 0.4, 0.3), 14.0, 4.0,
                        PaymentAdjustButton { card: card_type, delta: 1 },
                    );

                    // Stack value contribution
                    row.add_text_child(
                        format!("  = {}", chosen_value_for_stack),
                        None, Some(12.0), Some(Color::srgb(0.7, 0.9, 0.7)),
                    );
                });
            }
        });

        // Running total display
        panel.with_child(|total_row| {
            total_row
                .component::<PaymentValueDisplay>()
                .display_flex()
                .flex_row()
                .gap_px(8.0)
                .padding_all_px(8.0)
                .bg_color(Color::srgba(0.12, 0.12, 0.18, 0.95))
                .border_radius_all_px(4.0);

            let status_color = if can_confirm {
                Color::srgb(0.3, 0.9, 0.3)
            } else {
                Color::srgb(0.9, 0.4, 0.3)
            };
            total_row.add_text_child(
                format!("Paying: {} / {} required", chosen_value, total_cost),
                None, Some(14.0), Some(status_color),
            );
        });

        // Buttons
        panel.with_child(|buttons| {
            buttons
                .display_flex()
                .flex_row()
                .gap_px(16.0);

            buttons.add_button_observe(
                "Back",
                |_btn| {},
                |_: On<Activate>, mut back_writer: MessageWriter<crate::civilization::BackToCardSelection>| {
                    back_writer.write(crate::civilization::BackToCardSelection);
                },
            );

            if can_confirm {
                let selected: Vec<_> = selection_state.selected_cards.iter().cloned().collect();
                let payment = payment_state.chosen.clone();
                buttons.add_button_observe(
                    "Confirm Purchase",
                    |_btn| {},
                    move |_: On<Activate>,
                          mut purchase_writer: MessageWriter<ConfirmCivCardPurchase>,
                          human_player_query: Query<Entity, With<IsHuman>>| {
                        if let Ok(player_entity) = human_player_query.single() {
                            purchase_writer.write(ConfirmCivCardPurchase {
                                player: player_entity,
                                cards_to_buy: selected.iter().cloned().collect(),
                                payment: payment.clone(),
                            });
                        }
                    },
                );
            }
        });
    });
}

pub fn handle_payment_adjust(
    mut interaction_query: Query<
        (&Interaction, &PaymentAdjustButton),
        Changed<Interaction>,
    >,
    mut payment_state: ResMut<PaymentState>,
    human_player_query: Query<&PlayerTradeCards, With<IsHuman>>,
    mut refresh_writer: MessageWriter<RefreshCivCardsUi>,
) {
    let Ok(player_trade_cards) = human_player_query.single() else { return; };

    let mut changed = false;
    for (interaction, btn) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let owned = player_trade_cards.number_of_cards_for_trade_card(btn.card);
        let current = payment_state.chosen.get(&btn.card).copied().unwrap_or(0);

        if btn.delta > 0 {
            if current < owned {
                *payment_state.chosen.entry(btn.card).or_insert(0) += 1;
                changed = true;
            }
        } else if current > 0 {
            if current > 1 {
                *payment_state.chosen.get_mut(&btn.card).unwrap() -= 1;
            } else {
                payment_state.chosen.remove(&btn.card);
            }
            changed = true;
        }
    }

    if changed {
        refresh_writer.write(RefreshCivCardsUi);
    }
}


pub fn process_civ_card_purchase(
    mut purchase_reader: MessageReader<ConfirmCivCardPurchase>,
    mut player_query: Query<(&mut PlayerCivilizationCards, &mut PlayerTradeCards)>,
    mut trade_cards_resource: ResMut<CivilizationTradeCards>,
    mut selection_state: ResMut<CivCardSelectionState>,
    mut done_writer: MessageWriter<PlayerDoneAcquiringCivilizationCards>,
    mut commands: Commands,
    ui_query: Query<Entity, With<CivTradeUi>>,
) {
    for purchase in purchase_reader.read() {
        if let Ok((mut player_cards, mut player_trade_cards)) = player_query.get_mut(purchase.player) {
            // Add civilization cards to player
            for card_name in &purchase.cards_to_buy {
                player_cards.add_card(*card_name);
            }
            
            // Remove trade cards used for payment and return to piles
            for (trade_card, count) in &purchase.payment {
                if player_trade_cards.remove_n_trade_cards(*count, *trade_card).is_some() {
                    // Return cards to the appropriate pile
                    let pile = trade_card.value();
                    if let Some(pile_vec) = trade_cards_resource.card_piles.get_mut(&pile) {
                        for _ in 0..*count {
                            pile_vec.push(*trade_card);
                        }
                    }
                }
            }
            
            // Clear selection state
            selection_state.clear();
            
            // Despawn UI
            for entity in ui_query.iter() {
                commands.entity(entity).despawn();
            }
            
            // Mark player as done
            done_writer.write(PlayerDoneAcquiringCivilizationCards(purchase.player));
        }
    }
}

pub fn player_is_done(
    mut commands: Commands,
    mut done_reader: MessageReader<PlayerDoneAcquiringCivilizationCards>,
    mut civ_cards_acquisition: ResMut<CivCardsAcquisition>,
    mut next_state: ResMut<NextState<GameActivity>>,
    ui_query: Query<Entity, With<CivTradeUi>>,
) {
    let mut human_done = false;
    for done in done_reader.read() {
        commands
            .entity(done.0)
            .remove::<PlayerAcquiringCivilizationCards>();
        if civ_cards_acquisition.human_players.remove(&done.0) {
            human_done = true;
        }
        civ_cards_acquisition.players.remove(&done.0);
    }
    if human_done {
        for entity in ui_query.iter() {
            commands.entity(entity).despawn();
        }
    }
    if civ_cards_acquisition.is_empty() {
        next_state.set(GameActivity::PopulationExpansion);
    }
}

pub fn begin_acquire_civ_cards(
    mut commands: Commands,
    mut civ_cards_acquisition: ResMut<CivCardsAcquisition>,
    players: Query<(Entity, Has<IsHuman>), With<Player>>,
    mut selection_state: ResMut<CivCardSelectionState>,
) {
    selection_state.clear();
    
    for (entity, is_human) in players.iter() {
        commands
            .entity(entity)
            .insert(PlayerAcquiringCivilizationCards);
        civ_cards_acquisition.players.insert(entity);
        if is_human {
            civ_cards_acquisition.human_players.insert(entity);
        }
    }
}

pub fn shuffle_trade_card_piles_on_exit(
    mut trade_cards_resource: ResMut<CivilizationTradeCards>,
) {
    use rand::seq::SliceRandom;
    let mut rng = rand::rng();
    
    for pile in trade_cards_resource.card_piles.values_mut() {
        pile.shuffle(&mut rng);
    }
}
