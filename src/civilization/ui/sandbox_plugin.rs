use crate::civilization::ui::ui_builder::{
    ButtonPartial, NodePartial, UIBuilder, UiBuilderDefaults, BG_COLOR, BORDER_COLOR, TEXT_COLOR,
};
use crate::GameState;
use bevy::prelude::*;

pub struct SandboxPlugin;

impl Plugin for SandboxPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(UiBuilderDefaults::new())
            .init_resource::<SandboxLayoutState>()
            .add_systems(OnEnter(GameState::Sandbox), setup)
            .add_systems(
                Update,
                (handle_layout_controls, update_sample_box).run_if(in_state(GameState::Sandbox)),
            );
    }
}

#[derive(Component, Default)]
struct SandboxUiRoot;

#[derive(Component, Default)]
struct SampleBox;

#[derive(Component)]
struct LayoutValueDisplay {
    property: LayoutProperty,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LayoutProperty {
    Width,
    Height,
    PaddingAll,
    MarginAll,
    BorderWidth,
    BorderRadius,
}

#[derive(Component)]
struct LayoutControlButton {
    property: LayoutProperty,
    delta: f32,
}

#[derive(Resource)]
struct SandboxLayoutState {
    width: f32,
    height: f32,
    padding: f32,
    margin: f32,
    border_width: f32,
    border_radius: f32,
}

impl Default for SandboxLayoutState {
    fn default() -> Self {
        Self {
            width: 150.0,
            height: 100.0,
            padding: 16.0,
            margin: 8.0,
            border_width: 2.0,
            border_radius: 0.0,
        }
    }
}

const SAMPLE_BOX_COLOR: Color = Color::srgba(0.3, 0.5, 0.8, 0.9);
const CONTROL_PANEL_COLOR: Color = Color::srgba(0.2, 0.2, 0.2, 0.95);
const CONTROL_BTN_COLOR: Color = Color::srgba(0.4, 0.4, 0.4, 1.0);
const CONTROL_BTN_HOVER: Color = Color::srgba(0.5, 0.5, 0.5, 1.0);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ui_defaults: ResMut<UiBuilderDefaults>,
    layout_state: Res<SandboxLayoutState>,
) {
    // Spawn camera for UI rendering
    commands.spawn((
        Camera2d,
        IsDefaultUiCamera,
        Projection::Orthographic(OrthographicProjection::default_2d()),
        Msaa::Off,
    ));

    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    ui_defaults.base_font = font;
    ui_defaults.bg_color = BG_COLOR;
    ui_defaults.text_color = TEXT_COLOR;
    ui_defaults.font_size = 16.0;
    ui_defaults.border_color = BORDER_COLOR;
    ui_defaults.button_def = Some(ButtonPartial {
        border_radius: Some(BorderRadius::MAX),
        border_color: Some(BORDER_COLOR),
        ..default()
    });
    ui_defaults.node_def = Some(
        NodePartial::new()
            .border_all_px(2.0)
            .border_color(BORDER_COLOR)
            .border_radius_zero()
            .padding_all_px(8.0)
            .margin_all_px(6.0),
    );

    let mut ui = UIBuilder::new(commands, Some(ui_defaults.clone()));

    ui.with_component::<SandboxUiRoot>()
        .size_percent(98.0, 98.0)
        .display_flex()
        .flex_dir_row();

    // Left side: Sample box display area
    ui.add_panel(|ui| {
        ui.size_percent(50.0, 100.0)
            .padding_all_px(8.0)
            .display_flex()
            .justify_center()
            .align_items_center()
            .bg_color(Color::srgba(0.1, 0.1, 0.1, 0.5));

        // The sample box we'll modify
        ui.add_panel(|ui| {
            ui.with_component::<SampleBox>()
                .size_px(layout_state.width, layout_state.height)
                .padding_all_px(layout_state.padding)
                .margin_all_px(layout_state.margin)
                .border_all_px(layout_state.border_width, BORDER_COLOR)
                .bg_color(SAMPLE_BOX_COLOR)
                .display_flex()
                .justify_center()
                .align_items_center();

            ui
                .text_node("Sample Text");
        });
    });
    // Right side: Control panel
    ui.add_panel(|ui| {
        ui.width_auto()
            .height_percent(100.)
            .display_flex()
            .flex_dir_column()
            .bg_color(CONTROL_PANEL_COLOR)
            .padding_all_px(16.0)
            .row_gap_px(8.0);

        // Title
        ui.with_child(|ui| {
            ui.default_text("Layout Inspector").margin_btm_px(16.0);
        });

        // Control rows
        build_control_row(ui, "Width", LayoutProperty::Width, layout_state.width);
        build_control_row(ui, "Height", LayoutProperty::Height, layout_state.height);
        build_control_row(
            ui,
            "Padding",
            LayoutProperty::PaddingAll,
            layout_state.padding,
        );
        build_control_row(ui, "Margin", LayoutProperty::MarginAll, layout_state.margin);
        build_control_row(
            ui,
            "Border",
            LayoutProperty::BorderWidth,
            layout_state.border_width,
        );
        build_control_row(
            ui,
            "Radius",
            LayoutProperty::BorderRadius,
            layout_state.border_radius,
        );
    });

    let (_root, _commands) = ui.build();
}

