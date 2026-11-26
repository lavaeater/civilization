//! The human's side of the trade table (docs/trade-dialog-design.md, step 2):
//! the feed of talk and proposals, grouped by thread, the buttons that turn a
//! line into a pre-filled reply or proposal, and the "say something" dialog.

use crate::civilization::concepts::acquire_trade_cards::{PlayerTradeCards, TradeCard};
use crate::civilization::concepts::trade::trade_components::{
    CanTrade, OpenOffersListContainer, OpenTradeOffer,
};
use crate::civilization::concepts::trade::trade_functions::offer_hand_values;
use crate::civilization::concepts::trade::trade_resources::{
    CreateOfferState, TradePhaseState, TradeUiState,
};
use crate::civilization::concepts::trade::trade_systems::{
    ACCEPT_OFFER_PALETTE, AcceptOfferButton, CREATE_OFFER_PALETTE, DONE_TRADING_PALETTE,
    DeclineOfferButton, commodity_color,
};
use crate::civilization::concepts::trade::trade_talk::{
    ProposalDraft, TalkDraft, TalkKind, TradeTalk,
};
use crate::civilization::ui::{ButtonActivated, game_button};
use crate::civilization::{TradeCardTrait, hidden_text};
use crate::stupid_ai::IsHuman;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use lava_ui_builder::InteractionPalette;

/// Threads shown in the feed; the rest have gone quiet.
const THREADS_SHOWN: usize = 10;
/// Statements shown per thread: the opening one and the latest few.
const LINES_PER_THREAD: usize = 6;

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

// ── Feed ────────────────────────────────────────────────────────────────────

enum FeedItem<'a> {
    Talk(Entity, &'a TradeTalk),
    Offer(Entity, &'a OpenTradeOffer),
}

impl FeedItem<'_> {
    /// When it was said; the countdown runs down, so higher is older.
    fn at(&self) -> u32 {
        match self {
            FeedItem::Talk(_, t) => t.said_at,
            FeedItem::Offer(_, o) => o.posted_at,
        }
    }
}

