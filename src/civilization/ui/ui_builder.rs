use bevy::color::palettes::basic::{BLACK, WHITE};
use bevy::prelude::*;
use bevy::reflect::Enum;
use std::collections::VecDeque;

pub const BG_COLOR: Color = Color::srgba(0.5, 0.5, 0.5, 0.25);
pub const CARD_COLOR: Color = Color::srgba(0.7, 0.6, 0.2, 0.8);
pub const TEXT_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 1.0);
pub const BORDER_COLOR: Color = Color::srgba(0.2, 0.2, 0.2, 0.8);

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
        BorderColor(self.border_color)
    }

    pub fn get_text(&self) -> Text {
        Text(self.text.clone())
    }

    pub fn get_text_font(&self) -> TextFont {
        TextFont::from_font(self.font.clone_weak()).with_font_size(self.font_size)
    }

    pub fn get_text_color(&self) -> TextColor {
        TextColor::from(self.text_color)
    }

    pub fn get_button_bundle(&self) -> (Button, BackgroundColor, BorderColor, BorderRadius) {
        (Button, self.get_background_color(), self.get_border_color(), self.get_border_radius())
    }

    pub fn get_text_bundle(&self) -> (Text, TextFont, TextColor) {
        (self.get_text(), self.get_text_font(), self.get_text_color())
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
}

impl Default for NodeDef {
    fn default() -> Self {
        Self {
            display: Display::Flex,
            position_type: PositionType::Relative,
            overflow: Overflow::DEFAULT,
            overflow_clip_margin: OverflowClipMargin::default(),
            left: Val::Auto,
            right: Val::Auto,
            top: Val::Auto,
            bottom: Val::Auto,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            min_width: Val::Auto,
            min_height: Val::Auto,
            max_width: Val::Auto,
            max_height: Val::Auto,
            aspect_ratio: None,
            align_items: AlignItems::Start,
            justify_items: JustifyItems::Start,
            align_self: AlignSelf::Auto,
            justify_self: JustifySelf::Auto,
            align_content: AlignContent::Default,
            justify_content: JustifyContent::Default,
            margin: UiRect::all(Val::Auto),
            padding: UiRect::all(Val::Auto),
            border: UiRect::all(Val::Auto),
            flex_direction: FlexDirection::Column,
            flex_wrap: FlexWrap::Wrap,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: Val::Auto,
            row_gap: Val::Auto,
            column_gap: Val::Auto,
            grid_auto_flow: GridAutoFlow::Column,
            grid_template_rows: vec![],
            grid_template_columns: vec![],
            grid_auto_rows: vec![],
            grid_auto_columns: vec![],
            grid_row: GridPlacement::DEFAULT,
            grid_column: GridPlacement::DEFAULT,
            bg_color: Color::from(BG_COLOR),
            border_color: Color::from(BLACK),
            border_radius: BorderRadius::ZERO,
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
        }
    }
    pub fn get_background_color(&self) -> BackgroundColor {
        BackgroundColor(self.bg_color)
    }

    pub fn get_border_radius(&self) -> BorderRadius {
        self.border_radius
    }

    pub fn get_border_color(&self) -> BorderColor {
        BorderColor(self.border_color)
    }

