use crate::civilization::save_game::{LoadGameRequest, SaveGameRequest};
use crate::civilization::GameCamera;
use crate::loading::TextureAssets;
use crate::{GamePaused, GameState};
use bevy::{
    color::palettes,
    feathers::{
        controls::{
            button, checkbox, color_plane, color_slider, color_swatch, radio, slider,
            toggle_switch, ButtonProps, ButtonVariant, ColorChannel, ColorPlane, ColorPlaneValue,
            ColorSlider, ColorSliderProps, ColorSwatch, ColorSwatchValue, SliderBaseColor,
            SliderProps,
        },
        cursor::{EntityCursor, OverrideCursor},
        dark_theme::create_dark_theme,
        rounded_corners::RoundedCorners,
        theme::{ThemeBackgroundColor, ThemedText, UiTheme},
        tokens, FeathersPlugins,
    },
    input_focus::tab_navigation::TabGroup,
    prelude::*,
    ui::{Checked, InteractionDisabled},
    ui_widgets::{
        checkbox_self_update, observe, slider_self_update, Activate, RadioButton, RadioGroup,
        SliderPrecision, SliderStep, SliderValue, ValueChange,
    },
    window::SystemCursorIcon,
};


#[derive(Resource)]
struct DemoWidgetStates {
    rgb_color: Srgba,
    hsl_color: Hsla,
}

#[derive(Component, Clone, Copy, PartialEq)]
enum SwatchType {
    Rgb,
    Hsl,
}

#[derive(Component, Clone, Copy)]
struct DemoDisabledButton;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FeathersPlugins)
            .insert_resource(UiTheme(create_dark_theme()))
            .insert_resource(DemoWidgetStates {
                rgb_color: palettes::tailwind::EMERALD_800.with_alpha(0.7),
                hsl_color: palettes::tailwind::AMBER_800.into(),
            })
            .add_systems(OnEnter(GameState::Menu), setup_menu)
            .add_systems(
                Update,
                handle_menu_buttons.run_if(in_state(GameState::Menu)),
            )
            .add_systems(OnExit(GameState::Menu), cleanup_menu)
            .add_systems(
                Update,
                (
                    toggle_pause.run_if(in_state(GameState::Playing)),
                    handle_pause_buttons.run_if(resource_exists::<GamePaused>),
                ),
            );
    }
}

// ============================================================================
// Shared components
// ============================================================================

#[derive(Component)]
struct ChangeState(GameState);

#[derive(Component)]
struct LoadGameButton;

#[derive(Component)]
struct SaveGameButton;

#[derive(Component)]
struct ResumeButton;

#[derive(Component)]
struct MainMenuButton;

#[derive(Component, Default)]
struct Menu;

#[derive(Component, Default)]
struct PauseMenu;

// ============================================================================
// Main Menu
// ============================================================================

