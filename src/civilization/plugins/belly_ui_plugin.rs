use crate::civilization::concepts::trade_cards::components::{PlayerTradeCards, TradeCard};
use crate::civilization::concepts::trade_cards::events::HumanPlayerPulledTradeCard;
use crate::stupid_ai::prelude::IsHuman;
use belly::build::*;
use belly::widgets::common::DivWidget;
use bevy::prelude::OnAdd;
use bevy::prelude::*;
use bevy::render::render_resource::encase::private::RuntimeSizedArray;
use itertools::Itertools;

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
                
                
                
                let trade_card_iter: Vec<CardValueGroup> = player_cards
                    .trade_cards_grouped_by_value_and_type().iter().map(|(value, by_type)|{
                        CardValueGroup::new(format!("Value: {value}"), *value, by_type.iter().map(|(card_type, cards)| CardsInfo::new(card_type.to_string(), cards)).collect_vec())
                    }).collect();
                
                
                
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
fn card_group(ctx: &mut WidgetContext, card_value_group_state: &mut CardValueGroupState) {
    let this = ctx.entity();
    let color = ctx.spawn();
    let card_value_groups = card_value_group_state.card_value_groups.clone();
    ctx.render(eml! {
        <for value_group in=card_value_groups>
            <span c:trade-card>
                <img modulate="#ffffff00" bind:src=from!(this, AnimalState:avatar.image())/>
                <span c:label>
                    <label bind:value=from!(this, AnimalState:name)/>
                </span>
            </span>
        </for>
    })
}

#[derive(Component, Default, Clone)]
pub struct CardValueGroupState {
    pub card_value_groups: Vec<CardValueGroup>
}

impl From<CardValueGroupState> for Variant {
    fn from(value: CardValueGroupState) -> Self {
        Variant::boxed(value)
    }
}

#[derive(Default, Clone)]
pub struct CardValueGroup {
    name: String,
    value: usize,
    cards: Vec<CardsInfo>
}

impl From<CardValueGroup> for Variant {
    fn from(value: CardValueGroup) -> Self {
        Variant::boxed(value)
    }
}

impl CardValueGroup {
    pub fn new(name: String, value: usize, cards: Vec<CardsInfo>) -> Self {
        CardValueGroup {
            name,
            value,
            cards
        }
    }
}

#[derive(Default, Clone)]
pub struct CardsInfo {
    name: String,
    tradeable: bool,
    value: usize,
    calamity: bool,
    commodity: bool,
    color: Color,
    count: usize
}

impl From<CardsInfo> for Variant {
    fn from(value: CardsInfo) -> Self {
        Variant::boxed(value)
    }
}

impl CardsInfo {
    pub fn new(name: String, cards: &Vec<TradeCard>) -> Self {
        if let Some(card) = cards.first() {
            CardsInfo {
                name,
                tradeable: card.tradeable,
                value: card.value,
                calamity: card.is_calamity(),
                commodity: card.is_commmodity(),
                color: card.color.clone(),
                count: cards.len()
            }
        } else {
            CardsInfo {
                name,
                ..default()
            }
        }
        
    }
}