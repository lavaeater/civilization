//! Composing a proposal (or a counter), the way a proposal reads when it
//! arrives: "You give" and "You get" with the cards on them. Cards go in by
//! clicking them in your hand or in the list of commodities, and come out by
//! clicking them in the deal -- no arrow buttons.

use crate::civilization::TradeCardTrait;
use crate::civilization::concepts::acquire_trade_cards::{PlayerTradeCards, TradeCard};
use crate::civilization::concepts::trade::trade_components::{CanTrade, CreateOfferModal};
use crate::civilization::concepts::trade::trade_resources::{
    CreateOfferState, TradePhaseState, TradeUiState,
};
use crate::civilization::concepts::trade::trade_systems::{
    CloseCreateOfferModalButton, OfferValidationDisplay, OfferValueDisplay, PublishOfferButton,
    commodity_color, ink_on,
};
use crate::civilization::concepts::trade::trade_table_ui::small_button;
use crate::civilization::ui::{ButtonActivated, game_button};
use crate::stupid_ai::IsHuman;
use bevy::prelude::*;
use lava_ui_builder::InteractionPalette;

/// Named cards on each side of a proposal (the game's trade rule).
pub const NAMED_PER_SIDE: usize = 2;
/// Hidden-card counts offered as pills.
const HIDDEN_CHOICES: std::ops::RangeInclusive<usize> = 0..=4;

const LABEL: Color = Color::srgb(0.75, 0.75, 0.8);
const HINT: Color = Color::srgb(0.55, 0.55, 0.6);
const GIVE: Color = Color::srgb(0.95, 0.7, 0.45);
const GET: Color = Color::srgb(0.6, 0.95, 0.6);
const PILL: Color = Color::srgb(0.22, 0.22, 0.28);
const PILL_ON: Color = Color::srgb(0.25, 0.55, 0.85);

const CLOSE_PALETTE: InteractionPalette = InteractionPalette {
    none: Color::srgb(0.45, 0.2, 0.2),
    hovered: Color::srgb(0.6, 0.25, 0.25),
    pressed: Color::srgb(0.35, 0.15, 0.15),
};

/// What a click in the composer does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComposerAction {
    Give(TradeCard),
    TakeBack(TradeCard),
    Ask(TradeCard),
    Unask(TradeCard),
    HiddenGive(usize),
    HiddenAsk(usize),
    Claim(Option<TradeCard>),
    Target(Entity),
}

#[derive(Component, Clone, Copy, Debug)]
pub struct ComposerChip(pub ComposerAction);

/// The part of the composer rebuilt whenever the draft changes.
#[derive(Component, Default)]
pub struct ComposerBody;

/// Apply one click to the draft. `held` is how many of a card the player
/// holds. Returns false when the click is refused (side full, card not held).
pub fn apply_composer_action(
    state: &mut CreateOfferState,
    action: ComposerAction,
    held: impl Fn(TradeCard) -> usize,
) -> bool {
    fn take(side: &mut bevy::platform::collections::HashMap<TradeCard, usize>, card: TradeCard) {
        if let Some(n) = side.get_mut(&card) {
            *n -= 1;
            if *n == 0 {
                side.remove(&card);
            }
        }
    }
    match action {
        ComposerAction::Give(card) => {
            let given = state.offering_guaranteed.get(&card).copied().unwrap_or(0);
            if state.guaranteed_offering_count() >= NAMED_PER_SIDE || given >= held(card) {
                return false;
            }
            *state.offering_guaranteed.entry(card).or_insert(0) += 1;
        }
        ComposerAction::TakeBack(card) => take(&mut state.offering_guaranteed, card),
        ComposerAction::Ask(card) => {
            if state.guaranteed_wanting_count() >= NAMED_PER_SIDE {
                return false;
            }
            *state.wanting_guaranteed.entry(card).or_insert(0) += 1;
        }
        ComposerAction::Unask(card) => take(&mut state.wanting_guaranteed, card),
        ComposerAction::HiddenGive(n) => state.offering_hidden_count = n,
        ComposerAction::HiddenAsk(n) => state.wanting_hidden_count = n,
        ComposerAction::Claim(claim) => state.offering_claim = claim,
        ComposerAction::Target(player) => state.target = Some(player),
    }
    true
}