fn demo_root() -> impl Bundle {
    (
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Start,
            justify_content: JustifyContent::Start,
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            ..default()
        },
        TabGroup::default(),
        ThemeBackgroundColor(tokens::WINDOW_BG),
        children![(
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                justify_content: JustifyContent::Start,
                padding: UiRect::all(px(8)),
                row_gap: px(8),
                width: percent(30),
                min_width: px(200),
                ..default()
            },
            children![
                (
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Start,
                        column_gap: px(8),
                        ..default()
                    },
                    children![
                        (
                            button(
                                ButtonProps::default(),
                                (),
                                Spawn((Text::new("Normal"), ThemedText))
                            ),
                            observe(|_activate: On<Activate>| {
                                info!("Normal button clicked!");
                            })
                        ),
                        (
                            button(
                                ButtonProps::default(),
                                (InteractionDisabled, DemoDisabledButton),
                                Spawn((Text::new("Disabled"), ThemedText))
                            ),
                            observe(|_activate: On<Activate>| {
                                info!("Disabled button clicked!");
                            })
                        ),
                        (
                            button(
                                ButtonProps {
                                    variant: ButtonVariant::Primary,
                                    ..default()
                                },
                                (),
                                Spawn((Text::new("Primary"), ThemedText))
                            ),
                            observe(|_activate: On<Activate>| {
                                info!("Disabled button clicked!");
                            })
                        ),
                    ]
                ),
                (
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Start,
                        column_gap: px(1),
                        ..default()
                    },
                    children![
                        (
                            button(
                                ButtonProps {
                                    corners: RoundedCorners::Left,
                                    ..default()
                                },
                                (),
                                Spawn((Text::new("Left"), ThemedText))
                            ),
                            observe(|_activate: On<Activate>| {
                                info!("Left button clicked!");
                            })
                        ),
                        (
                            button(
                                ButtonProps {
                                    corners: RoundedCorners::None,
                                    ..default()
                                },
                                (),
                                Spawn((Text::new("Center"), ThemedText))
                            ),
                            observe(|_activate: On<Activate>| {
                                info!("Center button clicked!");
                            })
                        ),
                        (
                            button(
                                ButtonProps {
                                    variant: ButtonVariant::Primary,
                                    corners: RoundedCorners::Right,
                                },
                                (),
                                Spawn((Text::new("Right"), ThemedText))
                            ),
                            observe(|_activate: On<Activate>| {
                                info!("Right button clicked!");
                            })
                        ),
                    ]
                ),
                (
                    button(
                        ButtonProps::default(),
                        (),
                        Spawn((Text::new("Toggle override"), ThemedText))
                    ),
                    observe(|_activate: On<Activate>, mut ovr: ResMut<OverrideCursor>| {
                        ovr.0 = if ovr.0.is_some() {
                            None
                        } else {
                            Some(EntityCursor::System(SystemCursorIcon::Wait))
                        };
                        info!("Override cursor button clicked!");
                    })
                ),
                (
                    checkbox(Checked, Spawn((Text::new("Checkbox"), ThemedText))),
                    observe(
                        |change: On<ValueChange<bool>>,
                         query: Query<Entity, With<DemoDisabledButton>>,
                         mut commands: Commands| {
                            info!("Checkbox clicked!");
                            let mut button = commands.entity(query.single().unwrap());
                            if change.value {
                                button.insert(InteractionDisabled);
                            } else {
                                button.remove::<InteractionDisabled>();
                            }
                            let mut checkbox = commands.entity(change.source);
                            if change.value {
                                checkbox.insert(Checked);
                            } else {
                                checkbox.remove::<Checked>();
                            }
                        }
                    )
                ),
                (
                    checkbox(
                        InteractionDisabled,
                        Spawn((Text::new("Disabled"), ThemedText))
                    ),
                    observe(|_change: On<ValueChange<bool>>| {
                        warn!("Disabled checkbox clicked!");
                    })
                ),
                (
                    checkbox(
                        (InteractionDisabled, Checked),
                        Spawn((Text::new("Disabled+Checked"), ThemedText))
                    ),
                    observe(|_change: On<ValueChange<bool>>| {
                        warn!("Disabled checkbox clicked!");
                    })
                ),
                (
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        row_gap: px(4),
                        ..default()
                    },
                    RadioGroup,
                    observe(
                        |value_change: On<ValueChange<Entity>>,
                         q_radio: Query<Entity, With<RadioButton>>,
                         mut commands: Commands| {
                            for radio in q_radio.iter() {
                                if radio == value_change.value {
                                    commands.entity(radio).insert(Checked);
                                } else {
                                    commands.entity(radio).remove::<Checked>();
                                }
                            }
                        }
                    ),
                    children![
                        radio(Checked, Spawn((Text::new("One"), ThemedText))),
                        radio((), Spawn((Text::new("Two"), ThemedText))),
                        radio((), Spawn((Text::new("Three"), ThemedText))),
                        radio(
                            InteractionDisabled,
                            Spawn((Text::new("Disabled"), ThemedText))
                        ),
                    ]
                ),
                (
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Start,
                        column_gap: px(8),
                        ..default()
                    },
                    children![
                        (toggle_switch((),), observe(checkbox_self_update)),
                        (
                            toggle_switch(InteractionDisabled,),
                            observe(checkbox_self_update)
                        ),
                        (
                            toggle_switch((InteractionDisabled, Checked),),
                            observe(checkbox_self_update)
                        ),
                    ]
                ),
                (
                    slider(
                        SliderProps {
                            max: 100.0,
                            value: 20.0,
                            ..default()
                        },
                        (SliderStep(10.), SliderPrecision(2)),
                    ),
                    observe(slider_self_update)
                ),
                (
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    children![Text("Srgba".to_owned()), color_swatch(SwatchType::Rgb),]
                ),
                (
                    color_plane(ColorPlane::RedBlue, ()),
                    observe(
                        |change: On<ValueChange<Vec2>>, mut color: ResMut<DemoWidgetStates>| {
                            color.rgb_color.red = change.value.x;
                            color.rgb_color.blue = change.value.y;
                        }
                    )
                ),
                (
                    color_slider(
                        ColorSliderProps {
                            value: 0.5,
                            channel: ColorChannel::Red
                        },
                        ()
                    ),
                    observe(
                        |change: On<ValueChange<f32>>, mut color: ResMut<DemoWidgetStates>| {
                            color.rgb_color.red = change.value;
                        }
                    )
                ),
                (
                    color_slider(
                        ColorSliderProps {
                            value: 0.5,
                            channel: ColorChannel::Green
                        },
                        ()
                    ),
                    observe(
                        |change: On<ValueChange<f32>>, mut color: ResMut<DemoWidgetStates>| {
                            color.rgb_color.green = change.value;
                        },
                    )
                ),
                (
                    color_slider(
                        ColorSliderProps {
                            value: 0.5,
                            channel: ColorChannel::Blue
                        },
                        ()
                    ),
                    observe(
                        |change: On<ValueChange<f32>>, mut color: ResMut<DemoWidgetStates>| {
                            color.rgb_color.blue = change.value;
                        },
                    )
                ),
                (
                    color_slider(
                        ColorSliderProps {
                            value: 0.5,
                            channel: ColorChannel::Alpha
                        },
                        ()
                    ),
                    observe(
                        |change: On<ValueChange<f32>>, mut color: ResMut<DemoWidgetStates>| {
                            color.rgb_color.alpha = change.value;
                        },
                    )
                ),
                (
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    children![Text("Hsl".to_owned()), color_swatch(SwatchType::Hsl),]
                ),
                (
                    color_slider(
                        ColorSliderProps {
                            value: 0.5,
                            channel: ColorChannel::HslHue
                        },
                        ()
                    ),
                    observe(
                        |change: On<ValueChange<f32>>, mut color: ResMut<DemoWidgetStates>| {
                            color.hsl_color.hue = change.value;
                        },
                    )
                ),
                (
                    color_slider(
                        ColorSliderProps {
                            value: 0.5,
                            channel: ColorChannel::HslSaturation
                        },
                        ()
                    ),
                    observe(
                        |change: On<ValueChange<f32>>, mut color: ResMut<DemoWidgetStates>| {
                            color.hsl_color.saturation = change.value;
                        },
                    )
                ),
                (
                    color_slider(
                        ColorSliderProps {
                            value: 0.5,
                            channel: ColorChannel::HslLightness
                        },
                        ()
                    ),
                    observe(
                        |change: On<ValueChange<f32>>, mut color: ResMut<DemoWidgetStates>| {
                            color.hsl_color.lightness = change.value;
                        },
                    )
                )
            ]
        ),],
    )
}

