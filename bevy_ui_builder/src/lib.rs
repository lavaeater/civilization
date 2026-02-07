use bevy::color::palettes::basic::{BLACK, WHITE};
use bevy::prelude::*;
use bevy::reflect::Enum;
use std::collections::VecDeque;

mod builder;
mod button_builder;

// Re-export feathers types for external use
pub use bevy::feathers::controls::{button, ButtonProps, ButtonVariant};
pub use bevy::feathers::rounded_corners::RoundedCorners;
pub use bevy::feathers::theme::ThemedText;
pub use bevy::ui::InteractionDisabled;
pub use bevy::ui_widgets::Activate;

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
            overflow: Overflow::clip(),
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

// Re-export Feathers types for convenience
pub use bevy::feathers::controls::button as feathers_button_bundle;

/// Marker component for a collapsible UI section.
/// When collapsed, the content is hidden and only the toggle button is shown.
#[derive(Component)]
pub struct Collapsible {
    pub collapsed: bool,
    pub label: String,
}

impl Collapsible {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            collapsed: false,
            label: label.into(),
        }
    }

    pub fn collapsed(label: impl Into<String>) -> Self {
        Self {
            collapsed: true,
            label: label.into(),
        }
    }
}

/// Marker for the toggle button that collapses/expands a Collapsible section
#[derive(Component)]
pub struct CollapseToggleButton {
    pub target: Entity,
}

/// Marker for the content container inside a Collapsible section
#[derive(Component)]
pub struct CollapsibleContent {
    pub parent: Entity,
}