    pub fn get_node_bundle(&self) -> (Node, BackgroundColor, BorderColor, BorderRadius) {
        (self.get_node(), self.get_background_color(), self.get_border_color(), self.get_border_radius())
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

#[derive(Resource, Default, Clone, Debug)]
pub struct UiBuilderDefaults {
    pub node_def: Option<NodePartial>,
    pub button_def: Option<ButtonPartial>,
    pub base_font: Handle<Font>,
    pub font_size: f32,
    pub bg_color: Color,
    pub border_color: Color,
    pub text_color: Color,
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
                def.font = (input
                    .font
                    .unwrap_or(internal.font.clone().unwrap_or(def.font)))
                .clone_weak();
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
                def.font_size = input.font_size.unwrap_or(
                    internal
                        .font_size
                        .unwrap_or(self.font_size),
                );
                def.bg_color = input.bg_color.unwrap_or(internal.bg_color.unwrap_or(self.bg_color));
                def.text_color = input.text_color.unwrap_or(
                    internal.text_color.unwrap_or(
                        self.text_color));
                def.border_color = input.border_color.unwrap_or(
                    internal.border_color.unwrap_or(
                        self.border_color));
            }
            (None, Some(input)) => {
                def.text = input.text.unwrap_or(def.text);
                def.font = (input.font.unwrap_or(def.font)).clone_weak();
                def.width = input.width.unwrap_or(def.width);
                def.height = input.height.unwrap_or(def.height);
                def.border = input.border.unwrap_or(def.border);
                def.justify_content = input.justify_content.unwrap_or(def.justify_content);
                def.align_items = input.align_items.unwrap_or(def.align_items);
                def.border_color = input.border_color.unwrap_or(def.border_color);
                def.border_radius = (input.border_radius.unwrap_or(def.border_radius)).clone();
                def.bg_color = input.bg_color.unwrap_or(def.bg_color);
                def.font_size = input.font_size.unwrap_or(def.font_size);
                def.text_color = input.text_color.unwrap_or(def.text_color);
            }
            (Some(internal), None) => {
                def.text = internal.text.clone().unwrap_or(def.text);
                def.font = (internal.font.clone().unwrap_or(def.font)).clone_weak();
                def.width = internal.width.unwrap_or(def.width);
                def.height = internal.height.unwrap_or(def.height);
                def.border = internal.border.unwrap_or(def.border);
                def.justify_content = internal.justify_content.unwrap_or(def.justify_content);
                def.align_items = internal.align_items.unwrap_or(def.align_items);
                def.border_color = internal.border_color.unwrap_or(def.border_color);
                def.border_radius = (internal.border_radius.unwrap_or(def.border_radius)).clone();
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
                def.bg_color = input.bg_color.unwrap_or(
                    internal
                        .bg_color
                        .unwrap_or(self.bg_color),
                );
                def.border_radius = input
                    .border_radius
                    .unwrap_or(internal.border_radius.unwrap_or(def.border_radius));
                def.border_color = input.border_color.unwrap_or(
                    internal
                        .border_color
                        .unwrap_or(self.border_color),
                );
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
                def.bg_color = input
                    .bg_color
                    .unwrap_or(self.bg_color);
                def.border_radius = input.border_radius.unwrap_or(def.border_radius);
                def.border_color = input
                    .border_color
                    .unwrap_or(self.border_color);
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
                def.bg_color = internal
                    .bg_color
                    .unwrap_or(self.bg_color);
                def.border_radius = internal.border_radius.unwrap_or(def.border_radius);
                def.border_color = internal
                    .border_color
                    .unwrap_or(self.border_color);
            }
            (None, None) => {}
        }
        def
    }

    pub fn get_node_bundle(
        &self,
        input: Option<NodePartial>,
    ) -> (Node, BackgroundColor, BorderColor, BorderRadius) {
        self.get_node_def(input).get_node_bundle()
    }


    pub fn get_btn_node(
        &self,
        input: Option<ButtonPartial>,
    ) -> Node {
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
    ) -> (Button, BackgroundColor, BorderColor, BorderRadius) {
        self.get_btn_def(input).get_button_bundle()
    }

    pub fn get_btn_text_bundle(
        &self,
        input: Option<ButtonPartial>,
    ) -> (Text, TextFont, TextColor) {
        self.get_btn_def(input).get_text_bundle()
    }
}

