use crate::{CivCardName, GameFaction, TradeCard};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Stable identifier for a board area: the `GameArea.id` value, which both
/// server and client know from the map definition.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct AreaId(pub i32);

impl Display for AreaId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "area#{}", self.0)
    }
}

/// Server-assigned opaque id for a transient entity (e.g. a trade offer).
/// Only ever echoed back by clients, never constructed by them.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct NetOfferId(pub u64);

/// Network mirror of the game's `GameMove`, with stable ids instead of
/// `Entity` values: areas by `GameArea.id`, players by faction, cards by
/// their enums. These are for *display* — submitting a move only echoes the
/// server's move index, so no client→server id translation ever happens.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum NetGameMove {
    PopulationExpansion {
        area: AreaId,
        max_tokens: usize,
    },
    Movement {
        source: AreaId,
        target: AreaId,
        max_tokens: usize,
    },
    /// Sea passage hop via a ship.
    ShipFerry {
        source: AreaId,
        target: AreaId,
        max_tokens: usize,
    },
    AttackArea {
        source: AreaId,
        target: AreaId,
        max_tokens: usize,
    },
    AttackCity {
        source: AreaId,
        target: AreaId,
        max_tokens: usize,
    },
    EndMovement,
    BuildCity {
        area: AreaId,
    },
    EndCityConstruction,
    /// Reduce a city back to population (city support shortfall).
    EliminateCity {
        area: AreaId,
        tokens_gained: usize,
        tokens_needed: usize,
    },
    Trade(NetTradeMove),
    AcquireCivCards {
        cards: Vec<CivCardName>,
    },
    DoneAcquiringCivCards,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum NetTradeMove {
    /// Open a trade negotiation with another player. `matching_cards` is the
    /// server's hint of card types both hands share (counts are the *other*
    /// player's, capped — never leak exact hands).
    ProposeTrade {
        to: GameFaction,
        matching_cards: Vec<(TradeCard, usize)>,
    },
    AcceptOrDeclineTrade {
        offer: NetOfferId,
    },
    AutoDeclineTrade {
        offer: NetOfferId,
    },
    StopTrading,
    SettleTrade {
        offer: NetOfferId,
    },
}

/// Network mirror of `GameActivity`. The server reports phase transitions so
/// clients can drive their UI flow without running any phase systems locally.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum NetPhase {
    PrepareGame,
    StartGame,
    CollectTaxes,
    PopulationExpansion,
    Census,
    ShipConstruction,
    Movement,
    Conflict,
    CityConstruction,
    RemoveSurplusPopulation,
    CheckCitySupportAfterRemoveSurplusPopulation,
    AcquireTradeCards,
    Trade,
    ResolveCalamities,
    CheckCitySupportAfterResolveCalamities,
    AcquireCivilizationCards,
    ShedCommodityCards,
    MoveSuccessionMarkers,
    GameOver,
}

// ---------------------------------------------------------------------------
// Client → Server
// ---------------------------------------------------------------------------

/// Sent once after connecting: claim a seat in the lobby.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct JoinGame {
    pub player_name: String,
    /// Reconnect secret from a previous join, if the client has one (see
    /// `docs/multiplayer.md`'s session-token section). `None` for a client's
    /// first-ever join, or the dev manual-auth path, which doesn't track one.
    #[serde(default)]
    pub reconnect_token: Option<String>,
}

/// Pick one of the moves the server offered in [`YourMoves`]. The index is
/// echoed back; optional fields parameterize moves that need player choices.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SubmitMove {
    pub move_index: usize,
    /// Token count for expansion/movement moves; `None` = sensible maximum.
    pub tokens: Option<usize>,
    /// Payment cards for civ-card purchases.
    pub payment: Vec<(TradeCard, usize)>,
    /// Accept (true) or decline (false) an incoming trade offer.
    pub accept: Option<bool>,
}

impl SubmitMove {
    pub fn index(move_index: usize) -> Self {
        SubmitMove {
            move_index,
            tokens: None,
            payment: Vec::new(),
            accept: None,
        }
    }
}

