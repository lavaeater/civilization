use crate::civilization::concepts::trade_cards::components::{PlayerTradeCards, TradeCard};
use crate::civilization::concepts::trade_cards::events::HumanPlayerPulledTradeCard;
use crate::stupid_ai::prelude::IsHuman;
use belly::build::*;
use bevy::prelude::OnAdd;
use bevy::prelude::*;

pub struct BellyUIPlugin;

impl Plugin for BellyUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BellyPlugin)
            .add_systems(Startup, setup)
            .add_systems(Update, handle_cards_added);
    }
}

fn handle_cards_added(
    mut elements: Elements,
    mut commands: Commands,
    mut reader: EventReader<HumanPlayerPulledTradeCard>,
    player_trade_cards: Query<&PlayerTradeCards>
) {
    for event in reader.read() {
        if let Some(trade_card_span) = elements.select("#trade_cards").entities().first() {
            commands.entity(*trade_card_span).despawn_descendants();
            elements.add_child(*trade_card_span, eml! {
                <span c:row>
                    <span c:column>
                        <span>"{event.1.name}"</span>
                        <span>"{event.1.description}"</span>
                    </span>
                </span>
            });

            elements.toggle_class(entity, event.0.into())
        }
    }
}

fn setup(mut commands: Commands) {
    commands.queue(StyleSheet::load("ui/styles.ess"));
    commands.queue(eml! {
        <body>
            <div id="trade_cards_container" c:column>
                <span c:row>"Your Trade Cards"</span>
            <div id="trade_cards" c:row></div>
            </div>
        </body>
    });
}
