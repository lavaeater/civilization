use bevy::color::palettes::basic::{BLACK, WHITE};
use bevy::ecs::system::IntoObserverSystem;
use bevy::prelude::*;
use bevy::reflect::Enum;
use bevy::text::{Justify, LineBreak, TextLayout};
use std::collections::VecDeque;

// Feathers imports - re-exported at bottom of file for external use
use bevy::feathers::controls::{button, ButtonProps, ButtonVariant};
use bevy::feathers::rounded_corners::RoundedCorners;
use bevy::feathers::theme::ThemedText;
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::Activate;

#[derive(Bundle)]
pub struct NodeBundle {
    pub node: Node,
    pub background_color: BackgroundColor,
    pub border_color: BorderColor,
}

#[derive(Bundle)]
pub struct ButtonBundle {
    pub button: Button,
    pub background_color: BackgroundColor,
    pub border_color: BorderColor,
}

#[derive(Bundle)]
pub struct TextBundle {
    pub text: Text,
    pub text_font: TextFont,
    pub text_color: TextColor,
}

pub const BG_COLOR: Color = Color::srgba(0.5, 0.5, 0.5, 0.25);
pub const CARD_COLOR: Color = Color::srgba(0.7, 0.6, 0.2, 0.8);
pub const TEXT_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 1.0);
pub const BORDER_COLOR: Color = Color::srgba(0.2, 0.2, 0.2, 0.25);

#[derive(Component, Default)]
pub struct ButtonAction<T: Enum> {
    pub action: T,
}

/// Fluent UI Builder for creating Bevy UI elements
pub struct UIBuilder<'w, 's> {
    commands: Commands<'w, 's>,
    defaults: UiBuilderDefaults,
    current_entity: Entity,
    parent_stack: VecDeque<Entity>,
}

/// Builder for button-specific properties, wraps UIBuilder
pub struct ButtonBuilder<'a, 'w, 's> {
    ui: &'a mut UIBuilder<'w, 's>,
    text_entity: Option<Entity>,
}

#[derive(Default, Clone, Debug)]
pub struct ButtonPartial {
    pub text: Option<String>,
    pub font: Option<Handle<Font>>,
    pub width: Option<Val>,
    pub height: Option<Val>,
    pub border: Option<UiRect>,
    pub justify_content: Option<JustifyContent>,
    pub align_items: Option<AlignItems>,
    pub border_color: Option<Color>,
    pub border_radius: Option<BorderRadius>,
    pub bg_color: Option<Color>,
    pub font_size: Option<f32>,
    pub text_color: Option<Color>,
}

#[derive(Clone, Debug)]
pub struct ButtonDef {
    pub text: String,
    pub font: Handle<Font>,
    pub width: Val,
    pub height: Val,
    pub border: UiRect,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub border_color: Color,
    pub border_radius: BorderRadius,
    pub bg_color: Color,
    pub font_size: f32,
    pub text_color: Color,
}