fn build_control_row(
    ui: &mut UIBuilder,
    label: &str,
    property: LayoutProperty,
    initial_value: f32,
) {
    ui.add_row(|ui| {
        ui
            .align_items_center()
            .justify_space_between()
            .column_gap_px(4.0)
            .margin_zero()
            .padding_zero();

        // Label
        ui.text_with_width(label, 50.0);
        
        //Decrease button (-10)
        ui.with_child(|ui| {
            ui.add_button(
                "-10",
                40.0,
                28.0,
                CONTROL_BTN_COLOR,
                12.0,
                4.0,
                LayoutControlButton {
                    property,
                    delta: -10.0,
                },
            );
        });
        
        //Decrease button (-1)
        ui.with_child(|ui| {
            ui.add_button(
                "-1",
                40.0,
                28.0,
                CONTROL_BTN_COLOR,
                12.0,
                4.0,
                LayoutControlButton {
                    property,
                    delta: -1.0,
                },
            );
        });
        

        // Value display
        ui.build_text(format!("{:.0}", initial_value), |ui| {
            ui.width_px(40.)
                .align_items_center()
                .justify_center()
                .insert(LayoutValueDisplay { property });
        });
        
        // ui.with_child(|ui| {
        //     ui.default_text(format!("{:.0}", initial_value))
        //         .width_px(40.)
        //         .align_items_center()
        //         .justify_center()
        //         .insert(LayoutValueDisplay { property });
        // });

        // Increase button (+1)
        ui.with_child(|ui| {
            ui.add_button(
                "+1",
                40.0,
                28.0,
                CONTROL_BTN_COLOR,
                12.0,
                4.0,
                LayoutControlButton {
                    property,
                    delta: 1.0,
                },
            );
        });

        // Increase button (+10)
        ui.with_child(|ui| {
            ui.add_button(
                "+10",
                40.0,
                28.0,
                CONTROL_BTN_COLOR,
                12.0,
                4.0,
                LayoutControlButton {
                    property,
                    delta: 10.0,
                },
            );
        });
    });
}

fn handle_layout_controls(
    mut interaction_query: Query<
        (&Interaction, &LayoutControlButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut layout_state: ResMut<SandboxLayoutState>,
) {
    for (interaction, control, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                let value = match control.property {
                    LayoutProperty::Width => &mut layout_state.width,
                    LayoutProperty::Height => &mut layout_state.height,
                    LayoutProperty::PaddingAll => &mut layout_state.padding,
                    LayoutProperty::MarginAll => &mut layout_state.margin,
                    LayoutProperty::BorderWidth => &mut layout_state.border_width,
                    LayoutProperty::BorderRadius => &mut layout_state.border_radius,
                };
                *value = (*value + control.delta).max(0.0);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(CONTROL_BTN_HOVER);
            }
            Interaction::None => {
                *bg_color = BackgroundColor(CONTROL_BTN_COLOR);
            }
        }
    }
}

fn update_sample_box(
    layout_state: Res<SandboxLayoutState>,
    mut sample_box_query: Query<&mut Node, With<SampleBox>>,
    mut value_displays: Query<(&LayoutValueDisplay, &mut Text)>,
) {
    if !layout_state.is_changed() {
        return;
    }

    // Update sample box
    for mut node in sample_box_query.iter_mut() {
        node.width = Val::Px(layout_state.width);
        node.height = Val::Px(layout_state.height);
        node.padding = UiRect::all(Val::Px(layout_state.padding));
        node.margin = UiRect::all(Val::Px(layout_state.margin));
        node.border = UiRect::all(Val::Px(layout_state.border_width));
        node.border_radius = BorderRadius::all(Val::Px(layout_state.border_radius));
    }

    // Update value displays
    for (display, mut text) in value_displays.iter_mut() {
        let value = match display.property {
            LayoutProperty::Width => layout_state.width,
            LayoutProperty::Height => layout_state.height,
            LayoutProperty::PaddingAll => layout_state.padding,
            LayoutProperty::MarginAll => layout_state.margin,
            LayoutProperty::BorderWidth => layout_state.border_width,
            LayoutProperty::BorderRadius => layout_state.border_radius,
        };
        text.0 = format!("{:.0}", value);
    }
}