/// "2x Iron, Salt" for one named side of a proposal.
fn side_text(side: &HashMap<TradeCard, usize>) -> String {
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

fn small_button(
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

fn feed_text(parent: &mut ChildSpawnerCommands, text: String, size: f32, color: Color) {
    parent.spawn((
        Text::new(text),
        TextFont {
            font_size: FontSize::Px(size),
            ..Default::default()
        },
        TextColor(color),
        Node {
            flex_grow: 1.0,
            flex_shrink: 1.0,
            ..Default::default()
        },
    ));
}

const TALK_COLOR: Color = Color::srgb(0.65, 0.7, 0.75);
const TO_ME_COLOR: Color = Color::srgb(0.95, 0.85, 0.4);
const MINE_COLOR: Color = Color::srgb(0.6, 0.75, 0.95);

/// The table, grouped by thread: the threads with a proposal waiting on the
/// human first, then the liveliest. Each thread shows its opening line and
/// its latest statements, oldest first, with what can be done about each.
#[allow(clippy::too_many_arguments)]
pub fn update_trade_table_feed(
    mut commands: Commands,
    changed_offers: Query<(), Changed<OpenTradeOffer>>,
    offers: Query<(Entity, &OpenTradeOffer)>,
    new_talk: Query<(), Added<TradeTalk>>,
    talk: Query<(Entity, &TradeTalk)>,
    mut removed_offers: RemovedComponents<OpenTradeOffer>,
    mut removed_talk: RemovedComponents<TradeTalk>,
    container_query: Query<Entity, With<OpenOffersListContainer>>,
    trade_ui_state: Res<TradeUiState>,
    trade_phase_state: Res<TradePhaseState>,
    hands: Query<&PlayerTradeCards>,
    changed_hand: Query<(), (With<IsHuman>, Changed<PlayerTradeCards>)>,
    at_table: Query<(), With<CanTrade>>,
) {
    let left = removed_offers.read().count() + removed_talk.read().count() > 0;
    if changed_offers.is_empty() && new_talk.is_empty() && changed_hand.is_empty() && !left {
        return;
    }
    let Ok(container) = container_query.single() else {
        return;
    };
    commands.entity(container).despawn_related::<Children>();

    let human = trade_ui_state.human_player;
    let human_trading =
        !trade_phase_state.human_done && human.is_some_and(|h| at_table.contains(h));
    let is_human = |e: Entity| human == Some(e);

    // Gather everything into threads.
    let talk_ids: Vec<Entity> = talk.iter().map(|(e, _)| e).collect();
    let root_of = |own: Entity, thread: Option<Entity>| {
        thread.filter(|t| talk_ids.contains(t)).unwrap_or(own)
    };
    let mut threads: HashMap<Entity, Vec<FeedItem>> = HashMap::default();
    for (entity, t) in &talk {
        // Others' "no deal"s are noise; only those to or from us show.
        if matches!(t.kind, TalkKind::Decline(_)) && !is_human(t.speaker) && t.to != human {
            continue;
        }
        threads
            .entry(root_of(entity, t.thread))
            .or_default()
            .push(FeedItem::Talk(entity, t));
    }
    for (entity, offer) in &offers {
        if offer.withdrawn && offer.accepted_by.is_none() {
            continue;
        }
        threads
            .entry(root_of(entity, offer.thread))
            .or_default()
            .push(FeedItem::Offer(entity, offer));
    }

    if threads.is_empty() {
        commands.entity(container).with_children(|parent| {
            feed_text(
                parent,
                "Nobody has said anything yet. Say what you want, or propose a deal.".into(),
                14.0,
                Color::srgb(0.5, 0.5, 0.5),
            );
        });
        return;
    }

    let waits_on_me = |items: &Vec<FeedItem>| {
        items.iter().any(|i| match i {
            FeedItem::Offer(_, o) => human.is_some_and(|h| o.can_accept(h)),
            FeedItem::Talk(..) => false,
        })
    };
    let mut ordered: Vec<(Entity, Vec<FeedItem>)> = threads.into_iter().collect();
    for (_, items) in &mut ordered {
        // Oldest first within a thread.
        items.sort_by_key(|i| std::cmp::Reverse(i.at()));
    }
    ordered.sort_by_key(|(root, items)| {
        (
            !waits_on_me(items),
            items.iter().map(FeedItem::at).min().unwrap_or(0),
            *root,
        )
    });

    let my_hand: HashMap<TradeCard, usize> = human
        .and_then(|h| hands.get(h).ok())
        .map_or_else(HashMap::default, |h| {
            h.commodity_cards().into_iter().collect()
        });

    commands.entity(container).with_children(|parent| {
        for (_, items) in ordered.iter().take(THREADS_SHOWN) {
            let mine_waiting = waits_on_me(items);
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
                        margin: UiRect::bottom(Val::Px(6.0)),
                        border: UiRect::left(Val::Px(3.0)),
                        ..Default::default()
                    },
                    BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.7)),
                    BorderColor::all(if mine_waiting {
                        TO_ME_COLOR
                    } else {
                        Color::srgb(0.3, 0.3, 0.4)
                    }),
                ))
                .with_children(|thread| {
                    // The opening line, then the latest few.
                    let skip = items.len().saturating_sub(LINES_PER_THREAD);
                    for (i, item) in items.iter().enumerate() {
                        if i > 0 && i <= skip {
                            if i == 1 {
                                feed_text(
                                    thread,
                                    format!("  … {skip} earlier"),
                                    11.0,
                                    Color::srgb(0.45, 0.45, 0.5),
                                );
                            }
                            continue;
                        }
                        match item {
                            FeedItem::Talk(entity, t) => spawn_talk_line(
                                thread,
                                *entity,
                                t,
                                human,
                                human_trading && at_table.contains(t.speaker),
                            ),
                            FeedItem::Offer(entity, offer) => spawn_proposal(
                                thread,
                                *entity,
                                offer,
                                human,
                                human_trading,
                                &my_hand,
                                &at_table,
                            ),
                        }
                    }
                });
        }
        let quiet = ordered.len().saturating_sub(THREADS_SHOWN);
        if quiet > 0 {
            feed_text(
                parent,
                format!("{quiet} quieter thread(s) not shown"),
                11.0,
                Color::srgb(0.45, 0.45, 0.5),
            );
        }
    });
}