/// Opens the composer when the flag is set.
pub fn spawn_create_offer_modal(
    mut commands: Commands,
    trade_phase_state: Res<TradePhaseState>,
    modal_query: Query<Entity, With<CreateOfferModal>>,
    mut create_offer_state: ResMut<CreateOfferState>,
) {
    if !trade_phase_state.create_offer_modal_open || !modal_query.is_empty() {
        return;
    }
    // Start blank, unless the feed (a reply or a counter) filled it in.
    let title = if create_offer_state.prefilled {
        create_offer_state.prefilled = false;
        "Your proposal (filled in from the table: change what you like)"
    } else {
        create_offer_state.reset();
        // A deal is 3+ cards a side with 2 named: start with the hidden one.
        create_offer_state.offering_hidden_count = 1;
        create_offer_state.wanting_hidden_count = 1;
        "Your proposal"
    };

    commands
        .spawn((
            CreateOfferModal,
            GlobalZIndex(100),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|scrim| {
            scrim
                .spawn((
                    Node {
                        width: Val::Px(760.0),
                        max_width: Val::Percent(96.0),
                        max_height: Val::Percent(94.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(16.0)),
                        row_gap: Val::Px(6.0),
                        ..Default::default()
                    },
                    BackgroundColor(Color::srgba(0.12, 0.12, 0.17, 0.98)),
                ))
                .with_children(|dialog| {
                    dialog
                        .spawn(Node {
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            ..Default::default()
                        })
                        .with_children(|header| {
                            text(header, title, 20.0, Color::srgb(0.9, 0.8, 0.3));
                            small_button(
                                header,
                                "Close",
                                CLOSE_PALETTE,
                                CloseCreateOfferModalButton,
                            );
                        });
                    dialog.spawn((
                        ComposerBody,
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(8.0),
                            overflow: Overflow::scroll_y(),
                            min_height: Val::Px(0.0),
                            flex_shrink: 1.0,
                            ..Default::default()
                        },
                    ));
                    // Footer: what the deal does, what is missing, Propose.
                    dialog.spawn((
                        OfferValueDisplay,
                        Text::new(""),
                        TextFont {
                            font_size: FontSize::Px(14.0),
                            ..Default::default()
                        },
                        TextColor(Color::srgb(0.95, 0.85, 0.45)),
                    ));
                    dialog.spawn((
                        OfferValidationDisplay,
                        Text::new(""),
                        TextFont {
                            font_size: FontSize::Px(13.0),
                            ..Default::default()
                        },
                        TextColor(HINT),
                    ));
                    dialog
                        .spawn(Node {
                            justify_content: JustifyContent::FlexEnd,
                            margin: UiRect::top(Val::Px(6.0)),
                            ..Default::default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                game_button(),
                                PublishOfferButton,
                                Node {
                                    padding: UiRect::axes(Val::Px(28.0), Val::Px(10.0)),
                                    ..Default::default()
                                },
                                BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                            ))
                            .with_child((
                                Text::new("Propose"),
                                TextFont {
                                    font_size: FontSize::Px(17.0),
                                    ..Default::default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                });
        });
}

/// Clicks on cards, pills and names change the draft.
pub fn handle_composer_chips(
    mut activated: MessageReader<ButtonActivated>,
    chips: Query<&ComposerChip>,
    mut state: ResMut<CreateOfferState>,
    trade_ui_state: Res<TradeUiState>,
    hands: Query<&PlayerTradeCards, With<IsHuman>>,
) {
    let hand = trade_ui_state.human_player.and_then(|h| hands.get(h).ok());
    for ComposerChip(action) in activated.read().filter_map(|a| chips.get(a.0).ok()) {
        apply_composer_action(&mut state, *action, |card| {
            hand.map_or(0, |h| h.number_of_cards_for_trade_card(card))
        });
    }
}

/// Rebuild the composer's body from the draft.
#[allow(clippy::too_many_arguments)]
pub fn render_composer_body(
    mut commands: Commands,
    body: Query<Entity, With<ComposerBody>>,
    added: Query<(), Added<ComposerBody>>,
    mut state: ResMut<CreateOfferState>,
    trade_ui_state: Res<TradeUiState>,
    hands: Query<Ref<PlayerTradeCards>, With<IsHuman>>,
    traders: Query<(Entity, &Name), With<CanTrade>>,
    mut seen_traders: Local<Vec<Entity>>,
) {
    let Ok(body) = body.single() else {
        return;
    };
    let human = trade_ui_state.human_player;
    let mut partners: Vec<(Entity, String)> = traders
        .iter()
        .filter(|(e, _)| Some(*e) != human)
        .map(|(e, n)| (e, n.to_string()))
        .collect();
    partners.sort_by(|a, b| a.1.cmp(&b.1));
    // Keep the addressee someone still at the table.
    if !partners.iter().any(|(e, _)| Some(*e) == state.target) {
        let first = partners.first().map(|(e, _)| *e);
        if state.target != first {
            state.target = first;
        }
    }
    let Some(hand) = human.and_then(|h| hands.get(h).ok()) else {
        return;
    };
    let at_table: Vec<Entity> = partners.iter().map(|(e, _)| *e).collect();
    if !state.is_changed() && added.is_empty() && !hand.is_changed() && *seen_traders == at_table {
        return;
    }
    *seen_traders = at_table;

    let give_full = state.guaranteed_offering_count() >= NAMED_PER_SIDE;
    let ask_full = state.guaranteed_wanting_count() >= NAMED_PER_SIDE;
    let mut held: Vec<(TradeCard, usize)> = hand.commodity_cards().into_iter().collect();
    held.sort_by_key(|(card, _)| (card.value(), card.to_string()));
    let mut commodities: Vec<TradeCard> = TradeCard::iter()
        .filter(TradeCardTrait::is_commodity)
        .collect();
    commodities.sort_by_key(|card| (card.value(), card.to_string()));

    commands
        .entity(body)
        .despawn_related::<Children>()
        .with_children(|body| {
            // To whom.
            row(body, |row| {
                text(row, "To:", 15.0, LABEL);
                for (player, name) in &partners {
                    pill(
                        row,
                        name,
                        state.target == Some(*player),
                        ComposerAction::Target(*player),
                    );
                }
                if partners.is_empty() {
                    text(row, "nobody is left at the table", 14.0, HINT);
                }
            });

            // The deal, as it will read to them (in your words).
            body.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    ..Default::default()
                },
                BackgroundColor(Color::srgba(0.18, 0.18, 0.24, 0.9)),
            ))
            .with_children(|deal| {
                side_row(
                    deal,
                    "You give:",
                    GIVE,
                    &state.offering_guaranteed,
                    ComposerAction::TakeBack,
                    state.offering_hidden_count,
                    "pick 2 from your hand below",
                );
                side_row(
                    deal,
                    "You get:",
                    GET,
                    &state.wanting_guaranteed,
                    ComposerAction::Unask,
                    state.wanting_hidden_count,
                    "pick 2 to ask for below",
                );
                text(deal, "Click a card in the deal to take it out.", 12.0, HINT);
            });

            // Your hand.
            text(body, "Your hand: click a card to give it", 14.0, LABEL);
            wrap(body, |row| {
                for (card, count) in &held {
                    let given = state.offering_guaranteed.get(card).copied().unwrap_or(0);
                    for _ in given..*count {
                        card_chip(row, *card, !give_full, ComposerAction::Give(*card));
                    }
                }
                if held.is_empty() {
                    text(row, "no commodities in hand", 13.0, HINT);
                }
            });
            row(body, |row| {
                text(row, "Hidden cards you give:", 13.0, LABEL);
                for n in HIDDEN_CHOICES {
                    pill(
                        row,
                        &n.to_string(),
                        state.offering_hidden_count == n,
                        ComposerAction::HiddenGive(n),
                    );
                }
            });
            if state.offering_hidden_count > 0 {
                wrap(body, |row| {
                    text(row, "Say they are:", 13.0, LABEL);
                    pill(
                        row,
                        "nothing",
                        state.offering_claim.is_none(),
                        ComposerAction::Claim(None),
                    );
                    for card in &commodities {
                        pill(
                            row,
                            &card.to_string(),
                            state.offering_claim == Some(*card),
                            ComposerAction::Claim(Some(*card)),
                        );
                    }
                });
                text(
                    body,
                    "Hidden cards may be anything, whatever you say.",
                    12.0,
                    HINT,
                );
            }

            // What to ask for.
            text(body, "Ask for: click a card", 14.0, LABEL);
            wrap(body, |row| {
                for card in &commodities {
                    card_chip(row, *card, !ask_full, ComposerAction::Ask(*card));
                }
            });
            row(body, |row| {
                text(row, "Hidden cards you ask for:", 13.0, LABEL);
                for n in HIDDEN_CHOICES {
                    pill(
                        row,
                        &n.to_string(),
                        state.wanting_hidden_count == n,
                        ComposerAction::HiddenAsk(n),
                    );
                }
            });
        });
}