/// Open a trade offer (rule 28.3: at least 2 truthfully-named guaranteed
/// cards per side, plus a hidden count that may be anything at settlement).
/// Mirrors the agent API's `POST /trade/offer` and the local human UI's
/// "publish offer" flow against the one real trade model, `OpenTradeOffer` —
/// not the dead `NetTradeMove`/`TradeMove` pair below, see
/// `agent-api-design.md`. A dedicated message rather than a `SubmitMove`
/// because the guaranteed card selections are free-form input, not a pick
/// from an enumerated move list.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ProposeTradeOffer {
    pub offering_guaranteed: Vec<(TradeCard, usize)>,
    pub offering_hidden_count: usize,
    pub wanting_guaranteed: Vec<(TradeCard, usize)>,
    pub wanting_hidden_count: usize,
    /// `None` = open to anyone.
    pub target: Option<GameFaction>,
}

/// Accept an open offer (rule 28.1's "open negotiation" — any eligible
/// player, not just an explicit target, may accept an untargeted offer).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AcceptTradeOffer {
    pub offer: NetOfferId,
}

/// Commit the real cards for your side of an accepted offer (the hidden
/// slots may be anything, honest bluff included — rule 28.3).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SettleTradeOffer {
    pub offer: NetOfferId,
    pub cards: Vec<(TradeCard, usize)>,
}

// ---------------------------------------------------------------------------
// Server → Client
// ---------------------------------------------------------------------------

/// Reply to [`JoinGame`]: which seat (faction) the client got.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct JoinAccepted {
    pub player_name: String,
    pub faction: GameFaction,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LobbyPlayer {
    pub name: String,
    pub faction: GameFaction,
    pub connected: bool,
}

/// Broadcast whenever lobby composition changes.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LobbyState {
    pub players: Vec<LobbyPlayer>,
    pub seats_total: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PhaseChanged {
    pub phase: NetPhase,
}

/// The moves available to *this* client's player right now. Indices are the
/// server's `AvailableMoves` keys and must be echoed back in [`SubmitMove`].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct YourMoves {
    pub moves: Vec<(usize, NetGameMove)>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MoveRejected {
    pub move_index: usize,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AreaView {
    pub area: AreaId,
    pub name: String,
    pub max_population: usize,
    /// Tokens per faction currently in the area.
    pub population: Vec<(GameFaction, usize)>,
    /// Faction owning a built city here, if any.
    pub city: Option<GameFaction>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PlayerView {
    pub name: String,
    pub faction: GameFaction,
    pub tokens_in_stock: usize,
    /// Owned civilization cards — public information per the rules.
    pub civ_cards: Vec<CivCardName>,
    /// Hand *size* is public; the cards themselves go via [`YourHand`].
    pub trade_card_count: usize,
}

/// A player's own trade-card hand. Hidden information: composed per client
/// and only ever sent to the owning seat, never broadcast.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct YourHand {
    pub cards: Vec<(TradeCard, usize)>,
}

/// Public board state, composed per broadcast. Hidden information (trade
/// cards in hand, …) must never go in here — it gets its own per-client
/// messages when those phases come online.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GameStateView {
    pub areas: Vec<AreaView>,
    pub players: Vec<PlayerView>,
}

/// One open trade offer, from anyone's negotiation. Public per rule 28.1
/// ("open negotiation") — but hidden slots stay counts-only here, same as
/// the agent API's `GET /trade`; actual hidden card identities are never
/// broadcast, only revealed to the two parties at settlement.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NetTradeOffer {
    pub id: NetOfferId,
    pub creator: GameFaction,
    pub target: Option<GameFaction>,
    pub accepted_by: Option<GameFaction>,
    /// True once `accepted_by` is set and both sides still need to commit
    /// real cards — the settlement step.
    pub settling: bool,
    pub offering_guaranteed: Vec<(TradeCard, usize)>,
    pub offering_hidden_count: usize,
    pub wanting_guaranteed: Vec<(TradeCard, usize)>,
    pub wanting_hidden_count: usize,
}

/// Every currently-open trade offer, broadcast whenever any of them changes
/// (created/accepted/settled/withdrawn) — mirrors `GET /trade`'s shape in
/// the agent API.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TradeOffersView {
    pub offers: Vec<NetTradeOffer>,
}
