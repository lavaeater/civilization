use bevy::prelude::{Component, Reflect, ReflectComponent};

/// How many commodity cards a player's hand exceeds by, over
/// [`crate::civilization::concepts::shed_commodity_cards::COMMODITY_CARD_HAND_LIMIT`].
/// Attached at the start of `GameActivity::ShedCommodityCards`; removed once the
/// player has discarded down to the limit (human: after UI confirmation; AI:
/// immediately).
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct NeedsToShedCommodityCards {
    pub must_discard: usize,
}

/// Marks the human player as currently choosing which commodity cards to
/// discard; removed by the shed-selection UI's Confirm button.
#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct AwaitingCommodityShedSelection;
