use crate::civilization::concepts::trade_cards::components::{PlayerTradeCards, TradeCard};
use crate::civilization::concepts::trade_cards::events::HumanPlayerPulledTradeCard;
use crate::stupid_ai::prelude::IsHuman;
use belly::build::*;
use belly::widgets::common::DivWidget;
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
    player_trade_cards: Query<&PlayerTradeCards>,
) {
    for event in reader.read() {
        if let Some(trade_card_span) = elements.select("#trade_cards").entities().first() {
            commands.entity(*trade_card_span).despawn_descendants();


            if let Ok(player_cards) = player_trade_cards
               .get(event.player_entity) {
                
                let trade_card_iter = player_cards.trade_cards_owned();
                    elements.add_child(
                        *trade_card_span,
                        eml! {
                       <span>
                       <for value_group in=trade_card_iter>
                       <span c:row>
                           <span c:column>
                               <span>"{event.1.name}"</span>
                               <span>"{event.1.description}"</span>
                           </span>
                       </span>
                       </for>
                       </span>
                   },
                    );
            }
        }
    }
}

fn setup(mut commands: Commands) {
    commands.queue(StyleSheet::load("ui/styles.ess"));
    commands.queue(eml! {
        <body>
            <span id="trade_cards_container" c:column>
                <span c:row>"Your Trade Cards"</span>
            <span id="trade_cards" c:row></span>
            </span>
        </body>
    });
}

#[widget]
#[extends(DivWidget)]
fn Animal(ctx: &mut WidgetContext, ch: &mut GroupedCards) {
    let seed = ctx.param("seed".into()).unwrap().take().unwrap();
    ch.randomize(seed);
    let this = ctx.entity();
    let color = ctx.spawn();
    ctx.commands()
        .queue(from!(this, AnimalState: color) >> to!(color, BackgroundColor:0));
    ctx.render(eml! {
        <button>
            <span {color} c:animal s:background-color=managed()>
                <img modulate="#ffffff00" bind:src=from!(this, AnimalState:avatar.image())/>
                <span c:label>
                    <label bind:value=from!(this, AnimalState:name)/>
                </span>
            </span>
        </button>
    })
}

#[derive(Component, Default, Clone)]
/// The AnimalState acts like a model. Changing this model properties
/// affects widgets binded to this model (the background of `Animal`
/// widget is changed as well as background of editor when you edit the
/// animal).
pub struct GroupedCards {
    name: String,
    avatar: Avatar,
    color: Color,
}
