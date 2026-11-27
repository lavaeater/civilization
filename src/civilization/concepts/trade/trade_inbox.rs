//! The human's trade inbox (docs/trade-dialog-design.md, step 2b).
//!
//! The table talks all at once; a person reads one thing at a time. What
//! concerns the human -- proposals made to them, things said to them, wants
//! they could serve -- queues up and is shown as one card. Nothing replaces
//! the card until the human acts on it: answer, dismiss, or move on. The
//! rest of the table goes to a quiet log that only ever grows at the bottom.

use crate::civilization::concepts::acquire_trade_cards::{PlayerTradeCards, TradeCard};
use crate::civilization::concepts::trade::trade_components::{CanTrade, OpenTradeOffer};
use crate::civilization::concepts::trade::trade_functions::describe_outcome;
use crate::civilization::concepts::trade::trade_resources::{TradePhaseState, TradeUiState};
use crate::civilization::concepts::trade::trade_systems::{
    ACCEPT_OFFER_PALETTE, AcceptOfferButton, CREATE_OFFER_PALETTE, DONE_TRADING_PALETTE,
    DeclineOfferButton,
};
use crate::civilization::concepts::trade::trade_table_ui::{
    LineAction, ProposalAction, ProposalActionButton, TalkLineButton, side_text, small_button,
};
use crate::civilization::concepts::trade::trade_talk::{
    TalkKind, TradeTalk, cards_to_offer, wants_and_haves,
};
use crate::civilization::hidden_text;
use crate::civilization::ui::ButtonActivated;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use std::collections::VecDeque;

/// A proposal made to you: it waits on your answer.
pub const PRIORITY_PROPOSAL: u8 = 0;
/// Something said to you, or news about your own deals.
pub const PRIORITY_TO_YOU: u8 = 1;
/// Someone asks for cards you hold.
pub const PRIORITY_WANT: u8 = 2;

/// Lines kept in the table log.
const LOG_KEPT: usize = 200;
/// Lines shown when the log is open.
const LOG_SHOWN: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InboxKind {
    Talk(Entity),
    Proposal(Entity),
    /// News, such as one of your proposals being taken.
    Notice,
}

#[derive(Debug, Clone)]
pub struct InboxItem {
    pub kind: InboxKind,
    /// What was said, kept so the card can still show it once it is gone.
    pub line: String,
    pub priority: u8,
    pub speaker: Option<Entity>,
    /// The conversation it belongs to: dismissing mutes the whole of it.
    pub thread: Option<Entity>,
}

/// What waits on the human, and what the rest of the table said.
#[derive(Resource, Default)]
pub struct TradeInbox {
    pub current: Option<InboxItem>,
    pub queue: VecDeque<InboxItem>,
    pub muted_threads: HashSet<Entity>,
    pub muted_players: HashSet<Entity>,
    /// Proposals of ours already reported as taken.
    pub notified: HashSet<Entity>,
    /// Everything said at the table, oldest first.
    pub log: Vec<String>,
    pub log_open: bool,
    /// Bumped whenever the card has to be drawn again; nothing else redraws it.
    pub version: u32,
    /// Whether the current card is still on the table.
    pub current_live: bool,
}

impl TradeInbox {
    /// Behind everything as urgent or more, ahead of anything less urgent.
    pub fn push(&mut self, item: InboxItem) {
        let at = self
            .queue
            .iter()
            .position(|q| q.priority > item.priority)
            .unwrap_or(self.queue.len());
        self.queue.insert(at, item);
    }

    /// The next card that is still worth showing. What went stale while
    /// waiting is dropped unseen.
    pub fn advance(&mut self, live: impl Fn(&InboxItem) -> bool) {
        self.current = None;
        while let Some(next) = self.queue.pop_front() {
            if live(&next) {
                self.current = Some(next);
                break;
            }
        }
        self.current_live = self.current.is_some();
        self.version += 1;
    }

    pub fn waiting_proposals(&self) -> usize {
        self.queue
            .iter()
            .filter(|i| matches!(i.kind, InboxKind::Proposal(_)))
            .count()
    }

    /// Nothing more from this conversation, except proposals made to us.
    pub fn mute_thread(&mut self, thread: Entity) {
        self.muted_threads.insert(thread);
        self.queue
            .retain(|i| matches!(i.kind, InboxKind::Proposal(_)) || i.thread != Some(thread));
    }

