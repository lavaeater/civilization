use crate::civilization::concepts::trade_cards::components::{PlayerTradeCards, TradeCard};
use crate::civilization::concepts::trade_cards::enums::TradeCardType;
use crate::stupid_ai::prelude::IsHuman;
use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_trade_ui)
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
                position_type: PositionType::Relative,
                left: Val::Px(10.0),
                top: Val::Px(10.0),
                width: Val::Px(300.0),
                height: Val::Px(800.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Default,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
            TradeCardsUI,
        ))
        .with_children(|parent| {
            // Title for the trade cards UI
            parent.spawn((
                Text::new("Your Trade Cards"),
                TextFont::from_font_size(24.0),
                TextColor(Color::WHITE),
            ));

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
                    column_gap: Val::Px(10.0),
                    row_gap: Val::Px(10.0),
                    overflow: Overflow::clip_y(),
                    ..default()
                },
                TradeCardsGrid,
            ));
        });
}

fn update_trade_ui(
    mut commands: Commands,
    human_query: Query<(Entity, &PlayerTradeCards), (With<IsHuman>, Changed<PlayerTradeCards>)>,
    card_grid_query: Query<Entity, With<TradeCardsGrid>>,
) {
    // Find the human player and their trade cards
    if let Ok((_human_entity, player_trade_cards)) = human_query.get_single() {
        debug!("Human player Trade Cards have changed, update the UI!");
        // Find the trade cards grid
        if let Ok(grid_entity) = card_grid_query.get_single() {
            // Clear existing cards
            if let Some(mut grid) = commands.get_entity(grid_entity) {
                grid.despawn_descendants();

                // Get all trade cards
                let cards = player_trade_cards.trade_cards();

                // Create a card for each trade card
                for card in cards {
                    create_trade_card_node(&mut grid, card);
                }
            }
        }
    }
}

fn create_trade_card_node(grid: &mut EntityCommands, card: TradeCard) {
    let (card_type_text, card_color) = match card.card_type {
        TradeCardType::CommodityCard(commodity) => {
            let commodity_name = format!("{:?}", commodity);
            (commodity_name, Color::srgb(0.2, 0.6, 0.2))
        }
        TradeCardType::CalamityCard(calamity) => {
            let calamity_name = format!("{:?}", calamity);
            (calamity_name, Color::srgb(0.7, 0.2, 0.2))
        }
    };

    grid.with_children(|parent| {
        parent
            .spawn((
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
                card_parent.spawn((
                    Text::new(card_type_text),
                    TextFont::from_font_size(16.0),
                    TextColor(Color::WHITE),
                ));

                // Card value
                card_parent.spawn((
                    Text::new(format!("Value: {}", card.value)),
                    TextFont::from_font_size(14.0),
                    TextColor(Color::WHITE),
                ));

                // Tradeable status
                card_parent.spawn((
                    Text::new(if card.tradeable {
                        "Tradeable"
                    } else {
                        "Not Tradeable"
                    }),
                    TextFont::from_font_size(12.0),
                    TextColor(if card.tradeable {
                        Color::srgb(0.2, 0.8, 0.2)
                    } else {
                        Color::srgb(0.8, 0.2, 0.2)
                    }),
                ));
            });
    });
}