fn text(parent: &mut ChildSpawnerCommands, s: impl Into<String>, size: f32, color: Color) {
    parent.spawn((
        Text::new(s.into()),
        TextFont {
            font_size: FontSize::Px(size),
            ..Default::default()
        },
        TextColor(color),
    ));
}

fn row(parent: &mut ChildSpawnerCommands, f: impl FnOnce(&mut ChildSpawnerCommands)) {
    parent
        .spawn(Node {
            align_items: AlignItems::Center,
            column_gap: Val::Px(6.0),
            ..Default::default()
        })
        .with_children(f);
}

fn wrap(parent: &mut ChildSpawnerCommands, f: impl FnOnce(&mut ChildSpawnerCommands)) {
    parent
        .spawn(Node {
            flex_wrap: FlexWrap::Wrap,
            align_items: AlignItems::Center,
            column_gap: Val::Px(4.0),
            row_gap: Val::Px(4.0),
            ..Default::default()
        })
        .with_children(f);
}

/// A card in its commodity colour; dimmed when it can't be taken now.
fn card_chip(
    parent: &mut ChildSpawnerCommands,
    card: TradeCard,
    enabled: bool,
    action: ComposerAction,
) {
    let color = commodity_color(card);
    let (bg, ink) = if enabled {
        (color, ink_on(color))
    } else {
        (color.with_alpha(0.3), ink_on(color).with_alpha(0.5))
    };
    parent
        .spawn((
            game_button(),
            ComposerChip(action),
            Node {
                padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
                ..Default::default()
            },
            BackgroundColor(bg),
        ))
        .with_child((
            Text::new(card.to_string()),
            TextFont {
                font_size: FontSize::Px(14.0),
                ..Default::default()
            },
            TextColor(ink),
        ));
}