    /// Nothing more said by this player, except proposals made to us.
    pub fn mute_player(&mut self, player: Entity) {
        self.muted_players.insert(player);
        self.queue
            .retain(|i| matches!(i.kind, InboxKind::Proposal(_)) || i.speaker != Some(player));
    }

    fn muted(&self, speaker: Entity, thread: Entity) -> bool {
        self.muted_players.contains(&speaker) || self.muted_threads.contains(&thread)
    }

    pub fn reset(&mut self) {
        *self = Self {
            version: self.version + 1,
            ..Self::default()
        };
    }
}

/// Whether a statement concerns `human`, and how urgently: said to them, or
/// said to the table asking for cards they hold. Others' conversations and
/// "no deal"s to others only go to the log.
pub fn talk_priority(t: &TradeTalk, human: Entity, hand: &PlayerTradeCards) -> Option<u8> {
    if t.speaker == human {
        return None;
    }
    if t.to == Some(human) {
        return Some(PRIORITY_TO_YOU);
    }
    if t.to.is_some() || matches!(t.kind, TalkKind::Decline(_)) {
        return None;
    }
    let (wants, _) = wants_and_haves(&t.kind);
    wants
        .iter()
        .any(|c| hand.number_of_cards_for_trade_card(*c) > 0)
        .then_some(PRIORITY_WANT)
}

fn proposal_line(offer: &OpenTradeOffer) -> String {
    let wanting_hidden = if offer.wanting_hidden_count > 0 {
        format!(" +{} hidden", offer.wanting_hidden_count)
    } else {
        String::new()
    };
    format!(
        "{} -> {}: gives {}{} for {}{wanting_hidden}",
        offer.creator_name,
        offer.target_name.as_deref().unwrap_or("?"),
        side_text(&offer.offering_guaranteed),
        hidden_text(offer.offering_hidden_count, &offer.offering_claimed),
        side_text(&offer.wanting_guaranteed),
    )
}

/// Whether the card is still on the table.
fn is_live(
    item: &InboxItem,
    human: Entity,
    talk: &Query<&TradeTalk>,
    offers: &Query<&OpenTradeOffer>,
    at_table: &Query<(), With<CanTrade>>,
) -> bool {
    match item.kind {
        InboxKind::Talk(e) => talk.get(e).is_ok_and(|t| at_table.contains(t.speaker)),
        InboxKind::Proposal(e) => offers.get(e).is_ok_and(|o| o.can_accept(human)),
        InboxKind::Notice => true,
    }
}

/// Files what was said: the log gets everything, the inbox what concerns
/// the human.
#[allow(clippy::too_many_arguments)]
pub fn collect_inbox(
    mut inbox: ResMut<TradeInbox>,
    trade_ui_state: Res<TradeUiState>,
    hands: Query<&PlayerTradeCards>,
    new_talk: Query<(Entity, &TradeTalk), Added<TradeTalk>>,
    new_offers: Query<(Entity, &OpenTradeOffer), Added<OpenTradeOffer>>,
    changed_offers: Query<(Entity, &OpenTradeOffer), Changed<OpenTradeOffer>>,
    talk: Query<&TradeTalk>,
    offers: Query<&OpenTradeOffer>,
    at_table: Query<(), With<CanTrade>>,
) {
    let Some(human) = trade_ui_state.human_player else {
        return;
    };
    let Ok(hand) = hands.get(human) else {
        return;
    };

    let mut said: Vec<(u32, Entity, &TradeTalk)> =
        new_talk.iter().map(|(e, t)| (t.said_at, e, t)).collect();
    // The countdown runs down: the highest was said first.
    said.sort_by_key(|(at, e, _)| (std::cmp::Reverse(*at), *e));
    for (_, entity, t) in said {
        inbox.log.push(format!("💬 {}", t.line()));
        let thread = t.thread.unwrap_or(entity);
        if let Some(priority) = talk_priority(t, human, hand)
            && !inbox.muted(t.speaker, thread)
        {
            inbox.push(InboxItem {
                kind: InboxKind::Talk(entity),
                line: t.line(),
                priority,
                speaker: Some(t.speaker),
                thread: Some(thread),
            });
        }
    }
    for (entity, offer) in &new_offers {
        inbox.log.push(format!("📜 {}", proposal_line(offer)));
        if offer.can_accept(human) {
            inbox.push(InboxItem {
                kind: InboxKind::Proposal(entity),
                line: proposal_line(offer),
                priority: PRIORITY_PROPOSAL,
                speaker: Some(offer.creator),
                thread: offer.thread,
            });
        }
    }
    for (entity, offer) in &changed_offers {
        if offer.creator == human
            && let Some(by) = &offer.accepted_by_name
            && inbox.notified.insert(entity)
        {
            inbox.push(InboxItem {
                kind: InboxKind::Notice,
                line: format!("{by} accepted your proposal. Choose the cards to settle it."),
                priority: PRIORITY_TO_YOU,
                speaker: offer.accepted_by,
                thread: offer.thread,
            });
        }
    }
    let excess = inbox.log.len().saturating_sub(LOG_KEPT);
    inbox.log.drain(..excess);

    let live = |item: &InboxItem| is_live(item, human, &talk, &offers, &at_table);
    if inbox.current.is_none() && !inbox.queue.is_empty() {
        inbox.advance(live);
    } else if let Some(current) = &inbox.current {
        let now = live(current);
        if now != inbox.current_live {
            inbox.current_live = now;
            inbox.version += 1;
        }
    }
}

