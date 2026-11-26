//! Table talk for the dialog-based trade (docs/trade-dialog-design.md).
//!
//! Talk is free and never binding: a player says what it wants, answers with
//! what it has, or turns a deal down, and may lie while doing so. Only a
//! proposal (`OpenTradeOffer` addressed to one player) can be accepted, and
//! only by that player. Talk steers who proposes what to whom.

use crate::civilization::concepts::acquire_trade_cards::{
    PlayerTradeCards, TradeCard, TradeCardTrait,
};
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::{Component, Entity};

/// What was said.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TalkKind {
    /// "I want Iron!" Starts a thread.
    Want(Vec<TradeCard>),
    /// "I have Iron; I want Grain or Wine."
    Reply {
        have: Vec<TradeCard>,
        want: Vec<TradeCard>,
    },
    /// "No can do", with an optional reason.
    Decline(Option<String>),
}

/// One statement at the trade table. Spawned as its own entity.
#[derive(Component, Debug, Clone)]
pub struct TradeTalk {
    pub speaker: Entity,
    pub speaker_name: String,
    /// Who it is said to; `None` is said to the whole table.
    pub to: Option<Entity>,
    pub to_name: Option<String>,
    /// The Want this answers, if any.
    pub thread: Option<Entity>,
    pub kind: TalkKind,
    /// Trade countdown when it was said, in tenths of a second.
    pub said_at: u32,
    /// A Want stops counting once its speaker says a newer one.
    pub live: bool,
}

impl TradeTalk {
    pub fn new(speaker: Entity, speaker_name: impl Into<String>, kind: TalkKind) -> Self {
        Self {
            speaker,
            speaker_name: speaker_name.into(),
            to: None,
            to_name: None,
            thread: None,
            kind,
            said_at: 0,
            live: true,
        }
    }

    pub fn to(mut self, player: Entity, name: impl Into<String>) -> Self {
        self.to = Some(player);
        self.to_name = Some(name.into());
        self
    }

    pub fn in_thread(mut self, thread: Option<Entity>) -> Self {
        self.thread = thread;
        self
    }

    pub fn at(mut self, countdown: f32) -> Self {
        self.said_at = (countdown.max(0.0) * 10.0) as u32;
        self
    }

    /// "Babylon -> Egypt: I have Iron; I want Wine or Salt"
    pub fn line(&self) -> String {
        let who = match &self.to_name {
            Some(to) => format!("{} -> {to}", self.speaker_name),
            None => self.speaker_name.clone(),
        };
        let said = match &self.kind {
            TalkKind::Want(cards) => format!("I want {}!", card_list(cards, "and")),
            TalkKind::Reply { have, want } if want.is_empty() => {
                format!("I have {}.", card_list(have, "and"))
            }
            TalkKind::Reply { have, want } => format!(
                "I have {}; got any {}?",
                card_list(have, "and"),
                card_list(want, "or")
            ),
            TalkKind::Decline(Some(reason)) => format!("No deal: {reason}"),
            TalkKind::Decline(None) => "No deal.".to_string(),
        };
        format!("{who}: {said}")
    }
}

fn card_list(cards: &[TradeCard], joiner: &str) -> String {
    let names: Vec<String> = cards.iter().map(ToString::to_string).collect();
    match names.as_slice() {
        [] => "nothing".to_string(),
        [one] => one.clone(),
        [init @ .., last] => format!("{} {joiner} {last}", init.join(", ")),
    }
}

/// What the table has said about each player: the cards it asked for and the
/// cards it claimed to have, and whom it answered. All of it may be a bluff.
#[derive(Debug, Default)]
pub struct TableKnowledge {
    pub wants: HashMap<Entity, HashSet<TradeCard>>,
    pub haves: HashMap<Entity, HashSet<TradeCard>>,
    /// speaker -> players it replied to
    pub replied_to: HashMap<Entity, HashSet<Entity>>,
    /// speaker -> its live Want, to thread a proposal under
    pub want_talk: HashMap<Entity, Entity>,
}