impl<'w, 's> UIBuilder<'w, 's> {
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
            commands.entity(entity).despawn_descendants();
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
        button_def: Option<ButtonPartial>,
        component: T,
    ) -> &mut Self {
        let internal = self.child();
        internal
            .commands
            .entity(internal.current_entity)
            .insert(internal.defaults.get_btn_node(button_def.clone()))
            .insert(internal.defaults.get_btn_bundle(button_def.clone()))
            .insert(component)
            .with_child(internal.defaults.get_btn_text_bundle(button_def));
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
    pub fn with_display(&mut self, display: Display) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.display = display;
            });
        self
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
    pub fn with_aspect_ratio(&mut self, ratio: f32) -> &mut Self {
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

    /// Set the flex wrap property of the current entity's node
    pub fn with_flex_wrap(&mut self, wrap: FlexWrap) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.flex_wrap = wrap;
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

    /// Set the justify items property of the current entity's node
    pub fn with_justify_items(&mut self, justify: JustifyItems) -> &mut Self {
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
    pub fn with_align_content(&mut self, align: AlignContent) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.align_content = align;
            });
        self
    }

    /// Set the justify content property of the current entity's node
    pub fn with_justify_content(&mut self, justify: JustifyContent) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.justify_content = justify;
            });
        self
    }

    /// Set the row gap property of the current entity's node
    pub fn with_row_gap(&mut self, gap: Val) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.row_gap = gap;
            });
        self
    }

    /// Set the column gap property of the current entity's node
    pub fn with_column_gap(&mut self, gap: Val) -> &mut Self {
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

    pub fn with_children<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut Self), // Closure gets mutable borrow to allow multiple calls
    {
        let original_entity = self.current_entity;
        self.parent_stack.push_back(original_entity);

        // Use catch_unwind to ensure stack cleanup even on panic
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            f(self); // Call the closure
        }));

        // --- Cleanup ---
        // Pop the stack regardless of panic
        let popped = self.parent_stack.pop_back();
        // Restore the original entity focus BEFORE checking the popped value or resuming panic
        self.current_entity = original_entity;

        // Verify correct parent was popped after restoring focus
        if popped != Some(original_entity) {
            // Log error or panic, stack is potentially corrupted
            eprintln!("UIBuilder Internal Error: Stack mismatch on exiting with_children scope.");
            // Depending on desired robustness, might panic here
        }

        // If a panic occurred within the closure, resume unwinding
        if let Err(panic_payload) = result {
            std::panic::resume_unwind(panic_payload);
        }

        // Return the builder, now focused on the original entity
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
            .or_default();
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


    pub fn as_flex_col(
        &mut self,
        width: Val,
        height: Val,
    ) -> &mut Self {
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

    pub fn with_size(&mut self, width: Val, height: Val) -> &mut Self {
        self.width(width).height(height)
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

    /// Set justify content
    pub fn justify_content(&mut self, justify: JustifyContent) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.justify_content = justify);
        self
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
    pub fn with_padding(&mut self, padding: UiRect) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.padding = padding);
        self
    }

    /// Set margin
    pub fn with_margin(&mut self, margin: UiRect) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.margin = margin);
        self
    }

    pub fn with_border(&mut self, border: UiRect, border_color: Color) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.border = border);

        self.commands
            .entity(self.current_entity)
            .entry::<BorderColor>()
            .and_modify(move |mut b_color| *b_color = BorderColor(border_color))
            .or_insert(BorderColor(border_color));
        self
    }

    pub fn with_box_shadow(&mut self, offset: Vec2, spread: f32, blur: f32) -> &mut Self {
        self.commands.entity(self.current_entity).insert(BoxShadow {
            color: Color::BLACK.with_alpha(0.8),
            x_offset: Val::Percent(offset.x),
            y_offset: Val::Percent(offset.y),
            spread_radius: Val::Percent(spread),
            blur_radius: Val::Px(blur),
        });
        self
    }

    /// Apply a complete Node
    pub fn node(mut self, node: Node) -> Self {
        self.commands.entity(self.current_entity).insert(node);
        self
    }

    /// Add background color Component
    pub fn with_bg_color(&mut self, color: Color) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<BackgroundColor>()
            .and_modify(move |mut background| *background = BackgroundColor(color))
            .or_insert(BackgroundColor(color));
        self
    }

    /// Add a text child Entity
    pub fn add_text_child(
        &mut self,
        text: impl Into<String>,
        font: Option<Handle<Font>>,
        font_size: Option<f32>,
        color: Option<Color>,
    ) -> &mut Self {
        self.child()
            .with_text(text, font, font_size, color)
            .parent()
    }
    
    pub fn add_default_text_child(&mut self, text: impl Into<String>) -> &mut Self {
        self.add_text_child(text, None, None, None)
    }
    
    pub fn with_default_text(&mut self, text: impl Into<String>) -> &mut Self {
        self.with_text(text, None, None, None)
    }
    
    pub fn with_text(
        &mut self,
        text: impl Into<String>,
        font: Option<Handle<Font>>,
        font_size: Option<f32>,
        color: Option<Color>,
    ) -> &mut Self {
        let text_color = color.unwrap_or(self.defaults.text_color);

        let text_bundle = (
            Text::new(text.into()),
            TextFont::from_font(font.unwrap_or(self.defaults.base_font.clone_weak())).with_font_size(font_size.unwrap_or(self.defaults.font_size)),
            TextColor(text_color),
        );

        // Add the text component to the entity
        self.commands
            .entity(self.current_entity)
            .insert(text_bundle);
        self
    }

    /// Create a child container
    pub fn child(&mut self) -> &mut Self {
        // Push current entity to parent stack
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
}
