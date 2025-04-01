use crate::civilization::components::prelude::TradeMoveType;
use crate::civilization::components::prelude::TradeMoveType::OpenTradeOffer;
use crate::civilization::concepts::trade::components::{CanTrade, InSettlement, NeedsTradeMove, PlayerSettlements, PublishedOffer, PublishedOffersList, TradeButtonAction, TradeOffer};
use crate::civilization::concepts::trade::events::SendTradingCardsCommand;
use crate::civilization::concepts::trade::resources::{TradeCountdown, TradeUiState};
use crate::civilization::concepts::trade_cards::components::{PlayerTradeCards, TradeCard};
use crate::civilization::ui::ui_builder::{ButtonAction, ButtonDef, UIBuilder};
use crate::stupid_ai::prelude::IsHuman;
use crate::GameActivity;
use bevy::color::palettes::basic::GREEN;
use bevy::math::vec2;
use bevy::prelude::{debug, default, AssetServer, Button, Changed, Color, Commands, Entity, EventReader, Has, Interaction, JustifyContent, NextState, PositionType, Query, Res, ResMut, Time, UiRect, Val, With, Without};
use bevy::ui::BackgroundColor;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

pub fn button_action(
    mut interaction_query: Query<
        (&Interaction, &TradeButtonAction, &mut BackgroundColor),
        (Changed<Interaction>),
    >,
) {
    for (interaction, menu_button_action, mut bg_color) in &mut interaction_query {
        match interaction {
            Interaction::Pressed => {
                *bg_color = PRESSED_BUTTON.into();
                match menu_button_action {
                    TradeButtonAction::Ok => {}
                    TradeButtonAction::Cancel => {}
                    TradeButtonAction::TradeAction(trade_move_type) => match trade_move_type {
                        TradeMoveType::OpenTradeOffer => {
                            debug!("Open trade offer");
                        }
                        TradeMoveType::AcceptTradeOffer => {}
                        TradeMoveType::DeclineTradeOffer => {}
                        TradeMoveType::CounterTradeOffer(_) => {}
                        TradeMoveType::StopTrading => {}
                        TradeMoveType::SettleTrade => {}
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
    asset_server: ResMut<AssetServer>,
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
    } else {

        let card_color = Color::srgba(0.7, 0.6, 0.2, 0.8);
        let bg_color = Color::srgba(0.5, 0.5, 0.5, 0.25);
        let border_color = Color::srgba(0.2, 0.2, 0.2, 0.8);
        let font = asset_server.load("fonts/FiraSans-Bold.ttf");

        let mut ui_builder =  UIBuilder::new(commands);
        ui_builder
            .as_flex_col_with_props(Val::Percent(60.), Val::Percent(100.), bg_color)
            .at(Val::Percent(60.), Val::Percent(0.0), PositionType::Absolute)
            .child()
            .as_flex_row()
            .with_justify_content(JustifyContent::SpaceBetween)
            .with_children(|mut builder| {
                builder
                    .child()
                    .with_button(ButtonDef {
                        text: "Open Trade Offer".to_string(),
                        font: font.clone_weak(),
                        ..default()
                    }, TradeButtonAction::TradeAction(OpenTradeOffer))
                    .parent();
            })
            .parent()
            .child()
            .as_flex_col_with_props(Val::Percent(100.), Val::Percent(100.), bg_color)
            .with_component::<PublishedOffersList>();

        let (ui_entity, commands) = ui_builder.build();
        
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
