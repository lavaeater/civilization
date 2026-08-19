//! Shared network protocol between the civilization game server and its clients.
//!
//! This crate owns the wire vocabulary: stable identifiers (entities are
//! world-local and never cross the network), the move mirror of the game's
//! `GameMove`, and the lightyear message/channel registration. It must stay
//! free of game-rule and UI dependencies — both `adv_civ` and `adv_civ_server`
//! depend on it, never the other way around.

mod civ_cards;
mod faction;
mod messages;
mod plugin;
mod trade_cards;

/// Identifies this game protocol to netcode; client and server must agree.
pub const PROTOCOL_ID: u64 = 0xC1_71_20_26;

pub use civ_cards::{CivCardName, CivCardType, Credits};
pub use faction::GameFaction;
pub use messages::*;
pub use plugin::{ControlChannel, ProtocolPlugin};
pub use trade_cards::{TradeCard, TradeCardTrait};
