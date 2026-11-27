//! The human's trade controls (docs/trade-dialog-design.md, step 2): the
//! buttons that turn a line or a proposal into a pre-filled reply, proposal or
//! counter, and the "say something" dialog. The inbox that shows them is in
//! `trade_inbox.rs`.

use crate::civilization::TradeCardTrait;
use crate::civilization::concepts::acquire_trade_cards::{PlayerTradeCards, TradeCard};
use crate::civilization::concepts::trade::trade_components::OpenTradeOffer;
use crate::civilization::concepts::trade::trade_resources::{
    CreateOfferState, TradePhaseState, TradeUiState,
};
use crate::civilization::concepts::trade::trade_systems::{
    ACCEPT_OFFER_PALETTE, CREATE_OFFER_PALETTE, DONE_TRADING_PALETTE, commodity_color, ink_on,
};
use crate::civilization::concepts::trade::trade_talk::{
    ProposalDraft, TalkDraft, TalkKind, TradeTalk,
};
use crate::civilization::ui::{ButtonActivated, game_button};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use lava_ui_builder::InteractionPalette;

/// "💬 Say…" on the trade panel.
#[derive(Component, Default)]
pub struct SayButton;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineAction {
    Reply,
    Propose,
}

/// A button on a line of talk.
#[derive(Component)]
pub struct TalkLineButton {
    pub talk: Entity,
    pub action: LineAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalAction {
    /// Turn a proposal made to us around and send it back.
    Counter,
    /// Make the same deal to the same player, from us.
    Outbid,
    /// Take back our own proposal.
    Withdraw,
}

/// A button on a proposal, besides Accept and Decline.
#[derive(Component)]
pub struct ProposalActionButton {
    pub offer: Entity,
    pub action: ProposalAction,
}

#[derive(Component, Default)]
pub struct TalkComposerModal;

#[derive(Component, Default)]
pub struct CloseTalkComposerButton;

#[derive(Component, Default)]
pub struct SayItButton;

/// Clears who the talk is said to: it goes to the whole table.
#[derive(Component, Default)]
pub struct SayToTableButton;

#[derive(Component, Default)]
pub struct TalkComposerTitle;

#[derive(Component, Default)]
pub struct TalkPreviewText;

/// A card chip in the composer: on or off in "I want" or "I have".
#[derive(Component)]
pub struct TalkCardToggle {
    pub card: TradeCard,
    pub have: bool,
}

// ── Helpers ─────────────────────────────────────────────────────────────────

pub(crate) fn side_text(side: &HashMap<TradeCard, usize>) -> String {
    let mut cards: Vec<(TradeCard, usize)> = side.iter().map(|(c, n)| (*c, *n)).collect();
    cards.sort_by_key(|(c, _)| std::cmp::Reverse(c.value()));
    let parts: Vec<String> = cards
        .iter()
        .map(|(c, n)| {
            if *n > 1 {
                format!("{n}x {c}")
            } else {
                c.to_string()
            }
        })
        .collect();
    if parts.is_empty() {
        "nothing named".to_string()
    } else {
        parts.join(", ")
    }
}

pub(crate) fn small_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    palette: InteractionPalette,
    marker: impl Bundle,
) {
    parent
        .spawn((
            game_button(),
            marker,
            Node {
                padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                margin: UiRect::left(Val::Px(4.0)),
                ..Default::default()
            },
            BackgroundColor(palette.none),
            palette,
        ))
        .with_child((
            Text::new(label),
            TextFont {
                font_size: FontSize::Px(11.0),
                ..Default::default()
            },
            TextColor(Color::WHITE),
        ));
}

// ── Turning lines into replies and proposals ───────────────────────────────