fn spawn_talk_line(
    thread: &mut ChildSpawnerCommands,
    entity: Entity,
    t: &TradeTalk,
    human: Option<Entity>,
    can_answer: bool,
) {
    let from_me = human == Some(t.speaker);
    let color = if from_me {
        MINE_COLOR
    } else if human.is_some() && t.to == human {
        TO_ME_COLOR
    } else {
        TALK_COLOR
    };
    let indent = if t.thread.is_some() { 12.0 } else { 0.0 };
    thread
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            margin: UiRect {
                left: Val::Px(indent),
                bottom: Val::Px(2.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|row| {
            feed_text(row, format!("💬 {}", t.line()), 12.0, color);
            let answerable = !matches!(t.kind, TalkKind::Decline(_));
            if can_answer && !from_me && answerable {
                small_button(
                    row,
                    "Reply",
                    CREATE_OFFER_PALETTE,
                    TalkLineButton {
                        talk: entity,
                        action: LineAction::Reply,
                    },
                );
                small_button(
                    row,
                    "Propose",
                    ACCEPT_OFFER_PALETTE,
                    TalkLineButton {
                        talk: entity,
                        action: LineAction::Propose,
                    },
                );
            }
        });
}

fn spawn_proposal(
    thread: &mut ChildSpawnerCommands,
    entity: Entity,
    offer: &OpenTradeOffer,
    human: Option<Entity>,
    human_trading: bool,
    my_hand: &HashMap<TradeCard, usize>,
    at_table: &Query<(), With<CanTrade>>,
) {
    let mine = human == Some(offer.creator);
    let to_me = human.is_some_and(|h| offer.can_accept(h));
    let color = if to_me {
        TO_ME_COLOR
    } else if mine {
        MINE_COLOR
    } else {
        Color::srgb(0.8, 0.8, 0.8)
    };
    let wanting_hidden = if offer.wanting_hidden_count > 0 {
        format!(" +{} hidden", offer.wanting_hidden_count)
    } else {
        String::new()
    };
    let to = offer.target_name.as_deref().unwrap_or("?");
    let line = format!(
        "📜 {} -> {to}: gives {}{} for {}{wanting_hidden}",
        offer.creator_name,
        side_text(&offer.offering_guaranteed),
        hidden_text(offer.offering_hidden_count, &offer.offering_claimed),
        side_text(&offer.wanting_guaranteed),
    );

    thread
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(4.0)),
                margin: UiRect {
                    left: Val::Px(12.0),
                    bottom: Val::Px(3.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            BackgroundColor(if to_me {
                Color::srgba(0.3, 0.27, 0.1, 0.6)
            } else {
                Color::srgba(0.1, 0.1, 0.14, 0.6)
            }),
        ))
        .with_children(|card| {
            feed_text(card, line, 12.0, color);
            if let Some(by) = &offer.accepted_by_name {
                feed_text(
                    card,
                    format!("✅ Accepted by {by}"),
                    11.0,
                    Color::srgb(0.5, 0.9, 0.5),
                );
                return;
            }
            if to_me {
                // What taking it does to our hand: we give what it asks for.
                let (before, after) = offer_hand_values(
                    my_hand,
                    &offer.wanting_guaranteed,
                    offer.wanting_hidden_count,
                    &offer.offering_guaranteed,
                );
                let net = after as i64 - before as i64;
                let unknown = if offer.offering_hidden_count > 0 {
                    format!(" + {} hidden card(s) you get", offer.offering_hidden_count)
                } else {
                    String::new()
                };
                feed_text(
                    card,
                    format!(
                        "Your hand value: {before} -> {after} (net {net:+}){unknown}. Your hidden cards count as your cheapest."
                    ),
                    11.0,
                    Color::srgb(0.85, 0.85, 0.6),
                );
            }
            if !human_trading {
                return;
            }
            card.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexEnd,
                margin: UiRect::top(Val::Px(2.0)),
                ..Default::default()
            })
            .with_children(|buttons| {
                if to_me {
                    small_button(
                        buttons,
                        "Accept",
                        ACCEPT_OFFER_PALETTE,
                        AcceptOfferButton { offer: entity },
                    );
                    small_button(
                        buttons,
                        "Decline",
                        DONE_TRADING_PALETTE,
                        DeclineOfferButton { offer: entity },
                    );
                    small_button(
                        buttons,
                        "Counter",
                        CREATE_OFFER_PALETTE,
                        ProposalActionButton {
                            offer: entity,
                            action: ProposalAction::Counter,
                        },
                    );
                } else if mine {
                    small_button(
                        buttons,
                        "Withdraw",
                        DONE_TRADING_PALETTE,
                        ProposalActionButton {
                            offer: entity,
                            action: ProposalAction::Withdraw,
                        },
                    );
                } else if offer.target.is_some_and(|t| at_table.contains(t)) {
                    small_button(
                        buttons,
                        &format!("Outbid to {to}"),
                        CREATE_OFFER_PALETTE,
                        ProposalActionButton {
                            offer: entity,
                            action: ProposalAction::Outbid,
                        },
                    );
                }
            });
        });
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
    mut chips: Query<(&TalkCardToggle, &mut BackgroundColor)>,
    mut title: Query<&mut Text, (With<TalkComposerTitle>, Without<TalkPreviewText>)>,
    mut preview: Query<&mut Text, (With<TalkPreviewText>, Without<TalkComposerTitle>)>,
) {
    if !talk_draft.is_changed() && added.is_empty() {
        return;
    }
    for (chip, mut bg) in &mut chips {
        let list = if chip.have {
            &talk_draft.have
        } else {
            &talk_draft.want
        };
        *bg = BackgroundColor(if list.contains(&chip.card) {
            commodity_color(chip.card)
        } else {
            Color::srgb(0.2, 0.2, 0.25)
        });
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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    fn hand(cards: &[(TradeCard, usize)]) -> PlayerTradeCards {
        let mut hand = PlayerTradeCards::default();
        for (card, n) in cards {
            hand.add_trade_cards(*card, *n);
        }
        hand
    }

    /// The human sees Reply/Propose on others' talk, Accept/Decline/Counter
    /// only on a proposal made to them, and Outbid on one made to someone else.
    #[test]
    fn the_feed_offers_the_right_buttons() {
        let mut world = World::new();
        world.init_resource::<TradePhaseState>();
        let human = world
            .spawn((IsHuman, CanTrade, hand(&[(TradeCard::Iron, 2)])))
            .id();
        let b = world.spawn((CanTrade, hand(&[]))).id();
        let c = world.spawn((CanTrade, hand(&[]))).id();
        world.insert_resource(TradeUiState {
            human_player: Some(human),
        });
        world.spawn((OpenOffersListContainer, Node::default()));
        let want = world
            .spawn(TradeTalk::new(b, "B", TalkKind::Want(vec![TradeCard::Iron])).at(80.0))
            .id();
        let mut to_me = OpenTradeOffer::new(b, "B", Some(human), Some("Me".into()));
        to_me.thread = Some(want);
        to_me.offering_guaranteed = HashMap::from([(TradeCard::Salt, 2)]);
        to_me.wanting_guaranteed = HashMap::from([(TradeCard::Iron, 2)]);
        let to_me = world.spawn(to_me).id();
        let to_c = world
            .spawn(OpenTradeOffer::new(b, "B", Some(c), Some("C".into())))
            .id();

        world.run_system_once(update_trade_table_feed).unwrap();

        let mut q = world.query::<&TalkLineButton>();
        let lines: Vec<(Entity, LineAction)> = q.iter(&world).map(|b| (b.talk, b.action)).collect();
        assert!(lines.contains(&(want, LineAction::Reply)));
        assert!(lines.contains(&(want, LineAction::Propose)));
        let mut q = world.query::<&AcceptOfferButton>();
        let accept: Vec<Entity> = q.iter(&world).map(|b| b.offer).collect();
        assert_eq!(accept, vec![to_me], "only the proposal to the human");
        let mut q = world.query::<&ProposalActionButton>();
        let actions: Vec<(Entity, ProposalAction)> =
            q.iter(&world).map(|b| (b.offer, b.action)).collect();
        assert!(actions.contains(&(to_me, ProposalAction::Counter)));
        assert!(actions.contains(&(to_c, ProposalAction::Outbid)));
    }
}
