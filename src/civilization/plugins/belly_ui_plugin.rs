use crate::civilization::concepts::trade_cards::components::{PlayerTradeCards, TradeCard};
use crate::stupid_ai::prelude::{IsHuman};
use belly::build::*;
use bevy::prelude::OnAdd;
use bevy::prelude::*;

struct BellyUIPlugin;

impl Plugin for BellyUIPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(on_add_is_human)
            .add_plugins(BellyPlugin)
            .add_systems(Startup, setup)
        ;
    }
}

fn on_add_is_human(
    trigger: Trigger<OnAdd, IsHuman>,
    mut commands: Commands
) {
    
}


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BellyPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.queue(StyleSheet::load("ui/styles.ess"));
    commands.queue(eml! {
        <body>
            <span id="trade_cards" c:column>
                <span c:row>"Your Trade Cards"</span>
            </span>
        </body>
    });
}