// ── UI ──────────────────────────────────────────────────────────────────────

#[derive(Component, Default)]
pub struct InboxCardContainer;

#[derive(Component, Default)]
pub struct InboxCounterText;

#[derive(Component, Default)]
pub struct MyDealsContainer;

#[derive(Component, Default)]
pub struct TableTickerText;

#[derive(Component, Default)]
pub struct TableLogContainer;

#[derive(Component, Default)]
pub struct TableLogToggle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InboxAction {
    Dismiss,
    NoThanks,
    Reply,
    Propose,
    Mute,
    Accept,
    Counter,
    Next,
}

impl InboxAction {
    /// The key that presses it.
    fn key(self) -> KeyCode {
        match self {
            Self::Dismiss => KeyCode::KeyD,
            Self::NoThanks => KeyCode::KeyN,
            Self::Reply => KeyCode::KeyR,
            Self::Propose => KeyCode::KeyP,
            Self::Mute => KeyCode::KeyM,
            Self::Accept => KeyCode::KeyA,
            Self::Counter => KeyCode::KeyC,
            Self::Next => KeyCode::Enter,
        }
    }
}

/// A button on the inbox card.
#[derive(Component)]
pub struct InboxButton(pub InboxAction);

fn text(parent: &mut ChildSpawnerCommands, s: impl Into<String>, size: f32, color: Color) {
    parent.spawn((
        Text::new(s.into()),
        TextFont {
            font_size: FontSize::Px(size),
            ..Default::default()
        },
        TextColor(color),
        Node {
            margin: UiRect::bottom(Val::Px(4.0)),
            ..Default::default()
        },
    ));
}

fn label(action: InboxAction, words: &str) -> String {
    let key = match action.key() {
        KeyCode::Enter => "Enter".to_string(),
        k => format!("{k:?}").trim_start_matches("Key").to_string(),
    };
    format!("{words} [{key}]")
}

fn button_row(card: &mut ChildSpawnerCommands, f: impl FnOnce(&mut ChildSpawnerCommands)) {
    card.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        justify_content: JustifyContent::FlexEnd,
        margin: UiRect::top(Val::Px(8.0)),
        ..Default::default()
    })
    .with_children(f);
}

const CARD_TEXT: Color = Color::srgb(0.95, 0.92, 0.85);
const CARD_DIM: Color = Color::srgb(0.6, 0.6, 0.65);
const CARD_VALUE: Color = Color::srgb(0.85, 0.85, 0.6);