impl Default for ButtonDef {
    fn default() -> Self {
        Self {
            text: "".to_string(),
            font: Default::default(),
            width: Val::Px(150.0),
            height: Val::Px(75.0),
            border: UiRect::all(Val::Px(5.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_color: Color::from(BLACK),
            border_radius: BorderRadius::ZERO,
            bg_color: Color::default(),
            font_size: default(),
            text_color: Color::from(WHITE),
        }
    }
}

impl ButtonDef {
    pub fn get_node(&self) -> Node {
        Node {
            width: self.width,
            height: self.height,
            border: self.border,
            justify_content: self.justify_content,
            align_items: self.align_items,
            ..default()
        }
    }
    pub fn get_background_color(&self) -> BackgroundColor {
        BackgroundColor(self.bg_color)
    }

    pub fn get_border_radius(&self) -> BorderRadius {
        self.border_radius
    }

    pub fn get_border_color(&self) -> BorderColor {
        BorderColor::all(self.border_color)
    }

    pub fn get_text(&self) -> Text {
        Text(self.text.clone())
    }

    pub fn get_text_font(&self) -> TextFont {
        TextFont::default()
            .with_font(self.font.clone())
            .with_font_size(self.font_size)
    }

    pub fn get_text_color(&self) -> TextColor {
        TextColor::from(self.text_color)
    }

    pub fn get_button_bundle(&self) -> ButtonBundle {
        ButtonBundle {
            button: Button,
            background_color: self.get_background_color(),
            border_color: self.get_border_color(),
        }
    }

    pub fn get_text_bundle(&self) -> TextBundle {
        TextBundle {
            text: self.get_text(),
            text_font: self.get_text_font(),
            text_color: self.get_text_color(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct NodeDef {
    pub display: Display,
    pub position_type: PositionType,
    pub overflow: Overflow,
    pub overflow_clip_margin: OverflowClipMargin,
    pub left: Val,
    pub right: Val,
    pub top: Val,
    pub bottom: Val,
    pub width: Val,
    pub height: Val,
    pub min_width: Val,
    pub min_height: Val,
    pub max_width: Val,
    pub max_height: Val,
    pub aspect_ratio: Option<f32>,
    pub align_items: AlignItems,
    pub justify_items: JustifyItems,
    pub align_self: AlignSelf,
    pub justify_self: JustifySelf,
    pub align_content: AlignContent,
    pub justify_content: JustifyContent,
    pub margin: UiRect,
    pub padding: UiRect,
    pub border: UiRect,
    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: Val,
    pub row_gap: Val,
    pub column_gap: Val,
    pub grid_auto_flow: GridAutoFlow,
    pub grid_template_rows: Vec<RepeatedGridTrack>,
    pub grid_template_columns: Vec<RepeatedGridTrack>,
    pub grid_auto_rows: Vec<GridTrack>,
    pub grid_auto_columns: Vec<GridTrack>,
    pub grid_row: GridPlacement,
    pub grid_column: GridPlacement,
    pub bg_color: Color,
    pub border_color: Color,
    pub border_radius: BorderRadius,
    pub box_sizing: BoxSizing,
    pub scrollbar_width: f32,
}

impl Default for NodeDef {
    fn default() -> Self {
        Self {
            display: Display::DEFAULT,
            position_type: PositionType::DEFAULT,
            overflow: Overflow::DEFAULT,
            overflow_clip_margin: OverflowClipMargin::DEFAULT,
            left: Val::DEFAULT,
            right: Val::DEFAULT,
            top: Val::DEFAULT,
            bottom: Val::DEFAULT,
            width: Val::DEFAULT,
            height: Val::DEFAULT,
            min_width: Val::DEFAULT,
            min_height: Val::DEFAULT,
            max_width: Val::DEFAULT,
            max_height: Val::DEFAULT,
            aspect_ratio: None,
            align_items: AlignItems::DEFAULT,
            justify_items: JustifyItems::DEFAULT,
            align_self: AlignSelf::DEFAULT,
            justify_self: JustifySelf::DEFAULT,
            align_content: AlignContent::DEFAULT,
            justify_content: JustifyContent::DEFAULT,
            margin: UiRect::DEFAULT,
            padding: UiRect::DEFAULT,
            border: UiRect::DEFAULT,
            flex_direction: FlexDirection::DEFAULT,
            flex_wrap: FlexWrap::DEFAULT,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: Val::DEFAULT,
            row_gap: Val::DEFAULT,
            column_gap: Val::DEFAULT,
            grid_auto_flow: GridAutoFlow::DEFAULT,
            grid_template_rows: vec![],
            grid_template_columns: vec![],
            grid_auto_rows: vec![],
            grid_auto_columns: vec![],
            grid_row: GridPlacement::DEFAULT,
            grid_column: GridPlacement::DEFAULT,
            bg_color: BG_COLOR,
            border_color: BORDER_COLOR,
            border_radius: BorderRadius::DEFAULT,
            box_sizing: BoxSizing::DEFAULT,
            scrollbar_width: 15.0,
        }
    }
}

impl NodeDef {
    pub fn get_node(&self) -> Node {
        Node {
            display: self.display,
            position_type: self.position_type,
            overflow: self.overflow,
            overflow_clip_margin: self.overflow_clip_margin,
            left: self.left,
            right: self.right,
            top: self.top,
            bottom: self.bottom,
            width: self.width,
            height: self.height,
            min_width: self.min_width,
            min_height: self.min_height,
            max_width: self.max_width,
            max_height: self.max_height,
            aspect_ratio: self.aspect_ratio,
            align_items: self.align_items,
            justify_items: self.justify_items,
            align_self: self.align_self,
            justify_self: self.justify_self,
            align_content: self.align_content,
            justify_content: self.justify_content,
            margin: self.margin,
            padding: self.padding,
            border: self.border,
            border_radius: self.border_radius,
            flex_direction: self.flex_direction,
            flex_wrap: self.flex_wrap,
            flex_grow: self.flex_grow,
            flex_shrink: self.flex_shrink,
            flex_basis: self.flex_basis,
            row_gap: self.row_gap,
            column_gap: self.column_gap,
            grid_auto_flow: self.grid_auto_flow,
            grid_template_rows: self.grid_template_rows.clone(),
            grid_template_columns: self.grid_template_columns.clone(),
            grid_auto_rows: self.grid_auto_rows.clone(),
            grid_auto_columns: self.grid_auto_columns.clone(),
            grid_row: self.grid_row,
            grid_column: self.grid_column,
            box_sizing: self.box_sizing,
            scrollbar_width: self.scrollbar_width,
        }
    }
    pub fn get_background_color(&self) -> BackgroundColor {
        BackgroundColor(self.bg_color)
    }

    pub fn get_border_radius(&self) -> BorderRadius {
        self.border_radius
    }

    pub fn get_border_color(&self) -> BorderColor {
        BorderColor::all(self.border_color)
    }

    pub fn get_node_bundle(&self) -> NodeBundle {
        NodeBundle {
            node: self.get_node(),
            background_color: self.get_background_color(),
            border_color: self.get_border_color(),
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct NodePartial {
    pub display: Option<Display>,
    pub position_type: Option<PositionType>,
    pub overflow: Option<Overflow>,

    pub overflow_clip_margin: Option<OverflowClipMargin>,

    pub left: Option<Val>,

    pub right: Option<Val>,

    pub top: Option<Val>,

    pub bottom: Option<Val>,

    pub width: Option<Val>,

    pub height: Option<Val>,

    pub min_width: Option<Val>,

    pub min_height: Option<Val>,

    pub max_width: Option<Val>,

    pub max_height: Option<Val>,

    pub aspect_ratio: Option<f32>,

    pub align_items: Option<AlignItems>,

    pub justify_items: Option<JustifyItems>,

    pub align_self: Option<AlignSelf>,

    pub justify_self: Option<JustifySelf>,

    pub align_content: Option<AlignContent>,

    pub justify_content: Option<JustifyContent>,

    pub margin: Option<UiRect>,

    pub padding: Option<UiRect>,

    pub border: Option<UiRect>,

    pub flex_direction: Option<FlexDirection>,

    pub flex_wrap: Option<FlexWrap>,

    pub flex_grow: Option<f32>,

    pub flex_shrink: Option<f32>,

    pub flex_basis: Option<Val>,

    pub row_gap: Option<Val>,

    pub column_gap: Option<Val>,

    pub grid_auto_flow: Option<GridAutoFlow>,

    pub grid_template_rows: Option<Vec<RepeatedGridTrack>>,

    pub grid_template_columns: Option<Vec<RepeatedGridTrack>>,

    pub grid_auto_rows: Option<Vec<GridTrack>>,
    pub grid_auto_columns: Option<Vec<GridTrack>>,

    pub grid_row: Option<GridPlacement>,

    pub grid_column: Option<GridPlacement>,
    pub border_radius: Option<BorderRadius>,
    pub border_color: Option<Color>,
    pub bg_color: Option<Color>,
}

impl NodePartial {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn display(mut self, value: Display) -> Self {
        self.display = Some(value);
        self
    }

    pub fn position_type(mut self, value: PositionType) -> Self {
        self.position_type = Some(value);
        self
    }

    pub fn overflow(mut self, value: Overflow) -> Self {
        self.overflow = Some(value);
        self
    }

    pub fn overflow_clip_margin(mut self, value: OverflowClipMargin) -> Self {
        self.overflow_clip_margin = Some(value);
        self
    }

    pub fn left(mut self, value: Val) -> Self {
        self.left = Some(value);
        self
    }

    pub fn right(mut self, value: Val) -> Self {
        self.right = Some(value);
        self
    }

    pub fn top(mut self, value: Val) -> Self {
        self.top = Some(value);
        self
    }

    pub fn bottom(mut self, value: Val) -> Self {
        self.bottom = Some(value);
        self
    }

    pub fn width(mut self, value: Val) -> Self {
        self.width = Some(value);
        self
    }

    pub fn height(mut self, value: Val) -> Self {
        self.height = Some(value);
        self
    }

    pub fn min_width(mut self, value: Val) -> Self {
        self.min_width = Some(value);
        self
    }

    pub fn min_height(mut self, value: Val) -> Self {
        self.min_height = Some(value);
        self
    }

    pub fn max_width(mut self, value: Val) -> Self {
        self.max_width = Some(value);
        self
    }

    pub fn max_height(mut self, value: Val) -> Self {
        self.max_height = Some(value);
        self
    }

    pub fn aspect_ratio(mut self, value: f32) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn align_items(mut self, value: AlignItems) -> Self {
        self.align_items = Some(value);
        self
    }

    pub fn justify_items(mut self, value: JustifyItems) -> Self {
        self.justify_items = Some(value);
        self
    }

    pub fn align_self(mut self, value: AlignSelf) -> Self {
        self.align_self = Some(value);
        self
    }

    pub fn justify_self(mut self, value: JustifySelf) -> Self {
        self.justify_self = Some(value);
        self
    }

    pub fn align_content(mut self, value: AlignContent) -> Self {
        self.align_content = Some(value);
        self
    }

    pub fn justify_content(mut self, value: JustifyContent) -> Self {
        self.justify_content = Some(value);
        self
    }

    pub fn margin(mut self, value: UiRect) -> Self {
        self.margin = Some(value);
        self
    }

    /// Set margin on all sides in pixels
    pub fn margin_all_px(self, value: f32) -> Self {
        self.margin(UiRect::all(Val::Px(value)))
    }

    /// Set margin on all sides in percent
    pub fn margin_all_percent(self, value: f32) -> Self {
        self.margin(UiRect::all(Val::Percent(value)))
    }

    pub fn padding(mut self, value: UiRect) -> Self {
        self.padding = Some(value);
        self
    }

    /// Set padding on all sides in pixels
    pub fn padding_all_px(self, value: f32) -> Self {
        self.padding(UiRect::all(Val::Px(value)))
    }

    /// Set padding on all sides in percent
    pub fn padding_all_percent(self, value: f32) -> Self {
        self.padding(UiRect::all(Val::Percent(value)))
    }

    pub fn border(mut self, value: UiRect) -> Self {
        self.border = Some(value);
        self
    }

    /// Set border on all sides in pixels
    pub fn border_all_px(self, value: f32) -> Self {
        self.border(UiRect::all(Val::Px(value)))
    }

    pub fn flex_direction(mut self, value: FlexDirection) -> Self {
        self.flex_direction = Some(value);
        self
    }

    pub fn flex_wrap(mut self, value: FlexWrap) -> Self {
        self.flex_wrap = Some(value);
        self
    }

    pub fn flex_grow(mut self, value: f32) -> Self {
        self.flex_grow = Some(value);
        self
    }

    pub fn flex_shrink(mut self, value: f32) -> Self {
        self.flex_shrink = Some(value);
        self
    }

    pub fn flex_basis(mut self, value: Val) -> Self {
        self.flex_basis = Some(value);
        self
    }

    pub fn row_gap(mut self, value: Val) -> Self {
        self.row_gap = Some(value);
        self
    }

    pub fn column_gap(mut self, value: Val) -> Self {
        self.column_gap = Some(value);
        self
    }

    pub fn grid_auto_flow(mut self, value: GridAutoFlow) -> Self {
        self.grid_auto_flow = Some(value);
        self
    }

    pub fn grid_template_rows(mut self, value: Vec<RepeatedGridTrack>) -> Self {
        self.grid_template_rows = Some(value);
        self
    }

    pub fn grid_template_columns(mut self, value: Vec<RepeatedGridTrack>) -> Self {
        self.grid_template_columns = Some(value);
        self
    }

    pub fn grid_auto_rows(mut self, value: Vec<GridTrack>) -> Self {
        self.grid_auto_rows = Some(value);
        self
    }

    pub fn grid_auto_columns(mut self, value: Vec<GridTrack>) -> Self {
        self.grid_auto_columns = Some(value);
        self
    }

    pub fn grid_row(mut self, value: GridPlacement) -> Self {
        self.grid_row = Some(value);
        self
    }

    pub fn grid_column(mut self, value: GridPlacement) -> Self {
        self.grid_column = Some(value);
        self
    }

    pub fn border_radius(mut self, value: BorderRadius) -> Self {
        self.border_radius = Some(value);
        self
    }

    /// Set border radius on all corners in pixels
    pub fn border_radius_all_px(self, value: f32) -> Self {
        self.border_radius(BorderRadius::all(Val::Px(value)))
    }

    /// Set border radius on all corners in percent
    pub fn border_radius_all_percent(self, value: f32) -> Self {
        self.border_radius(BorderRadius::all(Val::Percent(value)))
    }

    /// Set border radius to zero
    pub fn border_radius_zero(self) -> Self {
        self.border_radius(BorderRadius::ZERO)
    }

    pub fn border_color(mut self, value: Color) -> Self {
        self.border_color = Some(value);
        self
    }

    pub fn bg_color(mut self, value: Color) -> Self {
        self.bg_color = Some(value);
        self
    }
}

#[derive(Resource, Default, Clone, Debug)]
pub struct UiBuilderDefaults {
    pub node_def: Option<NodePartial>,
    pub button_def: Option<ButtonPartial>,
    pub base_font: Handle<Font>,
    pub font_size: f32,
    pub bg_color: Color,
    pub border_color: Color,
    pub text_color: Color,
    pub text_justify: Option<bevy::text::Justify>,
    pub text_line_break: Option<bevy::text::LineBreak>,
}

impl UiBuilderDefaults {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_btn_def(&self, input: Option<ButtonPartial>) -> ButtonDef {
        let mut def = ButtonDef::default();
        match (&self.button_def, input) {
            (Some(internal), Some(input)) => {
                def.text = input
                    .text
                    .unwrap_or(internal.text.clone().unwrap_or(def.text));
                def.font = input
                    .font
                    .unwrap_or(internal.font.clone().unwrap_or(def.font));
                def.width = input.width.unwrap_or(internal.width.unwrap_or(def.width));
                def.height = input
                    .height
                    .unwrap_or(internal.height.unwrap_or(def.height));
                def.border = input
                    .border
                    .unwrap_or(internal.border.unwrap_or(def.border));
                def.justify_content = input
                    .justify_content
                    .unwrap_or(internal.justify_content.unwrap_or(def.justify_content));
                def.align_items = input
                    .align_items
                    .unwrap_or(internal.align_items.unwrap_or(def.align_items));
                def.border_radius = input
                    .border_radius
                    .unwrap_or(internal.border_radius.unwrap_or(def.border_radius));
                def.font_size = input
                    .font_size
                    .unwrap_or(internal.font_size.unwrap_or(self.font_size));
                def.bg_color = input
                    .bg_color
                    .unwrap_or(internal.bg_color.unwrap_or(self.bg_color));
                def.text_color = input
                    .text_color
                    .unwrap_or(internal.text_color.unwrap_or(self.text_color));
                def.border_color = input
                    .border_color
                    .unwrap_or(internal.border_color.unwrap_or(self.border_color));
            }
            (None, Some(input)) => {
                def.text = input.text.unwrap_or(def.text);
                def.font = input.font.unwrap_or(def.font).clone();
                def.width = input.width.unwrap_or(def.width);
                def.height = input.height.unwrap_or(def.height);
                def.border = input.border.unwrap_or(def.border);
                def.justify_content = input.justify_content.unwrap_or(def.justify_content);
                def.align_items = input.align_items.unwrap_or(def.align_items);
                def.border_color = input.border_color.unwrap_or(def.border_color);
                def.border_radius = input.border_radius.unwrap_or(def.border_radius).clone();
                def.bg_color = input.bg_color.unwrap_or(def.bg_color);
                def.font_size = input.font_size.unwrap_or(def.font_size);
                def.text_color = input.text_color.unwrap_or(def.text_color);
            }
            (Some(internal), None) => {
                def.text = internal.text.clone().unwrap_or(def.text);
                def.font = internal.font.clone().unwrap_or(def.font).clone();
                def.width = internal.width.unwrap_or(def.width);
                def.height = internal.height.unwrap_or(def.height);
                def.border = internal.border.unwrap_or(def.border);
                def.justify_content = internal.justify_content.unwrap_or(def.justify_content);
                def.align_items = internal.align_items.unwrap_or(def.align_items);
                def.border_color = internal.border_color.unwrap_or(def.border_color);
                def.border_radius = internal.border_radius.unwrap_or(def.border_radius).clone();
                def.bg_color = internal.bg_color.unwrap_or(def.bg_color);
                def.font_size = internal.font_size.unwrap_or(def.font_size);
                def.text_color = internal.text_color.unwrap_or(def.text_color);
            }
            (None, None) => {}
        }
        def
    }

    pub fn get_node_def(&self, input: Option<NodePartial>) -> NodeDef {
        let mut def = NodeDef::default();
        match (&self.node_def, input) {
            (Some(internal), Some(input)) => {
                def.display = input
                    .display
                    .unwrap_or(internal.display.unwrap_or(def.display));
                def.position_type = input
                    .position_type
                    .unwrap_or(internal.position_type.unwrap_or(def.position_type));
                def.overflow = input
                    .overflow
                    .unwrap_or(internal.overflow.unwrap_or(def.overflow));
                def.overflow_clip_margin = input.overflow_clip_margin.unwrap_or(
                    internal
                        .overflow_clip_margin
                        .unwrap_or(def.overflow_clip_margin),
                );
                def.left = input.left.unwrap_or(internal.left.unwrap_or(def.left));
                def.right = input.right.unwrap_or(internal.right.unwrap_or(def.right));
                def.top = input.top.unwrap_or(internal.top.unwrap_or(def.top));
                def.bottom = input
                    .bottom
                    .unwrap_or(internal.bottom.unwrap_or(def.bottom));
                def.width = input.width.unwrap_or(internal.width.unwrap_or(def.width));
                def.height = input
                    .height
                    .unwrap_or(internal.height.unwrap_or(def.height));
                def.min_width = input
                    .min_width
                    .unwrap_or(internal.min_width.unwrap_or(def.min_width));
                def.min_height = input
                    .min_height
                    .unwrap_or(internal.min_height.unwrap_or(def.min_height));
                def.max_width = input
                    .max_width
                    .unwrap_or(internal.max_width.unwrap_or(def.max_width));
                def.max_height = input
                    .max_height
                    .unwrap_or(internal.max_height.unwrap_or(def.max_height));
                def.aspect_ratio = input.aspect_ratio.or(internal.aspect_ratio);
                def.align_items = input
                    .align_items
                    .unwrap_or(internal.align_items.unwrap_or(def.align_items));
                def.justify_items = input
                    .justify_items
                    .unwrap_or(internal.justify_items.unwrap_or(def.justify_items));
                def.align_self = input
                    .align_self
                    .unwrap_or(internal.align_self.unwrap_or(def.align_self));
                def.justify_self = input
                    .justify_self
                    .unwrap_or(internal.justify_self.unwrap_or(def.justify_self));
                def.align_content = input
                    .align_content
                    .unwrap_or(internal.align_content.unwrap_or(def.align_content));
                def.justify_content = input
                    .justify_content
                    .unwrap_or(internal.justify_content.unwrap_or(def.justify_content));
                def.margin = input
                    .margin
                    .unwrap_or(internal.margin.unwrap_or(def.margin));
                def.padding = input
                    .padding
                    .unwrap_or(internal.padding.unwrap_or(def.padding));
                def.border = input
                    .border
                    .unwrap_or(internal.border.unwrap_or(def.border));
                def.flex_direction = input
                    .flex_direction
                    .unwrap_or(internal.flex_direction.unwrap_or(def.flex_direction));
                def.flex_wrap = input
                    .flex_wrap
                    .unwrap_or(internal.flex_wrap.unwrap_or(def.flex_wrap));
                def.flex_grow = input
                    .flex_grow
                    .unwrap_or(internal.flex_grow.unwrap_or(def.flex_grow));
                def.flex_shrink = input
                    .flex_shrink
                    .unwrap_or(internal.flex_shrink.unwrap_or(def.flex_shrink));
                def.flex_basis = input
                    .flex_basis
                    .unwrap_or(internal.flex_basis.unwrap_or(def.flex_basis));
                def.row_gap = input
                    .row_gap
                    .unwrap_or(internal.row_gap.unwrap_or(def.row_gap));
                def.column_gap = input
                    .column_gap
                    .unwrap_or(internal.column_gap.unwrap_or(def.column_gap));
                def.grid_auto_flow = input
                    .grid_auto_flow
                    .unwrap_or(internal.grid_auto_flow.unwrap_or(def.grid_auto_flow));
                def.grid_template_rows = input.grid_template_rows.clone().unwrap_or(
                    internal
                        .grid_template_rows
                        .clone()
                        .unwrap_or(def.grid_template_rows.clone()),
                );
                def.grid_template_columns = input.grid_template_columns.clone().unwrap_or(
                    internal
                        .grid_template_columns
                        .clone()
                        .unwrap_or(def.grid_template_columns.clone()),
                );
                def.grid_auto_rows = input.grid_auto_rows.clone().unwrap_or(
                    internal
                        .grid_auto_rows
                        .clone()
                        .unwrap_or(def.grid_auto_rows.clone()),
                );
                def.grid_auto_columns = input.grid_auto_columns.clone().unwrap_or(
                    internal
                        .grid_auto_columns
                        .clone()
                        .unwrap_or(def.grid_auto_columns.clone()),
                );
                def.grid_row = input
                    .grid_row
                    .unwrap_or(internal.grid_row.unwrap_or(def.grid_row));
                def.grid_column = input
                    .grid_column
                    .unwrap_or(internal.grid_column.unwrap_or(def.grid_column));
                def.bg_color = input
                    .bg_color
                    .unwrap_or(internal.bg_color.unwrap_or(self.bg_color));
                def.border_radius = input
                    .border_radius
                    .unwrap_or(internal.border_radius.unwrap_or(def.border_radius));
                def.border_color = input
                    .border_color
                    .unwrap_or(internal.border_color.unwrap_or(self.border_color));
            }
            (None, Some(input)) => {
                def.display = input.display.unwrap_or(def.display);
                def.position_type = input.position_type.unwrap_or(def.position_type);
                def.overflow = input.overflow.unwrap_or(def.overflow);
                def.overflow_clip_margin = input
                    .overflow_clip_margin
                    .unwrap_or(def.overflow_clip_margin);
                def.left = input.left.unwrap_or(def.left);
                def.right = input.right.unwrap_or(def.right);
                def.top = input.top.unwrap_or(def.top);
                def.bottom = input.bottom.unwrap_or(def.bottom);
                def.width = input.width.unwrap_or(def.width);
                def.height = input.height.unwrap_or(def.height);
                def.min_width = input.min_width.unwrap_or(def.min_width);
                def.min_height = input.min_height.unwrap_or(def.min_height);
                def.max_width = input.max_width.unwrap_or(def.max_width);
                def.max_height = input.max_height.unwrap_or(def.max_height);
                def.aspect_ratio = input.aspect_ratio.or(def.aspect_ratio);
                def.align_items = input.align_items.unwrap_or(def.align_items);
                def.justify_items = input.justify_items.unwrap_or(def.justify_items);
                def.align_self = input.align_self.unwrap_or(def.align_self);
                def.justify_self = input.justify_self.unwrap_or(def.justify_self);
                def.align_content = input.align_content.unwrap_or(def.align_content);
                def.justify_content = input.justify_content.unwrap_or(def.justify_content);
                def.margin = input.margin.unwrap_or(def.margin);
                def.padding = input.padding.unwrap_or(def.padding);
                def.border = input.border.unwrap_or(def.border);
                def.flex_direction = input.flex_direction.unwrap_or(def.flex_direction);
                def.flex_wrap = input.flex_wrap.unwrap_or(def.flex_wrap);
                def.flex_grow = input.flex_grow.unwrap_or(def.flex_grow);
                def.flex_shrink = input.flex_shrink.unwrap_or(def.flex_shrink);
                def.flex_basis = input.flex_basis.unwrap_or(def.flex_basis);
                def.row_gap = input.row_gap.unwrap_or(def.row_gap);
                def.column_gap = input.column_gap.unwrap_or(def.column_gap);
                def.grid_auto_flow = input.grid_auto_flow.unwrap_or(def.grid_auto_flow);
                def.grid_template_rows = input
                    .grid_template_rows
                    .clone()
                    .unwrap_or(def.grid_template_rows.clone());
                def.grid_template_columns = input
                    .grid_template_columns
                    .clone()
                    .unwrap_or(def.grid_template_columns.clone());
                def.grid_auto_rows = input
                    .grid_auto_rows
                    .clone()
                    .unwrap_or(def.grid_auto_rows.clone());
                def.grid_auto_columns = input
                    .grid_auto_columns
                    .clone()
                    .unwrap_or(def.grid_auto_columns.clone());
                def.grid_row = input.grid_row.unwrap_or(def.grid_row);
                def.grid_column = input.grid_column.unwrap_or(def.grid_column);
                def.bg_color = input.bg_color.unwrap_or(self.bg_color);
                def.border_radius = input.border_radius.unwrap_or(def.border_radius);
                def.border_color = input.border_color.unwrap_or(self.border_color);
            }
            (Some(internal), None) => {
                def.display = internal.display.unwrap_or(def.display);
                def.position_type = internal.position_type.unwrap_or(def.position_type);
                def.overflow = internal.overflow.unwrap_or(def.overflow);
                def.overflow_clip_margin = internal
                    .overflow_clip_margin
                    .unwrap_or(def.overflow_clip_margin);
                def.left = internal.left.unwrap_or(def.left);
                def.right = internal.right.unwrap_or(def.right);
                def.top = internal.top.unwrap_or(def.top);
                def.bottom = internal.bottom.unwrap_or(def.bottom);
                def.width = internal.width.unwrap_or(def.width);
                def.height = internal.height.unwrap_or(def.height);
                def.min_width = internal.min_width.unwrap_or(def.min_width);
                def.min_height = internal.min_height.unwrap_or(def.min_height);
                def.max_width = internal.max_width.unwrap_or(def.max_width);
                def.max_height = internal.max_height.unwrap_or(def.max_height);
                def.aspect_ratio = internal.aspect_ratio.or(def.aspect_ratio);
                def.align_items = internal.align_items.unwrap_or(def.align_items);
                def.justify_items = internal.justify_items.unwrap_or(def.justify_items);
                def.align_self = internal.align_self.unwrap_or(def.align_self);
                def.justify_self = internal.justify_self.unwrap_or(def.justify_self);
                def.align_content = internal.align_content.unwrap_or(def.align_content);
                def.justify_content = internal.justify_content.unwrap_or(def.justify_content);
                def.margin = internal.margin.unwrap_or(def.margin);
                def.padding = internal.padding.unwrap_or(def.padding);
                def.border = internal.border.unwrap_or(def.border);
                def.flex_direction = internal.flex_direction.unwrap_or(def.flex_direction);
                def.flex_wrap = internal.flex_wrap.unwrap_or(def.flex_wrap);
                def.flex_grow = internal.flex_grow.unwrap_or(def.flex_grow);
                def.flex_shrink = internal.flex_shrink.unwrap_or(def.flex_shrink);
                def.flex_basis = internal.flex_basis.unwrap_or(def.flex_basis);
                def.row_gap = internal.row_gap.unwrap_or(def.row_gap);
                def.column_gap = internal.column_gap.unwrap_or(def.column_gap);
                def.grid_auto_flow = internal.grid_auto_flow.unwrap_or(def.grid_auto_flow);
                def.grid_template_rows = internal
                    .grid_template_rows
                    .clone()
                    .unwrap_or(def.grid_template_rows.clone());
                def.grid_template_columns = internal
                    .grid_template_columns
                    .clone()
                    .unwrap_or(def.grid_template_columns.clone());
                def.grid_auto_rows = internal
                    .grid_auto_rows
                    .clone()
                    .unwrap_or(def.grid_auto_rows.clone());
                def.grid_auto_columns = internal
                    .grid_auto_columns
                    .clone()
                    .unwrap_or(def.grid_auto_columns.clone());
                def.grid_row = internal.grid_row.unwrap_or(def.grid_row);
                def.grid_column = internal.grid_column.unwrap_or(def.grid_column);
                def.bg_color = internal.bg_color.unwrap_or(self.bg_color);
                def.border_radius = internal.border_radius.unwrap_or(def.border_radius);
                def.border_color = internal.border_color.unwrap_or(self.border_color);
            }
            (None, None) => {}
        }
        def
    }

    pub fn get_node_bundle(
        &self,
        input: Option<NodePartial>,
    ) -> NodeBundle {
        self.get_node_def(input).get_node_bundle()
    }

    pub fn get_btn_node(&self, input: Option<ButtonPartial>) -> Node {
        let btn_def = self.get_btn_def(input);

        let node_partial = NodePartial {
            width: Some(btn_def.width),
            height: Some(btn_def.height),
            border: Some(btn_def.border),
            justify_content: Some(btn_def.justify_content),
            align_items: Some(btn_def.align_items),
            ..default()
        };
        self.get_node_def(Some(node_partial)).get_node()
    }

    pub fn get_btn_bundle(
        &self,
        input: Option<ButtonPartial>,
    ) -> ButtonBundle {
        self.get_btn_def(input).get_button_bundle()
    }

    pub fn get_btn_text_bundle(&self, input: Option<ButtonPartial>) -> TextBundle {
        self.get_btn_def(input).get_text_bundle()
    }
}

impl<'w, 's> UIBuilder<'w, 's> {
    /// Get a reference to the UI builder defaults
    pub fn get_defaults(&self) -> &UiBuilderDefaults {
        &self.defaults
    }

    /// Create a new root UI container
    pub fn new(mut commands: Commands<'w, 's>, defaults: Option<UiBuilderDefaults>) -> Self {
        // Create a basic node entity with default settings
        let entity = commands.spawn_empty().id();
        let defaults = defaults.unwrap_or_default();
        commands
            .entity(entity)
            .insert(defaults.get_node_bundle(None));

        Self {
            commands,
            defaults,
            current_entity: entity,
            parent_stack: VecDeque::new(),
        }
    }

    pub fn start_from_entity(
        mut commands: Commands<'w, 's>,
        entity: Entity,
        clear_children: bool,
        defaults: Option<UiBuilderDefaults>,
    ) -> Self {
        let defaults = defaults.unwrap_or_default();
        if clear_children {
            commands.entity(entity).despawn_related::<Children>();
        }
        Self {
            commands,
            defaults,
            current_entity: entity,
            parent_stack: VecDeque::new(),
        }
    }

    pub fn with_button<T: Component>(
        &mut self,
        button_partial: Option<ButtonPartial>,
        component: T,
    ) -> &mut Self {
        let internal = self.child();
        internal
            .commands
            .entity(internal.current_entity)
            .insert(internal.defaults.get_btn_node(button_partial.clone()))
            .insert(internal.defaults.get_btn_bundle(button_partial.clone()))
            .insert(component)
            .with_child(internal.defaults.get_btn_text_bundle(button_partial));
        internal.parent()
    }

    /// Apply a NodePartial to the current entity's node
    ///
    /// This function takes a NodePartial and applies any non-None properties to the current entity's node.
    /// This is useful for applying a set of properties at once without having to specify each one individually.
    pub fn with(&mut self, partial: NodePartial) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                if let Some(display) = partial.display {
                    n.display = display;
                }
                if let Some(position_type) = partial.position_type {
                    n.position_type = position_type;
                }
                if let Some(overflow) = partial.overflow {
                    n.overflow = overflow;
                }
                if let Some(overflow_clip_margin) = partial.overflow_clip_margin {
                    n.overflow_clip_margin = overflow_clip_margin;
                }
                if let Some(left) = partial.left {
                    n.left = left;
                }
                if let Some(right) = partial.right {
                    n.right = right;
                }
                if let Some(top) = partial.top {
                    n.top = top;
                }
                if let Some(bottom) = partial.bottom {
                    n.bottom = bottom;
                }
                if let Some(width) = partial.width {
                    n.width = width;
                }
                if let Some(height) = partial.height {
                    n.height = height;
                }
                if let Some(min_width) = partial.min_width {
                    n.min_width = min_width;
                }
                if let Some(min_height) = partial.min_height {
                    n.min_height = min_height;
                }
                if let Some(max_width) = partial.max_width {
                    n.max_width = max_width;
                }
                if let Some(max_height) = partial.max_height {
                    n.max_height = max_height;
                }
                if let Some(aspect_ratio) = partial.aspect_ratio {
                    n.aspect_ratio = Some(aspect_ratio);
                }
                if let Some(align_items) = partial.align_items {
                    n.align_items = align_items;
                }
                if let Some(justify_items) = partial.justify_items {
                    n.justify_items = justify_items;
                }
                if let Some(align_self) = partial.align_self {
                    n.align_self = align_self;
                }
                if let Some(justify_self) = partial.justify_self {
                    n.justify_self = justify_self;
                }
                if let Some(align_content) = partial.align_content {
                    n.align_content = align_content;
                }
                if let Some(justify_content) = partial.justify_content {
                    n.justify_content = justify_content;
                }
                if let Some(margin) = partial.margin {
                    n.margin = margin;
                }
                if let Some(padding) = partial.padding {
                    n.padding = padding;
                }
                if let Some(border) = partial.border {
                    n.border = border;
                }
                if let Some(flex_direction) = partial.flex_direction {
                    n.flex_direction = flex_direction;
                }
                if let Some(flex_wrap) = partial.flex_wrap {
                    n.flex_wrap = flex_wrap;
                }
                if let Some(flex_grow) = partial.flex_grow {
                    n.flex_grow = flex_grow;
                }
                if let Some(flex_shrink) = partial.flex_shrink {
                    n.flex_shrink = flex_shrink;
                }
                if let Some(flex_basis) = partial.flex_basis {
                    n.flex_basis = flex_basis;
                }
                if let Some(row_gap) = partial.row_gap {
                    n.row_gap = row_gap;
                }
                if let Some(column_gap) = partial.column_gap {
                    n.column_gap = column_gap;
                }
                if let Some(grid_auto_flow) = partial.grid_auto_flow {
                    n.grid_auto_flow = grid_auto_flow;
                }
                if let Some(grid_template_rows) = partial.grid_template_rows {
                    n.grid_template_rows = grid_template_rows;
                }
                if let Some(grid_template_columns) = partial.grid_template_columns {
                    n.grid_template_columns = grid_template_columns;
                }
                if let Some(grid_auto_rows) = partial.grid_auto_rows {
                    n.grid_auto_rows = grid_auto_rows;
                }
                if let Some(grid_auto_columns) = partial.grid_auto_columns {
                    n.grid_auto_columns = grid_auto_columns;
                }
                if let Some(grid_row) = partial.grid_row {
                    n.grid_row = grid_row;
                }
                if let Some(grid_column) = partial.grid_column {
                    n.grid_column = grid_column;
                }
            });
        self
    }

    /// Set the display property of the current entity's node
    pub fn display(&mut self, display: Display) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.display = display;
            });
        self
    }

    /// Set display to Flex
    pub fn display_flex(&mut self) -> &mut Self {
        self.display(Display::Flex)
    }

    /// Set display to Grid
    pub fn display_grid(&mut self) -> &mut Self {
        self.display(Display::Grid)
    }

    /// Set grid template columns with N equal-width columns using fr units
    pub fn grid_cols(&mut self, count: u16) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.grid_template_columns = vec![RepeatedGridTrack::fr(count, 1.0)];
            });
        self
    }