fn pill(parent: &mut ChildSpawnerCommands, label: &str, selected: bool, action: ComposerAction) {
    parent
        .spawn((
            game_button(),
            ComposerChip(action),
            Node {
                padding: UiRect::axes(Val::Px(9.0), Val::Px(3.0)),
                ..Default::default()
            },
            BackgroundColor(if selected { PILL_ON } else { PILL }),
        ))
        .with_child((
            Text::new(label.to_string()),
            TextFont {
                font_size: FontSize::Px(12.0),
                ..Default::default()
            },
            TextColor(Color::WHITE),
        ));
}

/// "You give: [Salt] [Ochre] + 1 hidden", the cards clickable to take out.
fn side_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    color: Color,
    side: &bevy::platform::collections::HashMap<TradeCard, usize>,
    remove: fn(TradeCard) -> ComposerAction,
    hidden: usize,
    hint: &str,
) {
    let mut cards: Vec<(TradeCard, usize)> = side.iter().map(|(c, n)| (*c, *n)).collect();
    cards.sort_by_key(|(card, _)| (std::cmp::Reverse(card.value()), card.to_string()));
    wrap(parent, |row| {
        text(row, label, 17.0, color);
        for (card, n) in &cards {
            for _ in 0..*n {
                card_chip(row, *card, true, remove(*card));
            }
        }
        let named: usize = cards.iter().map(|(_, n)| n).sum();
        if named < NAMED_PER_SIDE {
            text(
                row,
                format!("({} more: {hint})", NAMED_PER_SIDE - named),
                13.0,
                HINT,
            );
        }
        if hidden > 0 {
            text(row, format!("+ {hidden} hidden"), 15.0, color);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cards_go_in_by_clicking_and_come_out_the_same_way() {
        let mut state = CreateOfferState::default();
        let held = |card: TradeCard| if card == TradeCard::Salt { 1 } else { 3 };

        assert!(apply_composer_action(
            &mut state,
            ComposerAction::Give(TradeCard::Salt),
            held
        ));
        assert!(
            !apply_composer_action(&mut state, ComposerAction::Give(TradeCard::Salt), held),
            "only one Salt in hand"
        );
        assert!(apply_composer_action(
            &mut state,
            ComposerAction::Give(TradeCard::Ochre),
            held
        ));
        assert!(
            !apply_composer_action(&mut state, ComposerAction::Give(TradeCard::Ochre), held),
            "two named cards a side"
        );
        assert_eq!(state.guaranteed_offering_count(), 2);

        apply_composer_action(&mut state, ComposerAction::TakeBack(TradeCard::Salt), held);
        assert_eq!(state.offering_guaranteed.get(&TradeCard::Salt), None);

        assert!(apply_composer_action(
            &mut state,
            ComposerAction::Ask(TradeCard::Gold),
            held
        ));
        assert!(apply_composer_action(
            &mut state,
            ComposerAction::Ask(TradeCard::Gold),
            held
        ));
        assert!(!apply_composer_action(
            &mut state,
            ComposerAction::Ask(TradeCard::Iron),
            held
        ));
        apply_composer_action(&mut state, ComposerAction::Unask(TradeCard::Gold), held);
        assert_eq!(state.wanting_guaranteed.get(&TradeCard::Gold), Some(&1));

        apply_composer_action(&mut state, ComposerAction::HiddenGive(2), held);
        apply_composer_action(&mut state, ComposerAction::HiddenAsk(1), held);
        assert_eq!(
            (state.offering_hidden_count, state.wanting_hidden_count),
            (2, 1)
        );
    }
}