/// Draws the card again only when it changed: the current item, or whether
/// it is still on the table. New arrivals only move the counter.
#[allow(clippy::too_many_arguments)]
pub fn render_inbox_card(
    mut commands: Commands,
    inbox: Res<TradeInbox>,
    container: Query<Entity, With<InboxCardContainer>>,
    added: Query<(), Added<InboxCardContainer>>,
    mut counter: Query<&mut Text, With<InboxCounterText>>,
    mut drawn: Local<u32>,
    talk: Query<&TradeTalk>,
    offers: Query<&OpenTradeOffer>,
    trade_ui_state: Res<TradeUiState>,
    trade_phase_state: Res<TradePhaseState>,
    hands: Query<&PlayerTradeCards>,
) {
    if inbox.is_changed() || !added.is_empty() {
        let proposals = inbox.waiting_proposals();
        let waiting = inbox.queue.len();
        let line = match (waiting, proposals) {
            (0, _) => "Nothing else waiting.".to_string(),
            (n, 0) => format!("{n} more waiting"),
            (n, p) => format!("{n} more waiting, {p} of them proposals to you"),
        };
        for mut t in &mut counter {
            if t.0 != line {
                t.0.clone_from(&line);
            }
        }
    }
    if *drawn == inbox.version && added.is_empty() {
        return;
    }
    *drawn = inbox.version;
    let Ok(container) = container.single() else {
        return;
    };
    commands.entity(container).despawn_related::<Children>();
    let Some(human) = trade_ui_state.human_player else {
        return;
    };
    let hand = hands.get(human).ok();
    let trading = !trade_phase_state.human_done;

    commands.entity(container).with_children(|card| {
        let Some(item) = &inbox.current else {
            text(
                card,
                "Nothing waiting for you. Say what you want, or propose a deal.",
                15.0,
                CARD_DIM,
            );
            return;
        };
        if !inbox.current_live {
            let gone = match item.kind {
                InboxKind::Proposal(e) => match offers.get(e) {
                    Ok(o) if o.accepted_by == Some(human) => {
                        "You accepted. Choose the cards to settle it."
                    }
                    _ => "This proposal is off the table.",
                },
                _ => "They have left the table.",
            };
            text(card, item.line.clone(), 15.0, CARD_DIM);
            text(card, gone, 13.0, CARD_DIM);
            button_row(card, |row| {
                small_button(
                    row,
                    &label(InboxAction::Next, "Next"),
                    CREATE_OFFER_PALETTE,
                    InboxButton(InboxAction::Next),
                );
            });
            return;
        }
        match item.kind {
            InboxKind::Notice => {
                text(card, item.line.clone(), 16.0, CARD_TEXT);
                button_row(card, |row| {
                    small_button(
                        row,
                        &label(InboxAction::Next, "OK"),
                        CREATE_OFFER_PALETTE,
                        InboxButton(InboxAction::Next),
                    );
                });
            }
            InboxKind::Talk(entity) => {
                let Ok(t) = talk.get(entity) else {
                    return;
                };
                let who = if t.to.is_some() {
                    format!("{} says to you", t.speaker_name)
                } else {
                    format!("{} says to everyone", t.speaker_name)
                };
                text(card, who, 13.0, CARD_DIM);
                let said = t
                    .line()
                    .split_once(": ")
                    .map_or_else(|| t.line(), |(_, s)| s.to_string());
                text(card, format!("\"{said}\""), 18.0, CARD_TEXT);
                if let Some(hand) = hand {
                    let (wants, _) = wants_and_haves(&t.kind);
                    let held = cards_to_offer(hand, wants, &[]);
                    if !held.is_empty() {
                        let held: Vec<String> = held
                            .iter()
                            .map(|c| format!("{c} x{}", hand.number_of_cards_for_trade_card(*c)))
                            .collect();
                        text(
                            card,
                            format!("You hold: {}", held.join(", ")),
                            13.0,
                            CARD_VALUE,
                        );
                    }
                }
                let is_decline = matches!(t.kind, TalkKind::Decline(_));
                button_row(card, |row| {
                    if is_decline || !trading {
                        small_button(
                            row,
                            &label(InboxAction::Next, "OK"),
                            CREATE_OFFER_PALETTE,
                            InboxButton(InboxAction::Next),
                        );
                        return;
                    }
                    small_button(
                        row,
                        &label(InboxAction::Dismiss, "Dismiss"),
                        DONE_TRADING_PALETTE,
                        InboxButton(InboxAction::Dismiss),
                    );
                    small_button(
                        row,
                        &label(InboxAction::NoThanks, "No thanks"),
                        DONE_TRADING_PALETTE,
                        InboxButton(InboxAction::NoThanks),
                    );
                    small_button(
                        row,
                        &label(InboxAction::Mute, &format!("Mute {}", t.speaker_name)),
                        DONE_TRADING_PALETTE,
                        InboxButton(InboxAction::Mute),
                    );
                    small_button(
                        row,
                        &label(InboxAction::Reply, "Reply"),
                        CREATE_OFFER_PALETTE,
                        (
                            InboxButton(InboxAction::Reply),
                            TalkLineButton {
                                talk: entity,
                                action: LineAction::Reply,
                            },
                        ),
                    );
                    small_button(
                        row,
                        &label(InboxAction::Propose, "Propose"),
                        ACCEPT_OFFER_PALETTE,
                        (
                            InboxButton(InboxAction::Propose),
                            TalkLineButton {
                                talk: entity,
                                action: LineAction::Propose,
                            },
                        ),
                    );
                });
            }
            InboxKind::Proposal(entity) => {
                let Ok(offer) = offers.get(entity) else {
                    return;
                };
                text(
                    card,
                    format!("{} proposes a deal to you", offer.creator_name),
                    13.0,
                    CARD_DIM,
                );
                text(
                    card,
                    format!(
                        "You get: {}{}",
                        side_text(&offer.offering_guaranteed),
                        hidden_text(offer.offering_hidden_count, &offer.offering_claimed)
                    ),
                    17.0,
                    Color::srgb(0.6, 0.95, 0.6),
                );
                let hidden = if offer.wanting_hidden_count > 0 {
                    format!(
                        " + {} hidden card(s) of your choice",
                        offer.wanting_hidden_count
                    )
                } else {
                    String::new()
                };
                text(
                    card,
                    format!("You give: {}{hidden}", side_text(&offer.wanting_guaranteed)),
                    17.0,
                    Color::srgb(0.95, 0.7, 0.45),
                );
                if let Some(hand) = hand {
                    let holdings: HashMap<TradeCard, usize> =
                        hand.commodity_cards().into_iter().collect();
                    text(
                        card,
                        describe_outcome(
                            &holdings,
                            &offer.wanting_guaranteed,
                            offer.wanting_hidden_count,
                            &offer.offering_guaranteed,
                            offer.offering_hidden_count,
                        ),
                        14.0,
                        CARD_VALUE,
                    );
                }
                button_row(card, |row| {
                    small_button(
                        row,
                        &label(InboxAction::NoThanks, "No thanks"),
                        DONE_TRADING_PALETTE,
                        (
                            InboxButton(InboxAction::NoThanks),
                            DeclineOfferButton { offer: entity },
                        ),
                    );
                    small_button(
                        row,
                        &label(InboxAction::Counter, "Counter"),
                        CREATE_OFFER_PALETTE,
                        (
                            InboxButton(InboxAction::Counter),
                            ProposalActionButton {
                                offer: entity,
                                action: ProposalAction::Counter,
                            },
                        ),
                    );
                    small_button(
                        row,
                        &label(InboxAction::Accept, "Accept"),
                        ACCEPT_OFFER_PALETTE,
                        (
                            InboxButton(InboxAction::Accept),
                            AcceptOfferButton { offer: entity },
                        ),
                    );
                });
            }
        }
    });
}

