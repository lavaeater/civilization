use crate::civilization::concepts::trade_cards::components::{PlayerTradeCards, TradeCard};
use crate::civilization::concepts::trade_cards::enums::{Calamity, Commodity, TradeCardType};
use crate::stupid_ai::prelude::IsHuman;
use crate::GameState;
use bevy::remote::http::RemoteHttpPlugin;
use bevy::remote::RemotePlugin;
use bevy::{input::mouse::MouseWheel, prelude::*};
use bevy_hui::prelude::*;
use maud::*;

pub struct CobWebUiPlugin;

impl Plugin for CobWebUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RemotePlugin::default(),
            RemoteHttpPlugin::default(),
            HuiPlugin,
            HuiAutoLoadPlugin::new(&["ui/components"]),
        ))
        .add_systems(OnEnter(GameState::Playing), setup)
        .add_systems(
            Update,
            (
                update_collapse.run_if(in_state(GameState::Playing)),
                update_scroll.run_if(in_state(GameState::Playing)),
                cleaner.run_if(in_state(GameState::Playing)),
                update_inventory.run_if(in_state(GameState::Playing)),
            ),
        );
    }
}

fn setup(
    mut cmd: Commands,
    server: Res<AssetServer>,
    mut html_funcs: HtmlFunctions,
    mut html_comps: HtmlComponents,
) {
    cmd.spawn((
        HtmlNode(server.load("ui/demo/menu.html")),
        TemplateProperties::default().with("title", "Trading Cards"),
    ));

    // register function bindings
    html_funcs.register("inventory", init_inventory);
    html_funcs.register("scrollable", init_scrollable);
    html_funcs.register("collapse", |In(entity), mut cmd: Commands| {
        cmd.entity(entity).insert(Collapse(true));
    });

    // register custom node by passing a template handle
    html_comps.register_with_spawn_fn("panel", server.load("ui/demo/panel.html"), |mut cmd| {
        cmd.insert(Name::new("Panel"));
    });

    // a function that updates a property and triggers a recompile
    html_funcs.register(
        "debug",
        |In(entity),
         mut cmd: Commands,
         mut template_props: Query<&mut TemplateProperties>,
         scopes: Query<&TemplateScope>| {
            let Ok(scope) = scopes.get(entity) else {
                return;
            };

            let Ok(mut props) = template_props.get_mut(**scope) else {
                return;
            };

            let rng = rand::random::<u32>();
            props.insert("title".to_string(), format!("{}", rng));
            cmd.trigger_targets(CompileContextEvent, **scope);
        },
    );
}

#[derive(Component, Deref, DerefMut, Default)]
pub struct Collapse(pub bool);

fn update_collapse(
    mut interactions: Query<(&Interaction, &UiTarget, &mut Collapse), Changed<Interaction>>,
    mut style: Query<&mut HtmlStyle>,
) {
    interactions
        .iter_mut()
        .for_each(|(interaction, target, mut collapse)| {
            let Interaction::Pressed = interaction else {
                return;
            };

            let display = match **collapse {
                true => {
                    **collapse = false;
                    Display::None
                }
                false => {
                    **collapse = true;
                    Display::Flex
                }
            };

            if let Ok(mut style) = style.get_mut(**target) {
                style.computed.node.display = display;
            }
        });
}

#[derive(Component)]
pub struct Scrollable {
    offset: f32,
    speed: f32,
}

fn init_scrollable(In(entity): In<Entity>, mut cmd: Commands, tags: Query<&Tags>) {
    let speed = tags
        .get(entity)
        .ok()
        .and_then(|tags| {
            tags.get("scroll_speed")
                .and_then(|fstr| fstr.parse::<f32>().ok())
        })
        .unwrap_or(10.);

    cmd.entity(entity).insert(Scrollable { speed, offset: 0. });
}

fn update_scroll(
    mut events: EventReader<MouseWheel>,
    mut scrollables: Query<(&mut Scrollable, &UiTarget, &Interaction)>,
    mut targets: Query<&mut HtmlStyle>,
    time: Res<Time>,
) {
    // whatever
    events.read().for_each(|ev| {
        scrollables.iter_mut().for_each(|(mut scroll, target, _)| {
            let Ok(mut style) = targets.get_mut(**target) else {
                return;
            };

            scroll.offset = scroll.offset + ev.y.signum() * scroll.speed * time.delta_secs();
            style.computed.node.top = Val::Px(scroll.offset);
        });
    });
}

fn init_inventory(In(entity): In<Entity>, mut cmd: Commands, server: Res<AssetServer>, human_query: Query<(&PlayerTradeCards, Entity), With<IsHuman>>) {
    cmd.entity(entity).insert(InventoryContainer);
    
    if let Ok((player_cards, player_entity)) = human_query.get_single() {
        cmd.entity(entity).with_children(|cmd| {
            for card in player_cards.trade_cards() {
                spawn_trade_card(cmd, &server, &card);
            }
        });
    }
}

#[derive(Component)]
pub struct InventoryContainer;