//
// fn root() -> impl Bundle {
//     (
//         Node {
//             width: percent(50.0),
//             height: percent(50.0),
//             align_items: AlignItems::Start,
//             justify_content: JustifyContent::Start,
//             display: Display::Flex,
//             flex_direction: FlexDirection::Row,
//             row_gap: px(10.0),
//             ..default()
//         },
//         ThemeBackgroundColor(tokens::WINDOW_BG),
//         children![(
//                 button(
//                     ButtonProps {
//                     variant: ButtonVariant::Primary,
//                     ..default()
//                 },
//                     (),
//                     Spawn((Text::new("Play"), ThemedText))
//                 ),
//                 observe(|_activate: On<Activate>, mut next_state: ResMut<NextState<GameState>>| {
//                     next_state.set(GameState::Playing);
//                 })
//             ),
//             (
//                 button(
//                     ButtonProps::default(),
//                     (),
//                     Spawn((Text::new("Load Game"), ThemedText))
//                 ),
//                 observe(|_activate: On<Activate>, mut load_writer: MessageWriter<LoadGameRequest>| {
//                     load_writer.write(LoadGameRequest);
//                 })
//             )],
//     )
// }

fn setup_menu(mut commands: Commands, _textures: Res<TextureAssets>, theme: Res<UiTheme>) {
    commands.spawn((
        Camera2d,
        IsDefaultUiCamera,
        Projection::Orthographic(OrthographicProjection::default_2d()),
        GameCamera,
        Msaa::Off,
    ));

    commands.spawn(demo_root());

    // let mut ui = UIBuilder::new(commands, Some(theme.clone()));
    //
    // ui.component::<Menu>()
    //     .size_percent(100.0, 100.0)
    //     .display_flex()
    //     .flex_column()
    //     .align_items_center()
    //     .justify_center()
    //     .gap_px(16.0);
    //
    // ui.add_text_child("Advanced Civilization", None, Some(48.0), None);
    //
    // ui.add_themed_button(ChangeState(GameState::Playing), |btn| {
    //     btn.text("Play").size_px(300.0, 60.0);
    // });
    //
    // ui.add_themed_button(ChangeState(GameState::Sandbox), |btn| {
    //     btn.text("Sandbox").size_px(300.0, 60.0);
    // });
    //
    // ui.add_themed_button(LoadGameButton, |btn| {
    //     btn.text("Load Game").size_px(300.0, 60.0);
    // });
    //
    // ui.build();
}