    /// Set grid template rows with N equal-height rows using fr units
    pub fn grid_rows(&mut self, count: u16) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.grid_template_rows = vec![RepeatedGridTrack::fr(count, 1.0)];
            });
        self
    }

    /// Set grid template columns with N columns of fixed pixel width
    pub fn grid_cols_px(&mut self, count: u16, width: f32) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.grid_template_columns = vec![RepeatedGridTrack::px(count, width)];
            });
        self
    }

    /// Set grid template columns to auto-fill with minimum width
    pub fn grid_cols_auto_fill_percent(&mut self, min_width: f32) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.grid_template_columns = vec![RepeatedGridTrack::minmax(
                    GridTrackRepetition::AutoFill,
                    MinTrackSizingFunction::Percent(min_width),
                    MaxTrackSizingFunction::Fraction(1.0),
                )];
            });
        self
    }

    /// Set grid template columns to auto-fill with minimum width
    pub fn grid_cols_auto_fill_px(&mut self, min_width: f32) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.grid_template_columns = vec![RepeatedGridTrack::minmax(
                    GridTrackRepetition::AutoFill,
                    MinTrackSizingFunction::Px(min_width),
                    MaxTrackSizingFunction::Fraction(1.0),
                )];
            });
        self
    }

    /// Set the gap between grid cells
    pub fn grid_gap_px(&mut self, gap: f32) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.row_gap = Val::Px(gap);
                n.column_gap = Val::Px(gap);
            });
        self
    }

    pub fn grid_gap_percent(&mut self, gap: f32) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.row_gap = Val::Percent(gap);
                n.column_gap = Val::Percent(gap);
            });
        self
    }

    /// Set display to Block
    pub fn display_block(&mut self) -> &mut Self {
        self.display(Display::Block)
    }

    /// Set display to None
    pub fn display_none(&mut self) -> &mut Self {
        self.display(Display::None)
    }

    /// Set the position type property of the current entity's node
    pub fn with_position_type(&mut self, position_type: PositionType) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.position_type = position_type;
            });
        self
    }

    /// Set the overflow property of the current entity's node
    pub fn with_overflow(&mut self, overflow: Overflow) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.overflow = overflow;
            });
        self
    }

    /// Set the overflow clip margin property of the current entity's node
    pub fn with_overflow_clip_margin(&mut self, margin: OverflowClipMargin) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.overflow_clip_margin = margin;
            });
        self
    }

    /// Set the aspect ratio property of the current entity's node
    pub fn aspect_ratio(&mut self, ratio: f32) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.aspect_ratio = Some(ratio);
            });
        self
    }

    /// Clear the aspect ratio property of the current entity's node
    pub fn clear_aspect_ratio(&mut self) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.aspect_ratio = None;
            });
        self
    }

    /// Set the left position of the current entity's node
    pub fn with_left(&mut self, left: Val) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.left = left;
            });
        self
    }

    /// Set the right position of the current entity's node
    pub fn with_right(&mut self, right: Val) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.right = right;
            });
        self
    }

    /// Set the top position of the current entity's node
    pub fn with_top(&mut self, top: Val) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.top = top;
            });
        self
    }

    /// Set the bottom position of the current entity's node
    pub fn with_bottom(&mut self, bottom: Val) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.bottom = bottom;
            });
        self
    }

    /// Set the position (left, right, top, bottom) of the current entity's node
    pub fn with_position(&mut self, left: Val, right: Val, top: Val, bottom: Val) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.left = left;
                n.right = right;
                n.top = top;
                n.bottom = bottom;
            });
        self
    }

    /// Set the flex direction property of the current entity's node
    pub fn with_flex_direction(&mut self, direction: FlexDirection) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.flex_direction = direction;
            });
        self
    }

    /// Set flex direction to Row
    pub fn flex_dir_row(&mut self) -> &mut Self {
        self.with_flex_direction(FlexDirection::Row)
    }

    /// Set flex direction to Column
    pub fn flex_dir_column(&mut self) -> &mut Self {
        self.with_flex_direction(FlexDirection::Column)
    }

    /// Set flex direction to RowReverse
    pub fn flex_dir_row_reverse(&mut self) -> &mut Self {
        self.with_flex_direction(FlexDirection::RowReverse)
    }

    /// Set flex direction to ColumnReverse
    pub fn flex_dir_column_reverse(&mut self) -> &mut Self {
        self.with_flex_direction(FlexDirection::ColumnReverse)
    }

    /// Set flex wrap to Wrap
    pub fn flex_wrap(&mut self) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.flex_wrap = FlexWrap::Wrap;
            });
        self
    }

    /// Set flex wrap to NoWrap
    pub fn flex_wrap_none(&mut self) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.flex_wrap = FlexWrap::NoWrap;
            });
        self
    }

    /// Set flex wrap to WrapReverse
    pub fn flex_wrap_reverse(&mut self) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.flex_wrap = FlexWrap::WrapReverse;
            });
        self
    }

    /// Set the flex grow property of the current entity's node
    pub fn with_flex_grow(&mut self, grow: f32) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.flex_grow = grow;
            });
        self
    }

    /// Set the flex shrink property of the current entity's node
    pub fn with_flex_shrink(&mut self, shrink: f32) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.flex_shrink = shrink;
            });
        self
    }

    /// Set the flex basis property of the current entity's node
    pub fn with_flex_basis(&mut self, basis: Val) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.flex_basis = basis;
            });
        self
    }

    /// Set the align items property of the current entity's node
    pub fn with_align_items(&mut self, align: AlignItems) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.align_items = align;
            });
        self
    }

    /// Set align items to Default
    pub fn align_items_default(&mut self) -> &mut Self {
        self.with_align_items(AlignItems::Default)
    }

    /// Set align items to FlexStart
    pub fn align_items_start(&mut self) -> &mut Self {
        self.with_align_items(AlignItems::FlexStart)
    }

    /// Set align items to FlexEnd
    pub fn align_items_end(&mut self) -> &mut Self {
        self.with_align_items(AlignItems::FlexEnd)
    }

    /// Set align items to Center
    pub fn align_items_center(&mut self) -> &mut Self {
        self.with_align_items(AlignItems::Center)
    }

    /// Set align items to Baseline
    pub fn align_items_baseline(&mut self) -> &mut Self {
        self.with_align_items(AlignItems::Baseline)
    }

    /// Set align items to Stretch
    pub fn align_items_stretch(&mut self) -> &mut Self {
        self.with_align_items(AlignItems::Stretch)
    }

    /// Set the justify items property of the current entity's node
    pub fn justify_items(&mut self, justify: JustifyItems) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.justify_items = justify;
            });
        self
    }

    /// Set the align self property of the current entity's node
    pub fn with_align_self(&mut self, align: AlignSelf) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.align_self = align;
            });
        self
    }

    /// Set the justify self property of the current entity's node
    pub fn with_justify_self(&mut self, justify: JustifySelf) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.justify_self = justify;
            });
        self
    }

    /// Set the align content property of the current entity's node
    pub fn align_content(&mut self, align: AlignContent) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.align_content = align;
            });
        self
    }

    /// Set the justify content property of the current entity's node
    pub fn justify_content(&mut self, justify: JustifyContent) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.justify_content = justify;
            });
        self
    }

    /// Set justify content to FlexStart
    pub fn justify_start(&mut self) -> &mut Self {
        self.justify_content(JustifyContent::FlexStart)
    }

    /// Set justify content to FlexEnd
    pub fn justify_end(&mut self) -> &mut Self {
        self.justify_content(JustifyContent::FlexEnd)
    }

    /// Set justify content to Center
    pub fn justify_center(&mut self) -> &mut Self {
        self.justify_content(JustifyContent::Center)
    }

    /// Set justify content to SpaceBetween
    pub fn justify_space_between(&mut self) -> &mut Self {
        self.justify_content(JustifyContent::SpaceBetween)
    }

    /// Set justify content to SpaceAround
    pub fn justify_space_around(&mut self) -> &mut Self {
        self.justify_content(JustifyContent::SpaceAround)
    }

    /// Set justify content to SpaceEvenly
    pub fn justify_space_evenly(&mut self) -> &mut Self {
        self.justify_content(JustifyContent::SpaceEvenly)
    }

    /// Set the row gap property of the current entity's node
    pub fn row_gap(&mut self, gap: Val) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.row_gap = gap;
            });
        self
    }

    pub fn row_gap_px(&mut self, gap: f32) -> &mut Self {
        self.row_gap(Val::Px(gap))
    }

    pub fn column_gap_px(&mut self, gap: f32) -> &mut Self {
        self.column_gap(Val::Px(gap))
    }

    pub fn column_gap_percent(&mut self, gap: f32) -> &mut Self {
        self.column_gap(Val::Percent(gap))
    }

    pub fn row_gap_percent(&mut self, gap: f32) -> &mut Self {
        self.row_gap(Val::Percent(gap))
    }

    /// Set the column gap property of the current entity's node
    pub fn column_gap(&mut self, gap: Val) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.column_gap = gap;
            });
        self
    }

    /// Set both row and column gap properties of the current entity's node
    pub fn with_gap(&mut self, row_gap: Val, column_gap: Val) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.row_gap = row_gap;
                n.column_gap = column_gap;
            });
        self
    }

    /// Set the width property of the current entity's node
    pub fn with_width(&mut self, width: Val) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.width = width;
            });
        self
    }

    /// Set the height property of the current entity's node
    pub fn with_height(&mut self, height: Val) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.height = height;
            });
        self
    }

    /// Set the min width property of the current entity's node
    pub fn with_min_width(&mut self, min_width: Val) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.min_width = min_width;
            });
        self
    }

    /// Set the min height property of the current entity's node
    pub fn with_min_height(&mut self, min_height: Val) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.min_height = min_height;
            });
        self
    }

    /// Set the max width property of the current entity's node
    pub fn with_max_width(&mut self, max_width: Val) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.max_width = max_width;
            });
        self
    }

    /// Set the max height property of the current entity's node
    pub fn with_max_height(&mut self, max_height: Val) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.max_height = max_height;
            });
        self
    }

    pub fn with_child<F>(&mut self, mut f: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        let original_entity = self.current_entity;
        let original_stack_len = self.parent_stack.len();
        self.child();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            f(self);
        }));

        self.current_entity = original_entity;
        while self.parent_stack.len() > original_stack_len {
            self.parent_stack.pop_back();
        }

        if let Err(panic_payload) = result {
            std::panic::resume_unwind(panic_payload);
        }
        self
    }

    /// Add a flex row container child
    pub fn add_row<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        self.with_child(|ui| {
            ui.as_flex_row();
            f(ui);
        })
    }

    /// Add a flex column container child
    pub fn add_column<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        self.with_child(|ui| {
            ui.display_flex().flex_dir_column();
            f(ui);
        })
    }

    /// Add a panel container child with padding
    pub fn add_panel<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        self.with_child(|ui| {
            ui.display_flex().flex_dir_column().padding_all_px(16.0);
            f(ui);
        })
    }

    /// Add a grid container child with specified number of columns
    pub fn add_grid<F>(&mut self, cols: u16, f: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        self.with_child(|ui| {
            ui.display_grid().grid_cols(cols).grid_gap_percent(2.0);
            f(ui);
        })
    }

    /// Add a card container child with padding and background
    pub fn add_card<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        self.with_child(|ui| {
            ui.display_flex()
                .flex_dir_column()
                .padding_all_px(16.0)
                .border_radius_all_px(8.0);
            f(ui);
        })
    }

    /// Add a centered container child
    pub fn add_centered<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        self.with_child(|ui| {
            ui.display_flex().justify_center().align_items_center();
            f(ui);
        })
    }

    pub fn foreach_child<I, F>(&mut self, iter: I, mut f: F) -> &mut Self
    where
        I: IntoIterator,
        F: FnMut(&mut Self, I::Item),
    {
        let original_entity = self.current_entity;
        let original_stack_len = self.parent_stack.len();

        for item in iter {
            self.child();

            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                f(self, item);
            }));

            self.current_entity = original_entity;
            while self.parent_stack.len() > original_stack_len {
                self.parent_stack.pop_back();
            }

            if let Err(panic_payload) = result {
                std::panic::resume_unwind(panic_payload);
            }
        }

        self
    }

    pub fn as_block_with<T: Component + Default>(
        &mut self,
        width: Val,
        height: Val,
        bg_color: Color,
    ) -> &mut Self {
        self.as_block(width, height, bg_color).with_component::<T>()
    }

    pub fn at(&mut self, left: Val, top: Val, position_type: PositionType) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| {
                node.position_type = position_type;
                node.left = left;
                node.top = top;
            });
        self
    }

    /// Add a default instance of the given component to the current entity.
    ///
    /// If the entity does not already have the given component, this method will
    /// add a default instance of it. This is a shorthand for calling
    /// `commands.entity(self.current_entity).entry::<T>().or_default();`.
    pub fn with_component<T: Component + Default>(&mut self) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<T>()
            .or_insert(T::default());
        self
    }

    /// Insert a component instance into the current entity.
    pub fn insert<T: Component>(&mut self, component: T) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .insert(component);
        self
    }

    /// Set the current entity to be a flexbox container with a row flex direction
    ///
    /// The entity will be set to be a block with a flexbox display mode and a row
    /// flex direction. This means that any children of the entity will be laid out
    /// horizontally from left to right.
    ///
    /// # Example
    ///
    pub fn as_flex_row(&mut self) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| {
                node.display = Display::Flex;
                node.flex_direction = FlexDirection::Row;
            });
        self
    }

    pub fn as_flex_col(&mut self, width: Val, height: Val) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| {
                node.display = Display::Flex;
                node.flex_direction = FlexDirection::Column;
                node.align_items = AlignItems::FlexStart;
                node.align_content = AlignContent::FlexStart;
                node.justify_content = JustifyContent::FlexStart;
                node.width = width;
                node.height = height;
            });
        self
    }

    /// Set the current entity to be a block element with the specified dimensions and background color
    ///
    /// This method configures the entity's node to use block display mode and sets
    /// the width, height, and background color properties.
    ///
    /// # Arguments
    ///
    /// * `width` - The width of the block element
    /// * `height` - The height of the block element
    /// * `bg_color` - The background color of the block element
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining
    pub fn as_block(&mut self, width: Val, height: Val, bg_color: Color) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| {
                node.width = width;
                node.height = height;
                node.display = Display::Block;
            });

        self.commands
            .entity(self.current_entity)
            .entry::<BackgroundColor>()
            .and_modify(move |mut bg| {
                *bg = BackgroundColor(bg_color);
            })
            .or_insert(BackgroundColor(bg_color));
        self
    }

    pub fn as_block_px(&mut self, width: f32, height: f32, bg_color: Color) -> &mut Self {
        self.as_block(Val::Px(width), Val::Px(height), bg_color)
    }

    pub fn size(&mut self, width: Val, height: Val) -> &mut Self {
        self.width(width).height(height)
    }
    
    pub fn size_percent(&mut self, width: f32, height: f32) -> &mut Self {
        self.width_percent(width).height_percent(height)
    }
    
    pub fn size_px(&mut self, width: f32, height: f32) -> &mut Self {
        self.width_px(width).height_px(height)
    }

    pub fn size_auto(&mut self) -> &mut Self {
        self.width_auto().height_auto()
    }

    /// Set width Node
    pub fn width(&mut self, width: Val) -> &mut Self {
        // Get the current entity
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.width = width);
        self
    }

    pub fn width_px(&mut self, width: f32) -> &mut Self {
        self.width(Val::Px(width))
    }

    pub fn width_percent(&mut self, width: f32) -> &mut Self {
        self.width(Val::Percent(width))
    }

    pub fn width_auto(&mut self) -> &mut Self {
        self.width(Val::Auto)
    }

    pub fn height_auto(&mut self) -> &mut Self {
        self.height(Val::Auto)
    }

    pub fn height_px(&mut self, height: f32) -> &mut Self {
        self.height(Val::Px(height))
    }
    pub fn height_percent(&mut self, height: f32) -> &mut Self {
        self.height(Val::Percent(height))
    }

    /// Set height Node
    pub fn height(&mut self, height: Val) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.height = height);
        self
    }

    /// Set flex direction
    pub fn flex_direction(&mut self, direction: FlexDirection) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.flex_direction = direction);
        self
    }

    pub fn flex_direction_row(&mut self) -> &mut Self {
        self.flex_direction(FlexDirection::Row)
    }

    pub fn flex_direction_column(&mut self) -> &mut Self {
        self.flex_direction(FlexDirection::Column)
    }
    /// Set align items
    pub fn align_items(&mut self, align: AlignItems) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.align_items = align);
        self
    }

    /// Set padding
    pub fn padding(&mut self, padding: UiRect) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.padding = padding);
        self
    }

    /// Set padding on all sides in pixels (replaces entire padding)
    pub fn padding_all_px(&mut self, value: f32) -> &mut Self {
        self.padding(UiRect::all(Val::Px(value)))
    }

    /// Set padding on all sides in percent (replaces entire padding)
    pub fn padding_all_percent(&mut self, value: f32) -> &mut Self {
        self.padding(UiRect::all(Val::Percent(value)))
    }

    /// Set padding to zero on all sides
    pub fn padding_zero(&mut self) -> &mut Self {
        self.padding(UiRect::ZERO)
    }

    /// Set bottom padding in pixels (replaces entire padding)
    pub fn padding_btm_px(&mut self, value: f32) -> &mut Self {
        self.padding(UiRect::bottom(Val::Px(value)))
    }

    /// Set bottom padding in percent (replaces entire padding)
    pub fn padding_btm_percent(&mut self, value: f32) -> &mut Self {
        self.padding(UiRect::bottom(Val::Percent(value)))
    }

    /// Set top padding in pixels (replaces entire padding)
    pub fn padding_top_px(&mut self, value: f32) -> &mut Self {
        self.padding(UiRect::top(Val::Px(value)))
    }

    /// Set top padding in percent (replaces entire padding)
    pub fn padding_top_percent(&mut self, value: f32) -> &mut Self {
        self.padding(UiRect::top(Val::Percent(value)))
    }

    /// Set left padding in pixels (replaces entire padding)
    pub fn padding_left_px(&mut self, value: f32) -> &mut Self {
        self.padding(UiRect::left(Val::Px(value)))
    }

    /// Set left padding in percent (replaces entire padding)
    pub fn padding_left_percent(&mut self, value: f32) -> &mut Self {
        self.padding(UiRect::left(Val::Percent(value)))
    }

    /// Set right padding in pixels (replaces entire padding)
    pub fn padding_right_px(&mut self, value: f32) -> &mut Self {
        self.padding(UiRect::right(Val::Px(value)))
    }

    /// Set right padding in percent (replaces entire padding)
    pub fn padding_right_percent(&mut self, value: f32) -> &mut Self {
        self.padding(UiRect::right(Val::Percent(value)))
    }

    /// Modify bottom padding in pixels (preserves other padding)
    pub fn with_padding_btm_px(&mut self, value: f32) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.padding.bottom = Val::Px(value));
        self
    }

    /// Modify bottom padding in percent (preserves other padding)
    pub fn with_padding_btm_percent(&mut self, value: f32) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.padding.bottom = Val::Percent(value));
        self
    }

    /// Modify top padding in pixels (preserves other padding)
    pub fn with_padding_top_px(&mut self, value: f32) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.padding.top = Val::Px(value));
        self
    }

    /// Modify top padding in percent (preserves other padding)
    pub fn with_padding_top_percent(&mut self, value: f32) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.padding.top = Val::Percent(value));
        self
    }

    /// Modify left padding in pixels (preserves other padding)
    pub fn with_padding_left_px(&mut self, value: f32) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.padding.left = Val::Px(value));
        self
    }

    /// Modify left padding in percent (preserves other padding)
    pub fn with_padding_left_percent(&mut self, value: f32) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.padding.left = Val::Percent(value));
        self
    }

    /// Modify right padding in pixels (preserves other padding)
    pub fn with_padding_right_px(&mut self, value: f32) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.padding.right = Val::Px(value));
        self
    }

    /// Modify right padding in percent (preserves other padding)
    pub fn with_padding_right_percent(&mut self, value: f32) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.padding.right = Val::Percent(value));
        self
    }

    /// Set margin
    pub fn margin(&mut self, margin: UiRect) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.margin = margin);
        self
    }

    /// Set margin on all sides in pixels (replaces entire margin)
    pub fn margin_all_px(&mut self, value: f32) -> &mut Self {
        self.margin(UiRect::all(Val::Px(value)))
    }

    /// Set margin on all sides in percent (replaces entire margin)
    pub fn margin_all_percent(&mut self, value: f32) -> &mut Self {
        self.margin(UiRect::all(Val::Percent(value)))
    }

    /// Set margin to zero on all sides
    pub fn margin_zero(&mut self) -> &mut Self {
        self.margin(UiRect::ZERO)
    }

    /// Set bottom margin in pixels (replaces entire margin)
    pub fn margin_btm_px(&mut self, value: f32) -> &mut Self {
        self.margin(UiRect::bottom(Val::Px(value)))
    }

    /// Set bottom margin in percent (replaces entire margin)
    pub fn margin_btm_percent(&mut self, value: f32) -> &mut Self {
        self.margin(UiRect::bottom(Val::Percent(value)))
    }

    /// Set top margin in pixels (replaces entire margin)
    pub fn margin_top_px(&mut self, value: f32) -> &mut Self {
        self.margin(UiRect::top(Val::Px(value)))
    }

    /// Set top margin in percent (replaces entire margin)
    pub fn margin_top_percent(&mut self, value: f32) -> &mut Self {
        self.margin(UiRect::top(Val::Percent(value)))
    }

    /// Set left margin in pixels (replaces entire margin)
    pub fn margin_left_px(&mut self, value: f32) -> &mut Self {
        self.margin(UiRect::left(Val::Px(value)))
    }

    /// Set left margin in percent (replaces entire margin)
    pub fn margin_left_percent(&mut self, value: f32) -> &mut Self {
        self.margin(UiRect::left(Val::Percent(value)))
    }

    /// Set right margin in pixels (replaces entire margin)
    pub fn margin_right_px(&mut self, value: f32) -> &mut Self {
        self.margin(UiRect::right(Val::Px(value)))
    }

    /// Set right margin in percent (replaces entire margin)
    pub fn margin_right_percent(&mut self, value: f32) -> &mut Self {
        self.margin(UiRect::right(Val::Percent(value)))
    }

    /// Modify bottom margin in pixels (preserves other margins)
    pub fn with_margin_btm_px(&mut self, value: f32) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.margin.bottom = Val::Px(value));
        self
    }

    /// Modify bottom margin in percent (preserves other margins)
    pub fn with_margin_btm_percent(&mut self, value: f32) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.margin.bottom = Val::Percent(value));
        self
    }

    /// Modify top margin in pixels (preserves other margins)
    pub fn with_margin_top_px(&mut self, value: f32) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.margin.top = Val::Px(value));
        self
    }

    /// Modify top margin in percent (preserves other margins)
    pub fn with_margin_top_percent(&mut self, value: f32) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.margin.top = Val::Percent(value));
        self
    }

    /// Modify left margin in pixels (preserves other margins)
    pub fn with_margin_left_px(&mut self, value: f32) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.margin.left = Val::Px(value));
        self
    }

    /// Modify left margin in percent (preserves other margins)
    pub fn with_margin_left_percent(&mut self, value: f32) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.margin.left = Val::Percent(value));
        self
    }

    /// Modify right margin in pixels (preserves other margins)
    pub fn with_margin_right_px(&mut self, value: f32) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.margin.right = Val::Px(value));
        self
    }

    /// Modify right margin in percent (preserves other margins)
    pub fn with_margin_right_percent(&mut self, value: f32) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.margin.right = Val::Percent(value));
        self
    }

    pub fn border(&mut self, border: UiRect, border_color: Color) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.border = border);

        self.commands
            .entity(self.current_entity)
            .entry::<BorderColor>()
            .and_modify(move |mut b_color| *b_color = BorderColor::all(border_color))
            .or_insert(BorderColor::all(border_color));
        self
    }

    pub fn border_all_px(&mut self, width: f32, border_color: Color) -> &mut Self {
        self.border(UiRect::all(Val::Px(width)), border_color)
    }

    /// Set border radius on all corners in pixels
    pub fn border_radius_all_px(&mut self, value: f32) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.border_radius = BorderRadius::all(Val::Px(value));
            });
        self
    }

    /// Set border radius on all corners in percent
    pub fn border_radius_all_percent(&mut self, value: f32) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.border_radius = BorderRadius::all(Val::Percent(value));
            });
        self
    }

    /// Set border radius to zero
    pub fn border_radius_zero(&mut self) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.border_radius = BorderRadius::ZERO;
            });
        self
    }

    pub fn with_box_shadow(&mut self, offset: Vec2, spread: f32, blur: f32) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .insert(BoxShadow::new(
                Color::BLACK.with_alpha(0.8),
                Val::Percent(offset.x),
                Val::Percent(offset.y),
                Val::Percent(spread),
                Val::Px(blur),
            ));
        self
    }

    /// Apply a complete Node
    pub fn node(mut self, node: Node) -> Self {
        self.commands.entity(self.current_entity).insert(node);
        self
    }

    /// Add background color Component
    pub fn bg_color(&mut self, color: Color) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<BackgroundColor>()
            .and_modify(move |mut background| *background = BackgroundColor(color))
            .or_insert(BackgroundColor(color));
        self
    }

    /// Add background color using SRGBA values
    pub fn bg_color_srgba(&mut self, r: f32, g: f32, b: f32, a: f32) -> &mut Self {
        self.bg_color(Color::srgba(r, g, b, a))
    }

    /// Add a text child Entity
    pub fn add_text_child(
        &mut self,
        text: impl Into<String>,
        font: Option<Handle<Font>>,
        font_size: Option<f32>,
        color: Option<Color>,
    ) -> &mut Self {
        self.with_child(|b| { 
            b.with_text(text, font, font_size, color, None, None); 
        })
    }

    pub fn text_node(&mut self, text: impl Into<String>) -> &mut Self {
        self.add_text_child(text, None, None, None)
    }

    pub fn default_text(&mut self, text: impl Into<String>) -> &mut Self {
        self.with_text(text, None, None, None, None, None)
    }

    /// Add a text child and apply builder function to style it
    pub fn build_text<F>(&mut self, text: impl Into<String>, f: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        self.with_child(|ui| {
            ui.default_text(text);
            f(ui);
        })
    }

    /// Add a text child with a fixed width in pixels
    pub fn text_with_width(&mut self, text: impl Into<String>, width: f32) -> &mut Self {
        self.build_text(text, |ui| {
            ui.width_px(width);
        })
    }

    /// Add a centered text container with specified width
    /// Creates a flex container that centers text both horizontally and vertically
    pub fn add_centered_text(&mut self, text: impl Into<String>, width: f32, component: impl Component) -> &mut Self {
        self.with_child(|ui| {
            ui.width_px(width)
                .display_flex()
                .align_items_center()
                .justify_center();

            ui.build_text(text, |ui| {
                ui
                    .width_px(width)
                    .text_justify_center()  // This centers the text content
                    .insert(component);
            });
        })
    }

    /// Add a centered text container with specified width and apply builder to the container
    pub fn build_centered_text<F>(&mut self, text: impl Into<String>, width: f32, f: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        self.with_child(|ui| {
            ui.width_px(width)
                .display_flex()
                .align_items_center()
                .justify_center();
            f(ui);
            ui.with_child(|ui| {
                ui.default_text(text);
            });
        })
    }

    /// Set text layout justification (for text content alignment within text node)
    pub fn text_justify(&mut self, justify: Justify) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<TextLayout>()
            .and_modify(move |mut tl| tl.justify = justify)
            .or_insert(TextLayout::new(justify, LineBreak::default()));
        self
    }

    pub fn text_align(&mut self, justify: Justify) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<TextLayout>()
            .and_modify(move |mut tl| tl.justify = justify)
            .or_insert(TextLayout::new(justify, LineBreak::default()));
        self
    }

    /// Center text content within the text node
    pub fn text_justify_center(&mut self) -> &mut Self {
        self.text_justify(Justify::Center)
    }

    /// Left-align text content within the text node
    pub fn text_justify_left(&mut self) -> &mut Self {
        self.text_justify(Justify::Left)
    }

    /// Right-align text content within the text node
    pub fn text_justify_right(&mut self) -> &mut Self {
        self.text_justify(Justify::Right)
    }

    pub fn with_text(
        &mut self,
        text: impl Into<String>,
        font: Option<Handle<Font>>,
        font_size: Option<f32>,
        color: Option<Color>,
        justify: Option<bevy::text::Justify>,
        line_break: Option<bevy::text::LineBreak>,
    ) -> &mut Self {
        let text_color = color.unwrap_or(self.defaults.text_color);
        let layout = TextLayout::new(
            justify.unwrap_or(
                self.defaults.text_justify
                    .unwrap_or(Justify::default())), 
                line_break.unwrap_or(self.defaults.text_line_break.unwrap_or(LineBreak::default())));

        let text_bundle = (
            Text::new(text.into()),
            TextFont::default()
                .with_font(font.unwrap_or(self.defaults.base_font.clone()))
                .with_font_size(font_size.unwrap_or(self.defaults.font_size)),
            TextColor(text_color),
            layout,
        );

        // Add the text component to the entity
        self.commands
            .entity(self.current_entity)
            .insert(text_bundle);
        self
    }

    /// Create a child container
    pub fn child(&mut self) -> &mut Self {
        self.parent_stack.push_back(self.current_entity);

        // Spawn a new node entity as child
        let child = self.commands.spawn_empty().id();
        self.commands
            .entity(child)
            .insert(self.defaults.get_node_bundle(None));

        // Add the child to the current entity
        self.commands.entity(self.current_entity).add_child(child);

        // Update the current entity to the child
        self.current_entity = child;
        self
    }

    /// Return to parent container
    pub fn parent(&mut self) -> &mut Self {
        if let Some(parent) = self.parent_stack.pop_back() {
            self.current_entity = parent;
        }
        self
    }

    /// Finalize and get the root entity
    pub fn build(self) -> (Entity, Commands<'w, 's>) {
        // Return the top-level entity (first one created)
        if !self.parent_stack.is_empty() {
            // If we have parents, the first one is the root
            (self.parent_stack[0], self.commands)
        } else {
            // Otherwise return the current entity
            (self.current_entity, self.commands)
        }
    }

    /// Add a button with common properties and a single component
    pub fn add_button<T: Component>(
        &mut self,
        text: impl Into<String>,
        width: f32,
        height: f32,
        bg_color: Color,
        font_size: f32,
        border_radius: f32,
        component: T,
    ) -> &mut Self {
        self.build_button(component, |btn| {
            btn.text(text)
                .width_px(width)
                .height_px(height)
                .bg_color(bg_color)
                .font_size(font_size)
                .border_radius_all_px(border_radius);
        })
    }

    /// Add a button with common properties and multiple components via a bundle
    pub fn add_button_with<B: Bundle>(
        &mut self,
        text: impl Into<String>,
        width: f32,
        height: f32,
        bg_color: Color,
        font_size: f32,
        border_radius: f32,
        bundle: B,
    ) -> &mut Self {
        let text_str = text.into();
        let original_entity = self.current_entity;
        let original_stack_len = self.parent_stack.len();

        // Create button child with defaults
        self.child();
        let button_entity = self.current_entity;

        // Get default button definition
        let btn_def = self.defaults.get_btn_def(None);

        // Insert button components with overrides
        let mut node = btn_def.get_node();
        node.width = Val::Px(width);
        node.height = Val::Px(height);
        node.border_radius = BorderRadius::all(Val::Px(border_radius));
        self.commands
            .entity(button_entity)
            .insert(node)
            .insert(btn_def.get_button_bundle())
            .insert(BackgroundColor(bg_color))
            .insert(bundle);

        // Create text child entity with overrides
        let text_bundle = TextBundle {
            text: Text(text_str),
            text_font: TextFont::default()
                .with_font(btn_def.font.clone())
                .with_font_size(font_size),
            text_color: btn_def.get_text_color(),
        };
        let text_entity = self.commands.spawn(text_bundle).id();
        self.commands.entity(button_entity).add_child(text_entity);

        // Restore state
        self.current_entity = original_entity;
        while self.parent_stack.len() > original_stack_len {
            self.parent_stack.pop_back();
        }

        self
    }

    /// Build a button using a closure with ButtonBuilder
    pub fn build_button<T, F>(&mut self, component: T, f: F) -> &mut Self
    where
        T: Component,
        F: FnOnce(&mut ButtonBuilder),
    {
        let original_entity = self.current_entity;
        let original_stack_len = self.parent_stack.len();

        // Create button child with defaults
        self.child();
        let button_entity = self.current_entity;

        // Get default button definition
        let btn_def = self.defaults.get_btn_def(None);

        // Insert button components
        let mut node = btn_def.get_node();
        node.border_radius = btn_def.get_border_radius();
        self.commands
            .entity(button_entity)
            .insert(node)
            .insert(btn_def.get_button_bundle())
            .insert(component);

        // Create text child entity
        let text_entity = self.commands
            .spawn(btn_def.get_text_bundle())
            .id();
        self.commands.entity(button_entity).add_child(text_entity);

        // Create ButtonBuilder and call closure
        let mut button_builder = ButtonBuilder {
            ui: self,
            text_entity: Some(text_entity),
        };

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            f(&mut button_builder);
        }));

        // Restore state
        self.current_entity = original_entity;
        while self.parent_stack.len() > original_stack_len {
            self.parent_stack.pop_back();
        }

        if let Err(panic_payload) = result {
            std::panic::resume_unwind(panic_payload);
        }

        self
    }
}