impl TableKnowledge {
    pub fn from_talk<'a>(talk: impl Iterator<Item = (Entity, &'a TradeTalk)>) -> Self {
        let mut known = Self::default();
        for (entity, t) in talk {
            match &t.kind {
                TalkKind::Want(cards) if t.live => {
                    known
                        .wants
                        .entry(t.speaker)
                        .or_default()
                        .extend(cards.iter().copied());
                    known.want_talk.insert(t.speaker, entity);
                }
                TalkKind::Reply { have, want } => {
                    known
                        .haves
                        .entry(t.speaker)
                        .or_default()
                        .extend(have.iter().copied());
                    known
                        .wants
                        .entry(t.speaker)
                        .or_default()
                        .extend(want.iter().copied());
                    if let Some(to) = t.to {
                        known.replied_to.entry(t.speaker).or_default().insert(to);
                    }
                }
                _ => {}
            }
        }
        known
    }

    /// How well `partner` fits a deal where `me` gives `give` and asks for
    /// `want`, judged by the talk: it asked for what we give, it said it has
    /// what we want, and it spoke to us. 0 when it said nothing useful.
    pub fn partner_score(
        &self,
        me: Entity,
        partner: Entity,
        give: &HashMap<TradeCard, usize>,
        want: &HashMap<TradeCard, usize>,
    ) -> f32 {
        let empty = HashSet::default();
        let their_wants = self.wants.get(&partner).unwrap_or(&empty);
        let their_haves = self.haves.get(&partner).unwrap_or(&empty);
        let mut score = 0.0;
        if give.keys().any(|c| their_wants.contains(c)) {
            score += 2.0;
        }
        if want.keys().any(|c| their_haves.contains(c)) {
            score += 2.0;
        }
        if self
            .replied_to
            .get(&partner)
            .is_some_and(|to| to.contains(&me))
        {
            score += 1.0;
        }
        score
    }
}

/// The commodities a hand would like more of: its biggest sets first (a set
/// of n is worth v*n^2, rule 28.2), at most `max` of them.
pub fn cards_to_want(cards: &PlayerTradeCards, max: usize) -> Vec<TradeCard> {
    let mut held: Vec<(TradeCard, usize)> = cards
        .commodity_cards()
        .into_iter()
        .filter(|(c, n)| *n > 0 && c.is_commodity())
        .collect();
    held.sort_by_key(|(c, n)| (std::cmp::Reverse(n * n * c.value()), c.value()));
    held.into_iter().take(max).map(|(c, _)| c).collect()
}

/// The commodities in `wanted` this hand can offer: ones it holds but is not
/// collecting itself.
pub fn cards_to_offer(
    cards: &PlayerTradeCards,
    wanted: &[TradeCard],
    own_wants: &[TradeCard],
) -> Vec<TradeCard> {
    let held = cards.commodity_cards();
    wanted
        .iter()
        .copied()
        .filter(|c| held.get(c).copied().unwrap_or(0) > 0 && !own_wants.contains(c))
        .collect()
}

// ── Systems ────────────────────────────────────────────────────────────────

use crate::civilization::concepts::trade::trade_components::{CanTrade, OpenTradeOffer};
use crate::civilization::concepts::trade::trade_resources::TradePhaseState;
use crate::stupid_ai::{IsHuman, Personality, offer_creation_chance};
use bevy::prelude::{Commands, Local, Name, Query, Res, Time, With, Without, debug};
use rand::RngExt;

/// Talk kept on the table; older statements are swept away.
const MAX_TALK: usize = 80;

/// A proposal its maker can no longer pay for is off the table: the named
/// cards are gone (traded in another deal) or too few cards are left. So is
/// one whose addressee has left the table.
pub fn withdraw_unfulfillable_proposals(
    mut offers: Query<&mut OpenTradeOffer>,
    hands: Query<&PlayerTradeCards>,
    at_table: Query<(), With<CanTrade>>,
) {
    for mut offer in &mut offers {
        if offer.withdrawn || offer.accepted_by.is_some() {
            continue;
        }
        let can_pay = hands.get(offer.creator).is_ok_and(|hand| {
            offer
                .offering_guaranteed
                .iter()
                .all(|(card, n)| hand.number_of_cards_for_trade_card(*card) >= *n)
                && hand.number_of_tradeable_cards() >= offer.total_offering()
        });
        let addressee_there = offer.target.is_some_and(|t| at_table.contains(t));
        if !can_pay || !addressee_there {
            offer.withdrawn = true;
        }
    }
}

/// Withdrawn proposals nobody took leave the table.
pub fn clear_withdrawn_offers(mut commands: Commands, offers: Query<(Entity, &OpenTradeOffer)>) {
    for (entity, offer) in &offers {
        if offer.withdrawn && offer.accepted_by.is_none() {
            commands.entity(entity).despawn();
        }
    }
}

