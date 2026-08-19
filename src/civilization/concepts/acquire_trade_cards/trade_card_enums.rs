// TradeCard and its trait live in the network protocol crate so that client
// and server share one definition; re-exported here to keep existing paths.
pub use adv_civ_protocol::{TradeCard, TradeCardTrait};