impl<'a, 'w, 's> ButtonBuilder<'a, 'w, 's> {
    /// Set the button text
    pub fn text(&mut self, text: impl Into<String>) -> &mut Self {
        if let Some(text_entity) = self.text_entity {
            let text_str = text.into();
            self.ui.commands
                .entity(text_entity)
                .entry::<Text>()
                .and_modify(move |mut t| t.0 = text_str);
        }
        self
    }

    /// Set the button text font size
    pub fn font_size(&mut self, size: f32) -> &mut Self {
        if let Some(text_entity) = self.text_entity {
            self.ui.commands
                .entity(text_entity)
                .entry::<TextFont>()
                .and_modify(move |mut tf| tf.font_size = size);
        }
        self
    }

    /// Set the button text color
    pub fn text_color(&mut self, color: Color) -> &mut Self {
        if let Some(text_entity) = self.text_entity {
            self.ui.commands
                .entity(text_entity)
                .entry::<TextColor>()
                .and_modify(move |mut tc| *tc = TextColor(color));
        }
        self
    }

    /// Set the button width in pixels
    pub fn width_px(&mut self, width: f32) -> &mut Self {
        self.ui.width_px(width);
        self
    }

    /// Set the button height in pixels
    pub fn height_px(&mut self, height: f32) -> &mut Self {
        self.ui.height_px(height);
        self
    }

