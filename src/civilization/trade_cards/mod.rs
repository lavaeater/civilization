pub mod trade_card_plugin;
pub mod trade_card_components;
pub mod trade_card_systems;
mod trade_card_setup_systems;

pub mod prelude {
    pub use crate::civilization::trade_cards::trade_card_plugin::TradeCardPlugin;
    pub use crate::civilization::trade_cards::trade_card_components::*;
    pub use crate::civilization::trade_cards::trade_card_systems::*;
}