/// "Reply" opens the composer addressed to the speaker; "Propose" opens the
/// offer dialog filled in from what was said.
pub fn handle_talk_line_buttons(
    mut activated: MessageReader<ButtonActivated>,
    buttons: Query<&TalkLineButton>,
    talk: Query<&TradeTalk>,
    trade_ui_state: Res<TradeUiState>,
    hands: Query<&PlayerTradeCards>,
    mut trade_phase_state: ResMut<TradePhaseState>,
    mut create_offer_state: ResMut<CreateOfferState>,
    mut talk_draft: ResMut<TalkDraft>,
) {
    let Some(hand) = trade_ui_state.human_player.and_then(|h| hands.get(h).ok()) else {
        return;
    };
    for btn in activated.read().filter_map(|a| buttons.get(a.0).ok()) {
        let Ok(t) = talk.get(btn.talk) else {
            continue;
        };
        match btn.action {
            LineAction::Reply => {
                *talk_draft = TalkDraft::replying(t, btn.talk, hand);
                trade_phase_state.talk_modal_open = true;
            }
            LineAction::Propose => {
                create_offer_state.prefill(ProposalDraft::answering(t, btn.talk, hand));
                trade_phase_state.create_offer_modal_open = true;
            }
        }
    }
}

/// Counter, Outbid and Withdraw on a proposal.
pub fn handle_proposal_action_buttons(
    mut activated: MessageReader<ButtonActivated>,
    buttons: Query<&ProposalActionButton>,
    mut offers: Query<&mut OpenTradeOffer>,
    trade_ui_state: Res<TradeUiState>,
    hands: Query<&PlayerTradeCards>,
    mut trade_phase_state: ResMut<TradePhaseState>,
    mut create_offer_state: ResMut<CreateOfferState>,
) {
    let Some(human) = trade_ui_state.human_player else {
        return;
    };
    let Ok(hand) = hands.get(human) else {
        return;
    };
    for btn in activated.read().filter_map(|a| buttons.get(a.0).ok()) {
        let Ok(mut offer) = offers.get_mut(btn.offer) else {
            continue;
        };
        match btn.action {
            ProposalAction::Counter if offer.can_accept(human) => {
                create_offer_state.prefill(ProposalDraft::countering(&offer, hand));
                trade_phase_state.create_offer_modal_open = true;
            }
            ProposalAction::Outbid => {
                create_offer_state.prefill(ProposalDraft::outbidding(&offer, hand));
                trade_phase_state.create_offer_modal_open = true;
            }
            ProposalAction::Withdraw if offer.creator == human && offer.accepted_by.is_none() => {
                offer.withdrawn = true;
            }
            _ => {}
        }
    }
}

// ── The "say something" dialog ─────────────────────────────────────────────

/// "💬 Say…" opens the composer, said to the whole table.
pub fn handle_say_button(
    mut activated: MessageReader<ButtonActivated>,
    buttons: Query<(), With<SayButton>>,
    mut trade_phase_state: ResMut<TradePhaseState>,
    mut talk_draft: ResMut<TalkDraft>,
    trade_ui_state: Res<TradeUiState>,
    hands: Query<&PlayerTradeCards>,
) {
    if trade_phase_state.human_done || !activated.read().any(|a| buttons.contains(a.0)) {
        return;
    }
    *talk_draft = TalkDraft {
        want: trade_ui_state
            .human_player
            .and_then(|h| hands.get(h).ok())
            .map_or_else(Vec::new, |hand| {
                crate::civilization::concepts::trade::cards_to_want(hand, 2)
            }),
        ..Default::default()
    };
    trade_phase_state.talk_modal_open = true;
}

fn chip(row: &mut ChildSpawnerCommands, card: TradeCard, have: bool) {
    row.spawn((
        game_button(),
        TalkCardToggle { card, have },
        Node {
            padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
            margin: UiRect::all(Val::Px(3.0)),
            border: UiRect::all(Val::Px(2.0)),
            ..Default::default()
        },
        BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
        BorderColor::all(commodity_color(card)),
    ))
    .with_child((
        Text::new(card.to_string()),
        TextFont {
            font_size: FontSize::Px(13.0),
            ..Default::default()
        },
        TextColor(Color::WHITE),
    ));
}