    /// Set the button size in pixels
    pub fn size_px(&mut self, width: f32, height: f32) -> &mut Self {
        self.ui.size_px(width, height);
        self
    }

    /// Set the button background color
    pub fn bg_color(&mut self, color: Color) -> &mut Self {
        self.ui.bg_color(color);
        self
    }

    /// Set the button background color using SRGBA values
    pub fn bg_color_srgba(&mut self, r: f32, g: f32, b: f32, a: f32) -> &mut Self {
        self.ui.bg_color_srgba(r, g, b, a);
        self
    }

    /// Set border radius on all corners in pixels
    pub fn border_radius_all_px(&mut self, value: f32) -> &mut Self {
        self.ui.border_radius_all_px(value);
        self
    }

    /// Set border radius to zero
    pub fn border_radius_zero(&mut self) -> &mut Self {
        self.ui.border_radius_zero();
        self
    }

    /// Set margin on all sides in pixels
    pub fn margin_all_px(&mut self, value: f32) -> &mut Self {
        self.ui.margin_all_px(value);
        self
    }

    /// Set margin to zero
    pub fn margin_zero(&mut self) -> &mut Self {
        self.ui.margin_zero();
        self
    }

    /// Set padding on all sides in pixels
    pub fn padding_all_px(&mut self, value: f32) -> &mut Self {
        self.ui.padding_all_px(value);
        self
    }