/// What the card's buttons do to the inbox. Reply, Propose, Counter, Accept
/// and No thanks on a proposal also carry the trade panel's own button
/// components, whose handlers do the trading; here the card moves on.
#[allow(clippy::too_many_arguments)]
pub fn handle_inbox_buttons(
    mut commands: Commands,
    mut activated: MessageReader<ButtonActivated>,
    buttons: Query<&InboxButton>,
    mut inbox: ResMut<TradeInbox>,
    trade_ui_state: Res<TradeUiState>,
    trade_phase_state: Res<TradePhaseState>,
    names: Query<&Name>,
    talk: Query<&TradeTalk>,
    offers: Query<&OpenTradeOffer>,
    at_table: Query<(), With<CanTrade>>,
) {
    let Some(human) = trade_ui_state.human_player else {
        return;
    };
    for InboxButton(action) in activated.read().filter_map(|a| buttons.get(a.0).ok()) {
        let Some(item) = inbox.current.clone() else {
            continue;
        };
        match action {
            // Accepting leaves the card up; it turns into "settle it".
            InboxAction::Accept => continue,
            InboxAction::Dismiss => {
                if let Some(thread) = item.thread {
                    inbox.mute_thread(thread);
                }
            }
            InboxAction::Mute => {
                if let Some(speaker) = item.speaker {
                    inbox.mute_player(speaker);
                }
            }
            InboxAction::NoThanks => {
                // A proposal's "no" is said by the Decline handler; talk's here.
                if let InboxKind::Talk(entity) = item.kind
                    && let Ok(t) = talk.get(entity)
                {
                    let name = names
                        .get(human)
                        .map_or_else(|_| "You".to_string(), ToString::to_string);
                    commands.spawn(
                        TradeTalk::new(human, name, TalkKind::Decline(None))
                            .to(t.speaker, t.speaker_name.clone())
                            .in_thread(item.thread)
                            .at(trade_phase_state.countdown_seconds),
                    );
                    if let Some(thread) = item.thread {
                        inbox.mute_thread(thread);
                    }
                }
            }
            InboxAction::Reply
            | InboxAction::Propose
            | InboxAction::Counter
            | InboxAction::Next => {}
        }
        inbox.advance(|i| is_live(i, human, &talk, &offers, &at_table));
    }
}