fn handle_menu_buttons(
    mut next_state: ResMut<NextState<GameState>>,
    change_query: Query<(&Interaction, &ChangeState), (Changed<Interaction>, With<Button>)>,
    load_query: Query<&Interaction, (Changed<Interaction>, With<LoadGameButton>, With<Button>)>,
    mut load_writer: MessageWriter<LoadGameRequest>,
) {
    for (interaction, change_state) in &change_query {
        if *interaction == Interaction::Pressed {
            next_state.set(change_state.0.clone());
        }
    }
    for interaction in &load_query {
        if *interaction == Interaction::Pressed {
            load_writer.write(LoadGameRequest);
        }
    }
}

fn cleanup_menu(mut commands: Commands, menu: Query<Entity, With<Menu>>) {
    for entity in menu.iter() {
        commands.entity(entity).despawn();
    }
}

// ============================================================================
// Pause Menu (overlay — GameState stays Playing)
// ============================================================================

fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    paused: Option<Res<GamePaused>>,
    theme: Res<UiTheme>,
    pause_menu: Query<Entity, With<PauseMenu>>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }

    if paused.is_some() {
        // Unpause: remove resource and despawn overlay
        commands.remove_resource::<GamePaused>();
        for entity in pause_menu.iter() {
            commands.entity(entity).despawn();
        }
    } else {
        // Pause: insert resource and spawn overlay
        commands.insert_resource(GamePaused);
        spawn_pause_menu(commands, &theme);
    }
}

fn spawn_pause_menu(commands: Commands, theme: &UiTheme) {
    // let mut ui = UIBuilder::new(commands, None);
    // ui.component::<PauseMenu>()
    //     .size_percent(100.0, 100.0)
    //     .display_flex()
    //     .flex_column()
    //     .align_items_center()
    //     .justify_center()
    //     .bg_color(Color::srgba(0.0, 0.0, 0.0, 0.7))
    //     .gap_px(16.0);
    //
    // ui.add_text_child("Paused", None, Some(48.0), None);
    //
    // ui.add_themed_button(ResumeButton, |btn| {
    //     btn.text("Resume").size_px(300.0, 60.0);
    // });
    //
    // ui.add_themed_button(SaveGameButton, |btn| {
    //     btn.text("Save Game").size_px(300.0, 60.0);
    // });
    //
    // ui.add_themed_button(LoadGameButton, |btn| {
    //     btn.text("Load Game").size_px(300.0, 60.0);
    // });
    //
    // ui.add_themed_button(MainMenuButton, |btn| {
    //     btn.text("Main Menu").size_px(300.0, 60.0);
    // });
    //
    // ui.build();
}

fn handle_pause_buttons(
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    resume_query: Query<&Interaction, (Changed<Interaction>, With<ResumeButton>, With<Button>)>,
    save_query: Query<&Interaction, (Changed<Interaction>, With<SaveGameButton>, With<Button>)>,
    load_query: Query<&Interaction, (Changed<Interaction>, With<LoadGameButton>, With<Button>)>,
    main_menu_query: Query<
        &Interaction,
        (Changed<Interaction>, With<MainMenuButton>, With<Button>),
    >,
    pause_menu: Query<Entity, With<PauseMenu>>,
    mut save_writer: MessageWriter<SaveGameRequest>,
    mut load_writer: MessageWriter<LoadGameRequest>,
) {
    for interaction in &resume_query {
        if *interaction == Interaction::Pressed {
            commands.remove_resource::<GamePaused>();
            for entity in pause_menu.iter() {
                commands.entity(entity).despawn();
            }
            return;
        }
    }
    for interaction in &save_query {
        if *interaction == Interaction::Pressed {
            save_writer.write(SaveGameRequest);
        }
    }
    for interaction in &load_query {
        if *interaction == Interaction::Pressed {
            load_writer.write(LoadGameRequest);
        }
    }
    for interaction in &main_menu_query {
        if *interaction == Interaction::Pressed {
            commands.remove_resource::<GamePaused>();
            for entity in pause_menu.iter() {
                commands.entity(entity).despawn();
            }
            next_state.set(GameState::Menu);
        }
    }
}