    /// Set padding to zero
    pub fn padding_zero(&mut self) -> &mut Self {
        self.ui.padding_zero();
        self
    }

    /// Insert a component on the button entity
    pub fn insert<T: Component>(&mut self, component: T) -> &mut Self {
        self.ui.insert(component);
        self
    }
}

// ============================================================================
// Feathers-style button helpers using observe() pattern
// ============================================================================

impl<'w, 's> UIBuilder<'w, 's> {
    /// Add a Feathers-style button with an observer for the Activate event.
    /// 
    /// # Example
    /// ```ignore
    /// ui.feathers_button("Click Me", |_: On<Activate>, mut commands: Commands| {
    ///     info!("Button clicked!");
    /// });
    /// ```
    pub fn feathers_button<M>(
        &mut self,
        text: impl Into<String>,
        handler: impl IntoObserverSystem<Activate, (), M>,
    ) -> &mut Self {
        self.feathers_button_with_props(text, ButtonProps::default(), (), handler)
    }

    /// Add a Feathers-style button with custom props and an observer.
    /// 
    /// # Example
    /// ```ignore
    /// ui.feathers_button_with_props(
    ///     "Primary",
    ///     ButtonProps { variant: ButtonVariant::Primary, ..default() },
    ///     (),
    ///     |_: On<Activate>| { info!("Clicked!"); }
    /// );
    /// ```
    pub fn feathers_button_with_props<B, M>(
        &mut self,
        text: impl Into<String>,
        props: ButtonProps,
        extra_components: B,
        handler: impl IntoObserverSystem<Activate, (), M>,
    ) -> &mut Self
    where
        B: Bundle,
    {
        let text_str = text.into();
        let original_entity = self.current_entity;
        let original_stack_len = self.parent_stack.len();

        self.child();
        let button_entity = self.current_entity;

        // Spawn the Feathers button bundle with observer
        let button_bundle = button(
            props,
            extra_components,
            Spawn((Text::new(text_str), ThemedText)),
        );

        self.commands
            .entity(button_entity)
            .insert(button_bundle)
            .observe(handler);

        // Restore state
        self.current_entity = original_entity;
        while self.parent_stack.len() > original_stack_len {
            self.parent_stack.pop_back();
        }

        self
    }