/// D, N, R, P, M, A, C and Enter press the card's buttons, unless a dialog
/// is open.
pub fn inbox_shortcuts(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    trade_phase_state: Res<TradePhaseState>,
    buttons: Query<(Entity, &InboxButton)>,
    mut activated: MessageWriter<ButtonActivated>,
) {
    let Some(keys) = keys else {
        return;
    };
    if trade_phase_state.create_offer_modal_open
        || trade_phase_state.talk_modal_open
        || trade_phase_state.settlement_modal_open
    {
        return;
    }
    for (entity, InboxButton(action)) in &buttons {
        if keys.just_pressed(action.key()) {
            activated.write(ButtonActivated(entity));
            return;
        }
    }
}

/// The latest line of the table, always; the last few when the log is open.
pub fn render_table_log(
    mut commands: Commands,
    inbox: Res<TradeInbox>,
    mut ticker: Query<&mut Text, With<TableTickerText>>,
    log: Query<(Entity, &mut Node), With<TableLogContainer>>,
    mut shown: Local<(usize, bool)>,
) {
    if !inbox.is_changed() {
        return;
    }
    if let Some(last) = inbox.log.last() {
        for mut t in &mut ticker {
            if t.0 != *last {
                t.0.clone_from(last);
            }
        }
    }
    let state = (inbox.log.len(), inbox.log_open);
    if *shown == state {
        return;
    }
    *shown = state;
    for (entity, mut node) in log {
        node.display = if inbox.log_open {
            Display::Flex
        } else {
            Display::None
        };
        commands.entity(entity).despawn_related::<Children>();
        if inbox.log_open {
            let from = inbox.log.len().saturating_sub(LOG_SHOWN);
            commands.entity(entity).with_children(|parent| {
                for line in &inbox.log[from..] {
                    text(parent, line.clone(), 11.0, CARD_DIM);
                }
            });
        }
    }
}

pub fn handle_table_log_toggle(
    mut activated: MessageReader<ButtonActivated>,
    toggles: Query<(), With<TableLogToggle>>,
    mut inbox: ResMut<TradeInbox>,
) {
    if activated.read().any(|a| toggles.contains(a.0)) {
        inbox.log_open = !inbox.log_open;
    }
}