fn chip_row(dialog: &mut ChildSpawnerCommands, cards: &[TradeCard], have: bool) {
    dialog
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            margin: UiRect::bottom(Val::Px(8.0)),
            ..Default::default()
        })
        .with_children(|row| {
            for card in cards {
                chip(row, *card, have);
            }
        });
}

pub fn spawn_talk_composer(
    mut commands: Commands,
    trade_phase_state: Res<TradePhaseState>,
    modal: Query<(), With<TalkComposerModal>>,
    trade_ui_state: Res<TradeUiState>,
    hands: Query<&PlayerTradeCards>,
) {
    if !trade_phase_state.talk_modal_open || !modal.is_empty() {
        return;
    }
    let Some(hand) = trade_ui_state.human_player.and_then(|h| hands.get(h).ok()) else {
        return;
    };
    let mut all: Vec<TradeCard> = TradeCard::iter()
        .filter(TradeCardTrait::is_commodity)
        .collect();
    all.sort_by_key(TradeCardTrait::value);
    let mut held: Vec<TradeCard> = hand
        .commodity_cards()
        .into_iter()
        .filter(|(c, n)| *n > 0 && c.is_commodity())
        .map(|(c, _)| c)
        .collect();
    held.sort_by_key(TradeCardTrait::value);

    let label = |text: &str| {
        (
            Text::new(text),
            TextFont {
                font_size: FontSize::Px(15.0),
                ..Default::default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
        )
    };

    commands
        .spawn((
            TalkComposerModal,
            GlobalZIndex(100),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        ))
        .with_children(|scrim| {
            scrim
                .spawn((
                    Node {
                        width: Val::Percent(60.0),
                        max_height: Val::Percent(85.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(16.0)),
                        overflow: Overflow::scroll_y(),
                        ..Default::default()
                    },
                    ScrollPosition::default(),
                    BackgroundColor(Color::srgba(0.12, 0.12, 0.18, 0.98)),
                ))
                .with_children(|dialog| {
                    dialog
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            margin: UiRect::bottom(Val::Px(10.0)),
                            ..Default::default()
                        })
                        .with_children(|header| {
                            header.spawn((
                                TalkComposerTitle,
                                Text::new("Say to the table"),
                                TextFont {
                                    font_size: FontSize::Px(20.0),
                                    ..Default::default()
                                },
                                TextColor(Color::srgb(0.9, 0.8, 0.3)),
                            ));
                            small_button(
                                header,
                                "X",
                                DONE_TRADING_PALETTE,
                                CloseTalkComposerButton,
                            );
                        });
                    dialog.spawn(label("I want:"));
                    chip_row(dialog, &all, false);
                    dialog.spawn(label("I have (only what you hold; nobody checks):"));
                    chip_row(dialog, &held, true);
                    dialog.spawn((
                        TalkPreviewText,
                        Text::new(""),
                        TextFont {
                            font_size: FontSize::Px(14.0),
                            ..Default::default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        Node {
                            margin: UiRect::vertical(Val::Px(8.0)),
                            ..Default::default()
                        },
                    ));
                    dialog
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::FlexEnd,
                            ..Default::default()
                        })
                        .with_children(|row| {
                            small_button(
                                row,
                                "Say it to the whole table",
                                CREATE_OFFER_PALETTE,
                                SayToTableButton,
                            );
                            small_button(row, "Say it", ACCEPT_OFFER_PALETTE, SayItButton);
                        });
                });
        });
}