    /// Add a Feathers-style primary button with an observer.
    pub fn feathers_button_primary<M>(
        &mut self,
        text: impl Into<String>,
        handler: impl IntoObserverSystem<Activate, (), M>,
    ) -> &mut Self {
        self.feathers_button_with_props(
            text,
            ButtonProps {
                variant: ButtonVariant::Primary,
                ..default()
            },
            (),
            handler,
        )
    }

    /// Add a disabled Feathers-style button with an observer.
    pub fn feathers_button_disabled<M>(
        &mut self,
        text: impl Into<String>,
        handler: impl IntoObserverSystem<Activate, (), M>,
    ) -> &mut Self {
        self.feathers_button_with_props(text, ButtonProps::default(), InteractionDisabled, handler)
    }

    /// Add a Feathers-style button with a marker component and an observer.
    /// 
    /// # Example
    /// ```ignore
    /// ui.feathers_button_marked("Save", SaveButton, |_: On<Activate>| {
    ///     info!("Save clicked!");
    /// });
    /// ```
    pub fn feathers_button_marked<T, M>(
        &mut self,
        text: impl Into<String>,
        marker: T,
        handler: impl IntoObserverSystem<Activate, (), M>,
    ) -> &mut Self
    where
        T: Component,
    {
        self.feathers_button_with_props(text, ButtonProps::default(), marker, handler)
    }

    /// Add a Feathers-style button with custom rounded corners.
    pub fn feathers_button_corners<M>(
        &mut self,
        text: impl Into<String>,
        corners: RoundedCorners,
        handler: impl IntoObserverSystem<Activate, (), M>,
    ) -> &mut Self {
        self.feathers_button_with_props(
            text,
            ButtonProps {
                corners,
                ..default()
            },
            (),
            handler,
        )
    }
}

// Re-export Feathers types for convenience
pub use bevy::feathers::controls::button as feathers_button_bundle;
