use bevy::prelude::*;
use crate::civilization::concepts::trade_cards::components::{PlayerTradeCards, TradeCard};
use crate::civilization::concepts::trade_cards::enums::{TradeCardType, Commodity, Calamity};
use crate::stupid_ai::prelude::IsHuman;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_trade_ui)
            .add_systems(Update, update_trade_ui);
    }
}

#[derive(Component)]
struct TradeCardsUI;

#[derive(Component)]
struct TradeCardsGrid;

fn setup_trade_ui(mut commands: Commands) {
    // Create the root node for the trade cards UI
    commands
        .spawn((
            Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(10.0),
                    bottom: Val::Px(10.0),
                    width: Val::Px(600.0),
                    height: Val::Px(300.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
            },
            BackgroundColor(Color::rgba(0.1, 0.1, 0.1, 0.8)),
            TradeCardsUI,
        ))
        .with_children(|parent| {
            // Title for the trade cards UI
            parent.spawn(
                TextBundle::from_section(
                    "Your Trade Cards",
                    TextStyle {
                        font_size: 24.0,
                        color: Color::WHITE,
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                }),
            );

            // Container for the grid of cards
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    align_items: AlignItems::FlexStart,
                    justify_content: JustifyContent::FlexStart,
                    column_gap: Size::new(Val::Px(10.0)),
                    row_gap: Size::new(Val::Px(10.0)),
                    overflow: Overflow::clip_y(),
                    ..default()
                },
                TradeCardsGrid,
            ));
        });
}

fn update_trade_ui(
    mut commands: Commands,
    human_query: Query<(Entity, &PlayerTradeCards), With<IsHuman>>,
    card_grid_query: Query<Entity, With<TradeCardsGrid>>,
) {
    // Find the human player and their trade cards
    if let Ok((human_entity, player_trade_cards)) = human_query.get_single() {
        // Find the trade cards grid
        if let Ok(grid_entity) = card_grid_query.get_single() {
            // Clear existing cards
            if let Some(mut grid) = commands.get_entity(grid_entity) {
                grid.despawn_descendants();
                
                // Get all trade cards
                let cards = player_trade_cards.trade_cards();
                
                // Create a card for each trade card
                for card in cards {
                    let (card_type_text, card_color) = match card.card_type {
                        TradeCardType::CommodityCard(commodity) => {
                            let commodity_name = format!("{:?}", commodity);
                            (commodity_name, Color::rgb(0.2, 0.6, 0.2))
                        }
                        TradeCardType::CalamityCard(calamity) => {
                            let calamity_name = format!("{:?}", calamity);
                            (calamity_name, Color::rgb(0.7, 0.2, 0.2))
                        }
                    };
                    
                    grid.with_children(|parent| {
                        parent.spawn((
                            Node {
                                width: Val::Px(100.0),
                                height: Val::Px(120.0),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                padding: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                            BackgroundColor(card_color),
                        ))
                        .with_children(|card_parent| {
                            // Card type (Commodity or Calamity name)
                            card_parent.spawn(
                                TextBundle::from_section(
                                    card_type_text,
                                    TextStyle {
                                        font_size: 16.0,
                                        color: Color::WHITE,
                                        ..default()
                                    },
                                )
                                .with_style(Style {
                                    margin: UiRect::bottom(Val::Px(5.0)),
                                    ..default()
                                }),
                            );
                            
                            // Card value
                            card_parent.spawn(
                                TextBundle::from_section(
                                    format!("Value: {}", card.value),
                                    TextStyle {
                                        font_size: 14.0,
                                        color: Color::WHITE,
                                        ..default()
                                    },
                                ),
                            );
                            
                            // Tradeable status
                            card_parent.spawn(
                                TextBundle::from_section(
                                    if card.tradeable { "Tradeable" } else { "Not Tradeable" },
                                    TextStyle {
                                        font_size: 12.0,
                                        color: if card.tradeable { Color::rgb(0.2, 0.8, 0.2) } else { Color::rgb(0.8, 0.2, 0.2) },
                                        ..default()
                                    },
                                ),
                            );
                        });
                    });
                }
            }
        }
    }
}