/// Self-play reads every statement from the log as a `[TALK]` line.
pub fn log_new_talk(talk: Query<&TradeTalk, bevy::prelude::Added<TradeTalk>>) {
    for t in &talk {
        bevy::prelude::info!("[TALK] {}", t.line());
    }
}

/// Keep the table readable: only the latest `MAX_TALK` statements stay.
pub fn prune_old_talk(mut commands: Commands, talk: Query<(Entity, &TradeTalk)>) {
    let count = talk.iter().len();
    if count <= MAX_TALK {
        return;
    }
    let mut by_age: Vec<(Entity, u32)> = talk.iter().map(|(e, t)| (e, t.said_at)).collect();
    // The countdown runs down, so the oldest were said at the highest values.
    by_age.sort_by_key(|(_, at)| std::cmp::Reverse(*at));
    for (entity, _) in by_age.into_iter().take(count - MAX_TALK) {
        commands.entity(entity).despawn();
    }
}

/// AIs talk once a second: each says what it wants (again, when its hand
/// changed), and some answer one Want they can serve with what they have and
/// what they want back. Eager traders answer more often.
pub fn ai_trade_talk(
    mut commands: Commands,
    ai_players: Query<
        (Entity, &Name, &PlayerTradeCards, &Personality),
        (With<CanTrade>, Without<IsHuman>),
    >,
    traders: Query<(), With<CanTrade>>,
    mut talk: Query<(Entity, &mut TradeTalk)>,
    trade_phase_state: Res<TradePhaseState>,
    time: Res<Time>,
    mut timer: Local<f32>,
    mut answered: Local<HashSet<(Entity, Entity)>>,
) {
    *timer += time.delta_secs();
    if *timer < 1.0 {
        return;
    }
    *timer = 0.0;
    let now = trade_phase_state.countdown_seconds;
    // A new phase starts at the full countdown: forget last phase's answers.
    if talk.is_empty() {
        answered.clear();
    }
    let mut rng = rand::rng();
    let live_wants: Vec<(Entity, Entity, String, Vec<TradeCard>)> = talk
        .iter()
        .filter_map(|(e, t)| match &t.kind {
            TalkKind::Want(cards) if t.live && traders.contains(t.speaker) => {
                Some((e, t.speaker, t.speaker_name.clone(), cards.clone()))
            }
            _ => None,
        })
        .collect();

    for (me, name, hand, personality) in &ai_players {
        let wants = cards_to_want(hand, 2);
        let current = live_wants.iter().find(|(_, speaker, _, _)| *speaker == me);
        if !wants.is_empty() && current.is_none_or(|(_, _, _, cards)| *cards != wants) {
            for (_, mut t) in &mut talk {
                if t.speaker == me && matches!(t.kind, TalkKind::Want(_)) {
                    t.live = false;
                }
            }
            debug!("{name} wants {wants:?}");
            commands
                .spawn(TradeTalk::new(me, name.to_string(), TalkKind::Want(wants.clone())).at(now));
        }

        if rng.random::<f32>() > offer_creation_chance(&personality.weights) {
            continue;
        }
        let answer = live_wants
            .iter()
            .find_map(|(want_talk, speaker, speaker_name, cards)| {
                if *speaker == me || answered.contains(&(me, *want_talk)) {
                    return None;
                }
                let have = cards_to_offer(hand, cards, &wants);
                (!have.is_empty()).then(|| (*want_talk, *speaker, speaker_name.clone(), have))
            });
        if let Some((want_talk, speaker, speaker_name, have)) = answer {
            answered.insert((me, want_talk));
            commands.spawn(
                TradeTalk::new(
                    me,
                    name.to_string(),
                    TalkKind::Reply {
                        have,
                        want: wants.clone(),
                    },
                )
                .to(speaker, speaker_name)
                .in_thread(Some(want_talk))
                .at(now),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(n: u32) -> Entity {
        Entity::from_raw_u32(n).unwrap()
    }

    #[test]
    fn talk_reads_like_a_table() {
        let want = TradeTalk::new(e(1), "Egypt", TalkKind::Want(vec![TradeCard::Iron]));
        assert_eq!(want.line(), "Egypt: I want Iron!");
        let reply = TradeTalk::new(
            e(2),
            "Babylon",
            TalkKind::Reply {
                have: vec![TradeCard::Iron],
                want: vec![TradeCard::Grain, TradeCard::Wine, TradeCard::Salt],
            },
        )
        .to(e(1), "Egypt");
        assert_eq!(
            reply.line(),
            "Babylon -> Egypt: I have Iron; got any Grain, Wine or Salt?"
        );
    }

    #[test]
    fn the_partner_who_wants_what_we_give_and_has_what_we_want_comes_first() {
        let (me, b, c, d) = (e(1), e(2), e(3), e(4));
        let talk = [
            // B: "I have Iron, want Grain" to me.
            TradeTalk::new(
                b,
                "B",
                TalkKind::Reply {
                    have: vec![TradeCard::Iron],
                    want: vec![TradeCard::Grain],
                },
            )
            .to(me, "Me"),
            // C only wants Grain.
            TradeTalk::new(c, "C", TalkKind::Want(vec![TradeCard::Grain])),
        ];
        let known =
            TableKnowledge::from_talk(talk.iter().enumerate().map(|(i, t)| (e(10 + i as u32), t)));
        let give = HashMap::from([(TradeCard::Grain, 2)]);
        let want = HashMap::from([(TradeCard::Iron, 2)]);
        let score = |p| known.partner_score(me, p, &give, &want);
        assert_eq!(score(b), 5.0);
        assert_eq!(score(c), 2.0);
        assert_eq!(score(d), 0.0);
        assert_eq!(known.want_talk.get(&c), Some(&e(11)));
    }

    mod proposals {
        use super::super::*;
        use bevy::prelude::{App, IntoScheduleConfigs, Update};

        fn e(n: u32) -> Entity {
            Entity::from_raw_u32(n).unwrap()
        }

        #[test]
        fn only_the_addressee_can_accept() {
            let (a, b, c) = (e(1), e(2), e(3));
            let mut offer = OpenTradeOffer::new(a, "A", Some(b), Some("B".into()));
            assert!(offer.can_accept(b));
            assert!(
                !offer.can_accept(c),
                "a third player must counter, not take it"
            );
            assert!(!offer.can_accept(a));
            offer.target = None;
            assert!(!offer.can_accept(b), "nobody takes an unaddressed deal");
        }

        fn hand(cards: &[(TradeCard, usize)]) -> PlayerTradeCards {
            let mut hand = PlayerTradeCards::default();
            for (card, n) in cards {
                hand.add_trade_cards(*card, *n);
            }
            hand
        }

        fn proposal(from: Entity, to: Entity) -> OpenTradeOffer {
            let mut offer = OpenTradeOffer::new(from, "A", Some(to), Some("B".into()));
            offer.offering_guaranteed = HashMap::from([(TradeCard::Iron, 2)]);
            offer.offering_hidden_count = 1;
            offer.wanting_guaranteed = HashMap::from([(TradeCard::Salt, 2)]);
            offer.wanting_hidden_count = 1;
            offer
        }

        #[test]
        fn a_proposal_its_maker_cannot_pay_or_nobody_can_take_leaves_the_table() {
            let mut app = App::new();
            app.add_systems(
                Update,
                (withdraw_unfulfillable_proposals, clear_withdrawn_offers).chain(),
            );
            let world = app.world_mut();
            let rich = world
                .spawn((
                    hand(&[(TradeCard::Iron, 2), (TradeCard::Ochre, 1)]),
                    CanTrade,
                ))
                .id();
            let poor = world
                .spawn((
                    hand(&[(TradeCard::Iron, 1), (TradeCard::Ochre, 2)]),
                    CanTrade,
                ))
                .id();
            let gone = world.spawn(hand(&[(TradeCard::Salt, 3)])).id();
            let other = world.spawn((hand(&[(TradeCard::Salt, 3)]), CanTrade)).id();
            let good = world.spawn(proposal(rich, other)).id();
            let unpaid = world.spawn(proposal(poor, other)).id();
            let to_nobody = world.spawn(proposal(rich, gone)).id();
            app.update();
            app.update();
            let world = app.world();
            assert!(world.get_entity(good).is_ok());
            assert!(world.get_entity(unpaid).is_err(), "named cards it lacks");
            assert!(world.get_entity(to_nobody).is_err(), "addressee left");
        }
    }
}