/// Your own business at the table: what you said you want, and each
/// proposal you have out with where it stands.
pub fn render_my_deals(
    mut commands: Commands,
    trade_ui_state: Res<TradeUiState>,
    container: Query<Entity, With<MyDealsContainer>>,
    added: Query<(), Added<MyDealsContainer>>,
    changed_offers: Query<(), Changed<OpenTradeOffer>>,
    mut removed_offers: RemovedComponents<OpenTradeOffer>,
    new_talk: Query<(), Added<TradeTalk>>,
    offers: Query<(Entity, &OpenTradeOffer)>,
    talk: Query<&TradeTalk>,
    trade_phase_state: Res<TradePhaseState>,
) {
    let left = removed_offers.read().count() > 0;
    if added.is_empty() && changed_offers.is_empty() && new_talk.is_empty() && !left {
        return;
    }
    let (Some(human), Ok(container)) = (trade_ui_state.human_player, container.single()) else {
        return;
    };
    let want = talk
        .iter()
        .filter(|t| t.speaker == human && t.live && matches!(t.kind, TalkKind::Want(_)))
        .min_by_key(|t| t.said_at)
        .map(TradeTalk::line);
    let mut mine: Vec<(Entity, &OpenTradeOffer)> = offers
        .iter()
        .filter(|(_, o)| o.creator == human && !(o.withdrawn && o.accepted_by.is_none()))
        .collect();
    mine.sort_by_key(|(e, o)| (std::cmp::Reverse(o.posted_at), *e));
    commands.entity(container).despawn_related::<Children>();
    commands.entity(container).with_children(|parent| {
        match &want {
            Some(line) => text(
                parent,
                format!(
                    "Your want: {}",
                    line.split_once(": ").map_or(line.as_str(), |(_, s)| s)
                ),
                12.0,
                CARD_DIM,
            ),
            None => text(parent, "You have not said what you want.", 12.0, CARD_DIM),
        }
        for (entity, offer) in mine {
            let status = match &offer.accepted_by_name {
                Some(by) => format!("accepted by {by}: settle it"),
                None => "waiting".to_string(),
            };
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    ..Default::default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new(format!(
                            "📜 to {}: gives {} for {} ({status})",
                            offer.target_name.as_deref().unwrap_or("?"),
                            side_text(&offer.offering_guaranteed),
                            side_text(&offer.wanting_guaranteed),
                        )),
                        TextFont {
                            font_size: FontSize::Px(12.0),
                            ..Default::default()
                        },
                        TextColor(Color::srgb(0.6, 0.75, 0.95)),
                        Node {
                            flex_grow: 1.0,
                            ..Default::default()
                        },
                    ));
                    if offer.accepted_by.is_none() && !trade_phase_state.human_done {
                        small_button(
                            row,
                            "Withdraw",
                            DONE_TRADING_PALETTE,
                            ProposalActionButton {
                                offer: entity,
                                action: ProposalAction::Withdraw,
                            },
                        );
                    }
                });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(n: u32) -> Entity {
        Entity::from_raw_u32(n).unwrap()
    }

    fn item(kind: InboxKind, priority: u8, thread: u32, speaker: u32) -> InboxItem {
        InboxItem {
            kind,
            line: String::new(),
            priority,
            speaker: Some(e(speaker)),
            thread: Some(e(thread)),
        }
    }

    #[test]
    fn proposals_go_ahead_but_never_replace_the_card() {
        let mut inbox = TradeInbox::default();
        inbox.push(item(InboxKind::Talk(e(1)), PRIORITY_WANT, 1, 10));
        inbox.advance(|_| true);
        assert_eq!(inbox.current.as_ref().unwrap().kind, InboxKind::Talk(e(1)));

        inbox.push(item(InboxKind::Talk(e(2)), PRIORITY_WANT, 2, 11));
        inbox.push(item(InboxKind::Proposal(e(3)), PRIORITY_PROPOSAL, 3, 12));
        inbox.push(item(InboxKind::Talk(e(4)), PRIORITY_TO_YOU, 4, 13));
        // The card stays; the proposal waits first in line.
        assert_eq!(inbox.current.as_ref().unwrap().kind, InboxKind::Talk(e(1)));
        let order: Vec<InboxKind> = inbox.queue.iter().map(|i| i.kind).collect();
        assert_eq!(
            order,
            [
                InboxKind::Proposal(e(3)),
                InboxKind::Talk(e(4)),
                InboxKind::Talk(e(2))
            ]
        );
        assert_eq!(inbox.waiting_proposals(), 1);

        // What went stale while waiting is skipped.
        inbox.advance(|i| i.kind != InboxKind::Proposal(e(3)));
        assert_eq!(inbox.current.as_ref().unwrap().kind, InboxKind::Talk(e(4)));
    }

    #[test]
    fn muting_drops_talk_but_keeps_proposals() {
        let mut inbox = TradeInbox::default();
        inbox.push(item(InboxKind::Talk(e(1)), PRIORITY_WANT, 7, 10));
        inbox.push(item(InboxKind::Proposal(e(2)), PRIORITY_PROPOSAL, 7, 10));
        inbox.push(item(InboxKind::Talk(e(3)), PRIORITY_WANT, 8, 10));
        inbox.mute_thread(e(7));
        assert_eq!(inbox.queue.len(), 2);
        inbox.mute_player(e(10));
        let left: Vec<InboxKind> = inbox.queue.iter().map(|i| i.kind).collect();
        assert_eq!(left, [InboxKind::Proposal(e(2))]);
        assert!(inbox.muted(e(10), e(99)));
        assert!(inbox.muted(e(11), e(7)));
    }

    #[test]
    fn only_what_concerns_you_reaches_the_inbox() {
        let (me, b, c) = (e(1), e(2), e(3));
        let mut hand = PlayerTradeCards::default();
        hand.add_trade_cards(TradeCard::Hides, 2);
        let want = |cards: Vec<TradeCard>| TradeTalk::new(b, "B", TalkKind::Want(cards));
        assert_eq!(
            talk_priority(&want(vec![TradeCard::Hides]), me, &hand),
            Some(PRIORITY_WANT)
        );
        assert_eq!(talk_priority(&want(vec![TradeCard::Gold]), me, &hand), None);
        let to_me = TradeTalk::new(b, "B", TalkKind::Decline(None)).to(me, "Me");
        assert_eq!(talk_priority(&to_me, me, &hand), Some(PRIORITY_TO_YOU));
        let to_c = TradeTalk::new(
            b,
            "B",
            TalkKind::Reply {
                have: vec![],
                want: vec![TradeCard::Hides],
            },
        )
        .to(c, "C");
        assert_eq!(talk_priority(&to_c, me, &hand), None, "their conversation");
        let mine = TradeTalk::new(me, "Me", TalkKind::Want(vec![TradeCard::Hides]));
        assert_eq!(talk_priority(&mine, me, &hand), None);
    }

    mod flow {
        use super::super::*;
        use crate::civilization::concepts::trade::trade_resources::TradeUiState;
        use crate::stupid_ai::IsHuman;

        fn app() -> (App, Entity, Entity) {
            let mut app = App::new();
            app.add_message::<ButtonActivated>()
                .init_resource::<TradeInbox>()
                .init_resource::<TradePhaseState>()
                .add_systems(
                    Update,
                    (collect_inbox, handle_inbox_buttons, render_inbox_card).chain(),
                );
            let mut hand = PlayerTradeCards::default();
            hand.add_trade_cards(TradeCard::Hides, 2);
            let human = app.world_mut().spawn((IsHuman, CanTrade, hand)).id();
            let ai = app.world_mut().spawn(CanTrade).id();
            app.insert_resource(TradeUiState {
                human_player: Some(human),
            });
            app.world_mut().spawn((InboxCardContainer, Node::default()));
            (app, human, ai)
        }

        fn buttons(app: &mut App) -> Vec<(Entity, InboxAction)> {
            let mut q = app.world_mut().query::<(Entity, &InboxButton)>();
            q.iter(app.world()).map(|(e, b)| (e, b.0)).collect()
        }

        fn press(app: &mut App, action: InboxAction) {
            let button = buttons(app)
                .into_iter()
                .find(|(_, a)| *a == action)
                .expect("button on the card")
                .0;
            app.world_mut().write_message(ButtonActivated(button));
            app.update();
            app.update();
        }

        /// A want arrives, then a proposal: the want is on the card and
        /// stays there; the proposal comes next.
        #[test]
        fn one_card_at_a_time() {
            let (mut app, human, ai) = app();
            let want = app
                .world_mut()
                .spawn(TradeTalk::new(
                    ai,
                    "Illyria",
                    TalkKind::Want(vec![TradeCard::Hides]),
                ))
                .id();
            app.update();
            app.update();
            let mut offer = OpenTradeOffer::new(ai, "Illyria", Some(human), Some("Me".into()));
            offer.offering_guaranteed = HashMap::from([(TradeCard::Oil, 2)]);
            offer.wanting_guaranteed = HashMap::from([(TradeCard::Hides, 2)]);
            let offer = app.world_mut().spawn(offer).id();
            app.update();
            app.update();

            let inbox = app.world().resource::<TradeInbox>();
            assert_eq!(inbox.current.as_ref().unwrap().kind, InboxKind::Talk(want));
            assert_eq!(inbox.waiting_proposals(), 1);
            assert!(
                buttons(&mut app)
                    .iter()
                    .any(|(_, a)| *a == InboxAction::Dismiss)
            );

            // Dismissing the want brings up the proposal.
            press(&mut app, InboxAction::Dismiss);
            let inbox = app.world().resource::<TradeInbox>();
            assert_eq!(
                inbox.current.as_ref().unwrap().kind,
                InboxKind::Proposal(offer)
            );
            assert!(inbox.muted_threads.contains(&want));
            assert!(
                buttons(&mut app)
                    .iter()
                    .any(|(_, a)| *a == InboxAction::Accept)
            );

            // Withdrawn under our eyes: the card stays, greyed, until Next.
            app.world_mut()
                .get_mut::<OpenTradeOffer>(offer)
                .unwrap()
                .withdrawn = true;
            app.update();
            app.update();
            assert!(!app.world().resource::<TradeInbox>().current_live);
            assert_eq!(
                buttons(&mut app).iter().map(|b| b.1).collect::<Vec<_>>(),
                [InboxAction::Next]
            );
            press(&mut app, InboxAction::Next);
            assert!(app.world().resource::<TradeInbox>().current.is_none());
        }
    }
}