/// Talk is free and non-binding, so saying it never pauses for long: it is
/// said, and the dialog closes.
#[allow(clippy::too_many_arguments)]
pub fn handle_talk_composer_buttons(
    mut commands: Commands,
    mut activated: MessageReader<ButtonActivated>,
    toggles: Query<&TalkCardToggle>,
    close: Query<(), With<CloseTalkComposerButton>>,
    say: Query<(), With<SayItButton>>,
    say_to_table: Query<(), With<SayToTableButton>>,
    mut talk_draft: ResMut<TalkDraft>,
    mut trade_phase_state: ResMut<TradePhaseState>,
    trade_ui_state: Res<TradeUiState>,
    names: Query<&Name>,
    mut talk: Query<&mut TradeTalk>,
) {
    for button in activated.read().map(|a| a.0) {
        if let Ok(toggle) = toggles.get(button) {
            talk_draft.toggle(toggle.card, toggle.have);
        } else if close.contains(button) {
            trade_phase_state.talk_modal_open = false;
        } else if say.contains(button) || say_to_table.contains(button) {
            if say_to_table.contains(button) {
                talk_draft.to = None;
                talk_draft.thread = None;
            }
            let (Some(human), Some(kind)) = (trade_ui_state.human_player, talk_draft.kind()) else {
                continue;
            };
            if matches!(kind, TalkKind::Want(_)) {
                // A new wish replaces the old one.
                for mut t in &mut talk {
                    if t.speaker == human && matches!(t.kind, TalkKind::Want(_)) {
                        t.live = false;
                    }
                }
            }
            let name = names
                .get(human)
                .map_or_else(|_| "You".to_string(), ToString::to_string);
            let mut line = TradeTalk::new(human, name, kind)
                .in_thread(talk_draft.thread)
                .at(trade_phase_state.countdown_seconds);
            if let Some((to, to_name)) = talk_draft.to.clone() {
                line = line.to(to, to_name);
            }
            commands.spawn(line);
            trade_phase_state.talk_modal_open = false;
        }
    }
}

/// Keep the composer's chips, title and preview in step with the draft.
pub fn update_talk_composer(
    talk_draft: Res<TalkDraft>,
    added: Query<(), Added<TalkComposerModal>>,
    mut chips: Query<(&TalkCardToggle, &mut BackgroundColor, &Children)>,
    mut chip_text: Query<&mut TextColor>,
    mut title: Query<&mut Text, (With<TalkComposerTitle>, Without<TalkPreviewText>)>,
    mut preview: Query<&mut Text, (With<TalkPreviewText>, Without<TalkComposerTitle>)>,
) {
    if !talk_draft.is_changed() && added.is_empty() {
        return;
    }
    for (chip, mut bg, children) in &mut chips {
        let list = if chip.have {
            &talk_draft.have
        } else {
            &talk_draft.want
        };
        let color = if list.contains(&chip.card) {
            commodity_color(chip.card)
        } else {
            Color::srgb(0.2, 0.2, 0.25)
        };
        *bg = BackgroundColor(color);
        for child in children.iter() {
            if let Ok(mut ink) = chip_text.get_mut(child) {
                *ink = TextColor(ink_on(color));
            }
        }
    }
    for mut text in &mut title {
        text.0 = match &talk_draft.to {
            Some((_, name)) => format!("Reply to {name}"),
            None => "Say to the table".to_string(),
        };
    }
    for mut text in &mut preview {
        text.0 = match talk_draft.kind() {
            Some(kind) => {
                let mut line = TradeTalk::new(Entity::PLACEHOLDER, "You", kind);
                if let Some((to, name)) = talk_draft.to.clone() {
                    line = line.to(to, name);
                }
                format!("\"{}\"", line.line())
            }
            None => "Pick what you want, what you have, or both.".to_string(),
        };
    }
}

pub fn despawn_talk_composer(
    mut commands: Commands,
    trade_phase_state: Res<TradePhaseState>,
    modal: Query<Entity, With<TalkComposerModal>>,
) {
    if trade_phase_state.talk_modal_open {
        return;
    }
    for entity in &modal {
        commands.entity(entity).despawn();
    }
}
