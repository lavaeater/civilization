// Civilization-card vocabulary lives in the network protocol crate so that
// client and server share one definition; re-exported to keep existing paths.
pub use adv_civ_protocol::{CivCardName, CivCardType, Credits};