/// Spawn a single trade card UI element
fn spawn_trade_card(cmd: &mut ChildBuilder, server: &AssetServer, card: &TradeCard) {
    let (title, border_color) = match &card.card_type {
        TradeCardType::CommodityCard(commodity) => {
            let color = match commodity {
                Commodity::Ochre => "#CD7F32",  // Bronze-like
                Commodity::Hides => "#8B4513",  // Brown
                Commodity::Iron => "#708090",   // Slate gray
                Commodity::Papyrus => "#F5DEB3", // Wheat
                Commodity::Salt => "#FFFFFF",    // White
                Commodity::Timber => "#8B4513",  // Brown
                Commodity::Grain => "#FFD700",   // Gold
                Commodity::Oil => "#000000",     // Black
                Commodity::Cloth => "#E6E6FA",   // Lavender
                Commodity::Wine => "#800020",    // Burgundy
                Commodity::Bronze => "#CD7F32",  // Bronze
                Commodity::Silver => "#C0C0C0",  // Silver
                Commodity::Spices => "#FF4500",  // Orange Red
                Commodity::Resin => "#DAA520",   // Golden Rod
                Commodity::Gems => "#4B0082",    // Indigo
                Commodity::Dye => "#9370DB",     // Medium Purple
                Commodity::Gold => "#FFD700",    // Gold
                Commodity::Ivory => "#FFFFF0",   // Ivory
            };
            (format!("{:?} ({})", commodity, card.value), color)
        },
        TradeCardType::CalamityCard(calamity) => {
            (format!("{:?}", calamity), "#FF0000") // Red for calamities
        },
    };

    debug!("Spawning trade card: {:?}", title);
    
    cmd.spawn((
        HtmlNode(server.load("ui/demo/card.html")),
        TemplateProperties::default()
            .with("title", &title)
            .with("primary", border_color),
        TradeCardUI {
            card_type: card.card_type.clone(),
            value: card.value,
            tradeable: card.tradeable,
        },
    ));
}

#[derive(Component, Clone)]
pub struct TradeCardUI {
    pub card_type: TradeCardType,
    pub value: usize,
    pub tradeable: bool,
}


#[derive(Component, Deref, DerefMut)]
struct LifeTime(Timer);
impl LifeTime {
    pub fn new(s: f32) -> Self {
        LifeTime(Timer::new(
            std::time::Duration::from_secs_f32(s),
            TimerMode::Once,
        ))
    }
}

fn cleaner(mut expired: Query<(Entity, &mut LifeTime)>, mut cmd: Commands, time: Res<Time>) {
    expired.iter_mut().for_each(|(entity, mut lifetime)| {
        if lifetime.tick(time.delta()).finished() {
            cmd.entity(entity).despawn_recursive();
        }
    });
}

/// System to update the inventory UI when player trade cards change
fn update_inventory(
    mut commands: Commands,
    server: Res<AssetServer>,
    human_query: Query<(&PlayerTradeCards, Entity), (With<IsHuman>, Changed<PlayerTradeCards>)>,
    inventory_container_query: Query<(Entity, &Children), With<InventoryContainer>>,
    card_ui_query: Query<(Entity, &TradeCardUI)>,
) {
    // Only proceed if the human player's trade cards have changed
    if let Ok((player_cards, _)) = human_query.get_single() {
        debug!("Player trade cards have changed");
        if let Ok((container_entity, children)) = inventory_container_query.get_single() {
            // Get current cards in UI
            let mut current_ui_cards = Vec::new();
            for &child in children.iter() {
                if let Ok((entity, card_ui)) = card_ui_query.get(child) {
                    current_ui_cards.push((entity, card_ui.clone()));
                }
            }
            
            // Get player's actual cards
            let player_trade_cards = player_cards.trade_cards();
            
            // Remove cards that are no longer in the player's inventory
            for (entity, card_ui) in current_ui_cards.iter() {
                let card_exists = player_trade_cards.iter().any(|player_card| {
                    player_card.card_type == card_ui.card_type && 
                    player_card.value == card_ui.value && 
                    player_card.tradeable == card_ui.tradeable
                });
                
                if !card_exists {
                    commands.entity(*entity).despawn_recursive();
                }
            }
            
            // Add new cards that aren't in the UI yet
            for player_card in player_trade_cards.iter() {
                let card_exists = current_ui_cards.iter().any(|(_, card_ui)| {
                    player_card.card_type == card_ui.card_type && 
                    player_card.value == card_ui.value && 
                    player_card.tradeable == card_ui.tradeable
                });
                
                if !card_exists {
                    commands.entity(container_entity).with_children(|cmd| {
                        spawn_trade_card(cmd, &server, player_card);
                    });
                }
            }
        }
    }
}


fn setup_maude(mut cmd: Commands, mut templates: ResMut<Assets<HtmlTemplate>>) {
    cmd.spawn(Camera2d);

    let html = greet_button("Maud").render();

    let template = match parse_template::<VerboseHtmlError>(html.0.as_bytes()) {
        Ok((_, template)) => template,
        Err(err) => {
            let e = err.map(|e| e.format(html.0.as_bytes(), "maud"));
            dbg!(e);
            return;
        }
    };

    let handle = templates.add(template);
    cmd.spawn(HtmlNode(handle));
}

fn greet_button(name: &str) -> Markup {
    html!(
        template {
            node background="#000" padding="50px" border_radius="20px"
            {
                button background="#333" padding="10px" border_radius="10px" {
                    text font_size="32" {"Hello "(name)"!"}
                }
            }
        }
    )
}
