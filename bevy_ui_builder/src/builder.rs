use bevy::prelude::*;
use bevy::text::{Justify, LineBreak, TextLayout};
use std::collections::VecDeque;

use crate::{
    ButtonBuilder, ButtonPartial, Collapsible, CollapseToggleButton,
    CollapsibleContent, NodePartial, TextBundle, UiBuilderDefaults, UIBuilder,
};

impl<'w, 's> UIBuilder<'w, 's> {
    /// Get a reference to the UI builder defaults
    pub fn get_defaults(&self) -> &UiBuilderDefaults {
        &self.defaults
    }

    /// Create a new root UI container
    pub fn new(mut commands: Commands<'w, 's>, defaults: Option<UiBuilderDefaults>) -> Self {
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

    pub fn with(&mut self, partial: NodePartial) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                if let Some(display) = partial.display { n.display = display; }
                if let Some(position_type) = partial.position_type { n.position_type = position_type; }
                if let Some(overflow) = partial.overflow { n.overflow = overflow; }
                if let Some(overflow_clip_margin) = partial.overflow_clip_margin { n.overflow_clip_margin = overflow_clip_margin; }
                if let Some(left) = partial.left { n.left = left; }
                if let Some(right) = partial.right { n.right = right; }
                if let Some(top) = partial.top { n.top = top; }
                if let Some(bottom) = partial.bottom { n.bottom = bottom; }
                if let Some(width) = partial.width { n.width = width; }
                if let Some(height) = partial.height { n.height = height; }
                if let Some(min_width) = partial.min_width { n.min_width = min_width; }
                if let Some(min_height) = partial.min_height { n.min_height = min_height; }
                if let Some(max_width) = partial.max_width { n.max_width = max_width; }
                if let Some(max_height) = partial.max_height { n.max_height = max_height; }
                if let Some(aspect_ratio) = partial.aspect_ratio { n.aspect_ratio = Some(aspect_ratio); }
                if let Some(align_items) = partial.align_items { n.align_items = align_items; }
                if let Some(justify_items) = partial.justify_items { n.justify_items = justify_items; }
                if let Some(align_self) = partial.align_self { n.align_self = align_self; }
                if let Some(justify_self) = partial.justify_self { n.justify_self = justify_self; }
                if let Some(align_content) = partial.align_content { n.align_content = align_content; }
                if let Some(justify_content) = partial.justify_content { n.justify_content = justify_content; }
                if let Some(margin) = partial.margin { n.margin = margin; }
                if let Some(padding) = partial.padding { n.padding = padding; }
                if let Some(border) = partial.border { n.border = border; }
                if let Some(flex_direction) = partial.flex_direction { n.flex_direction = flex_direction; }
                if let Some(flex_wrap) = partial.flex_wrap { n.flex_wrap = flex_wrap; }
                if let Some(flex_grow) = partial.flex_grow { n.flex_grow = flex_grow; }
                if let Some(flex_shrink) = partial.flex_shrink { n.flex_shrink = flex_shrink; }
                if let Some(flex_basis) = partial.flex_basis { n.flex_basis = flex_basis; }
                if let Some(row_gap) = partial.row_gap { n.row_gap = row_gap; }
                if let Some(column_gap) = partial.column_gap { n.column_gap = column_gap; }
                if let Some(grid_auto_flow) = partial.grid_auto_flow { n.grid_auto_flow = grid_auto_flow; }
                if let Some(grid_template_rows) = partial.grid_template_rows { n.grid_template_rows = grid_template_rows; }
                if let Some(grid_template_columns) = partial.grid_template_columns { n.grid_template_columns = grid_template_columns; }
                if let Some(grid_auto_rows) = partial.grid_auto_rows { n.grid_auto_rows = grid_auto_rows; }
                if let Some(grid_auto_columns) = partial.grid_auto_columns { n.grid_auto_columns = grid_auto_columns; }
                if let Some(grid_row) = partial.grid_row { n.grid_row = grid_row; }
                if let Some(grid_column) = partial.grid_column { n.grid_column = grid_column; }
            });
        self
    }

    pub fn display(&mut self, display: Display) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.display = display; });
        self
    }

    pub fn display_flex(&mut self) -> &mut Self { self.display(Display::Flex) }
    pub fn display_grid(&mut self) -> &mut Self { self.display(Display::Grid) }
    pub fn display_block(&mut self) -> &mut Self { self.display(Display::Block) }
    pub fn display_none(&mut self) -> &mut Self { self.display(Display::None) }

    pub fn grid_cols(&mut self, count: u16) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.grid_template_columns = vec![RepeatedGridTrack::fr(count, 1.0)]; });
        self
    }

    pub fn grid_rows(&mut self, count: u16) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.grid_template_rows = vec![RepeatedGridTrack::fr(count, 1.0)]; });
        self
    }

    pub fn grid_cols_px(&mut self, count: u16, width: f32) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.grid_template_columns = vec![RepeatedGridTrack::px(count, width)]; });
        self
    }

    pub fn grid_cols_auto_fill_percent(&mut self, min_width: f32) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| {
                n.grid_template_columns = vec![RepeatedGridTrack::minmax(
                    GridTrackRepetition::AutoFill,
                    MinTrackSizingFunction::Percent(min_width),
                    MaxTrackSizingFunction::Fraction(1.0),
                )];
            });
        self
    }

    pub fn grid_cols_auto_fill_px(&mut self, min_width: f32) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| {
                n.grid_template_columns = vec![RepeatedGridTrack::minmax(
                    GridTrackRepetition::AutoFill,
                    MinTrackSizingFunction::Px(min_width),
                    MaxTrackSizingFunction::Fraction(1.0),
                )];
            });
        self
    }

    pub fn grid_gap_px(&mut self, gap: f32) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.row_gap = Val::Px(gap); n.column_gap = Val::Px(gap); });
        self
    }

    pub fn grid_gap_percent(&mut self, gap: f32) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.row_gap = Val::Percent(gap); n.column_gap = Val::Percent(gap); });
        self
    }

    pub fn with_position_type(&mut self, position_type: PositionType) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.position_type = position_type; });
        self
    }

    pub fn with_overflow(&mut self, overflow: Overflow) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.overflow = overflow; });
        self
    }

    pub fn with_overflow_clip_margin(&mut self, margin: OverflowClipMargin) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.overflow_clip_margin = margin; });
        self
    }

    pub fn aspect_ratio(&mut self, ratio: f32) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.aspect_ratio = Some(ratio); });
        self
    }

    pub fn clear_aspect_ratio(&mut self) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.aspect_ratio = None; });
        self
    }

    pub fn with_left(&mut self, left: Val) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.left = left; });
        self
    }

    pub fn with_right(&mut self, right: Val) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.right = right; });
        self
    }

    pub fn with_top(&mut self, top: Val) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.top = top; });
        self
    }

    pub fn with_bottom(&mut self, bottom: Val) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.bottom = bottom; });
        self
    }

    pub fn with_position(&mut self, left: Val, right: Val, top: Val, bottom: Val) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.left = left; n.right = right; n.top = top; n.bottom = bottom; });
        self
    }

    pub fn with_flex_direction(&mut self, direction: FlexDirection) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.flex_direction = direction; });
        self
    }

    pub fn flex_dir_row(&mut self) -> &mut Self { self.with_flex_direction(FlexDirection::Row) }
    pub fn flex_dir_column(&mut self) -> &mut Self { self.with_flex_direction(FlexDirection::Column) }
    pub fn flex_dir_row_reverse(&mut self) -> &mut Self { self.with_flex_direction(FlexDirection::RowReverse) }
    pub fn flex_dir_column_reverse(&mut self) -> &mut Self { self.with_flex_direction(FlexDirection::ColumnReverse) }

    pub fn flex_wrap(&mut self) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.flex_wrap = FlexWrap::Wrap; });
        self
    }

    pub fn flex_wrap_none(&mut self) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.flex_wrap = FlexWrap::NoWrap; });
        self
    }

    pub fn flex_wrap_reverse(&mut self) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.flex_wrap = FlexWrap::WrapReverse; });
        self
    }

    pub fn with_flex_grow(&mut self, grow: f32) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.flex_grow = grow; });
        self
    }

    pub fn with_flex_shrink(&mut self, shrink: f32) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.flex_shrink = shrink; });
        self
    }

    pub fn with_flex_basis(&mut self, basis: Val) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.flex_basis = basis; });
        self
    }

    pub fn with_align_items(&mut self, align: AlignItems) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.align_items = align; });
        self
    }

    pub fn align_items_default(&mut self) -> &mut Self { self.with_align_items(AlignItems::Default) }
    pub fn align_items_start(&mut self) -> &mut Self { self.with_align_items(AlignItems::FlexStart) }
    pub fn align_items_end(&mut self) -> &mut Self { self.with_align_items(AlignItems::FlexEnd) }
    pub fn align_items_center(&mut self) -> &mut Self { self.with_align_items(AlignItems::Center) }
    pub fn align_items_baseline(&mut self) -> &mut Self { self.with_align_items(AlignItems::Baseline) }
    pub fn align_items_stretch(&mut self) -> &mut Self { self.with_align_items(AlignItems::Stretch) }

    pub fn justify_items(&mut self, justify: JustifyItems) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.justify_items = justify; });
        self
    }

    pub fn with_align_self(&mut self, align: AlignSelf) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.align_self = align; });
        self
    }

    pub fn with_justify_self(&mut self, justify: JustifySelf) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.justify_self = justify; });
        self
    }

    pub fn align_content(&mut self, align: AlignContent) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.align_content = align; });
        self
    }

    pub fn justify_content(&mut self, justify: JustifyContent) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.justify_content = justify; });
        self
    }

    pub fn justify_start(&mut self) -> &mut Self { self.justify_content(JustifyContent::FlexStart) }
    pub fn justify_end(&mut self) -> &mut Self { self.justify_content(JustifyContent::FlexEnd) }
    pub fn justify_center(&mut self) -> &mut Self { self.justify_content(JustifyContent::Center) }
    pub fn justify_space_between(&mut self) -> &mut Self { self.justify_content(JustifyContent::SpaceBetween) }
    pub fn justify_space_around(&mut self) -> &mut Self { self.justify_content(JustifyContent::SpaceAround) }
    pub fn justify_space_evenly(&mut self) -> &mut Self { self.justify_content(JustifyContent::SpaceEvenly) }

    pub fn row_gap(&mut self, gap: Val) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.row_gap = gap; });
        self
    }

    pub fn row_gap_px(&mut self, gap: f32) -> &mut Self { self.row_gap(Val::Px(gap)) }
    pub fn row_gap_percent(&mut self, gap: f32) -> &mut Self { self.row_gap(Val::Percent(gap)) }

    pub fn column_gap(&mut self, gap: Val) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.column_gap = gap; });
        self
    }

    pub fn column_gap_px(&mut self, gap: f32) -> &mut Self { self.column_gap(Val::Px(gap)) }
    pub fn column_gap_percent(&mut self, gap: f32) -> &mut Self { self.column_gap(Val::Percent(gap)) }

    pub fn with_gap(&mut self, row_gap: Val, column_gap: Val) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.row_gap = row_gap; n.column_gap = column_gap; });
        self
    }

    pub fn with_width(&mut self, width: Val) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.width = width; });
        self
    }

    pub fn with_height(&mut self, height: Val) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.height = height; });
        self
    }

    pub fn with_min_width(&mut self, min_width: Val) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.min_width = min_width; });
        self
    }

    pub fn with_min_height(&mut self, min_height: Val) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.min_height = min_height; });
        self
    }

    pub fn with_max_width(&mut self, max_width: Val) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.max_width = max_width; });
        self
    }

    pub fn with_max_height(&mut self, max_height: Val) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.max_height = max_height; });
        self
    }

    pub fn with_child<F>(&mut self, f: F) -> &mut Self
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

    pub fn add_row<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        self.with_child(|ui| {
            ui.as_flex_row();
            f(ui);
        })
    }

    pub fn add_column<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        self.with_child(|ui| {
            ui.display_flex().flex_dir_column();
            f(ui);
        })
    }

    pub fn add_panel<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        self.with_child(|ui| {
            ui.display_flex().flex_dir_column().padding_all_px(16.0);
            f(ui);
        })
    }

    pub fn add_grid<F>(&mut self, cols: u16, f: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        self.with_child(|ui| {
            ui.display_grid().grid_cols(cols).grid_gap_percent(2.0);
            f(ui);
        })
    }

    pub fn add_card<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        self.with_child(|ui| {
            ui.display_flex()
                .flex_dir_column()
                .padding_all_px(4.0)
                .border_radius_all_px(4.0);
            f(ui);
        })
    }

    pub fn add_centered<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        self.with_child(|ui| {
            ui.display_flex().justify_center().align_items_center();
            f(ui);
        })
    }

    pub fn with_collapsible<F>(&mut self, label: &str, initially_collapsed: bool, f: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        let label_owned = label.to_string();
        let collapsed = initially_collapsed;
        
        self.with_child(|ui| {
            let collapsible_entity = ui.current_entity;
            ui.insert(if collapsed {
                Collapsible::collapsed(&label_owned)
            } else {
                Collapsible::new(&label_owned)
            });
            ui.display_flex().flex_dir_column();
            
            ui.with_child(|btn| {
                btn.insert(Button);
                btn.insert(CollapseToggleButton { target: collapsible_entity });
                btn.padding_all_px(8.0)
                    .margin(UiRect::bottom(Val::Px(4.0)))
                    .bg_color(Color::srgb(0.25, 0.25, 0.3));
                
                let arrow = if collapsed { "▶" } else { "▼" };
                btn.add_text_child(format!("{} {}", arrow, label_owned), None, Some(12.0), None);
            });
            
            ui.with_child(|content| {
                content.insert(CollapsibleContent { parent: collapsible_entity });
                content.display_flex().flex_dir_column().width_percent(100.0);
                
                if collapsed {
                    content.display(Display::None);
                }
                
                f(content);
            });
        })
    }

    pub fn add_collapsible<F>(&mut self, label: &str, f: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        self.with_collapsible(label, false, f)
    }

    pub fn add_collapsible_collapsed<F>(&mut self, label: &str, f: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        self.with_collapsible(label, true, f)
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
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut node| {
                node.position_type = position_type;
                node.left = left;
                node.top = top;
            });
        self
    }

    pub fn with_component<T: Component + Default>(&mut self) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<T>().or_insert(T::default());
        self
    }

    pub fn insert<T: Component>(&mut self, component: T) -> &mut Self {
        self.commands.entity(self.current_entity).insert(component);
        self
    }

    pub fn as_flex_row(&mut self) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut node| {
                node.display = Display::Flex;
                node.flex_direction = FlexDirection::Row;
            });
        self
    }

    pub fn as_flex_col(&mut self, width: Val, height: Val) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
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
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut node| {
                node.width = width;
                node.height = height;
                node.display = Display::Block;
            });

        self.commands.entity(self.current_entity).entry::<BackgroundColor>()
            .and_modify(move |mut bg| { *bg = BackgroundColor(bg_color); })
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

    pub fn width(&mut self, width: Val) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut node| node.width = width);
        self
    }

    pub fn width_px(&mut self, width: f32) -> &mut Self { self.width(Val::Px(width)) }
    pub fn width_percent(&mut self, width: f32) -> &mut Self { self.width(Val::Percent(width)) }
    pub fn width_auto(&mut self) -> &mut Self { self.width(Val::Auto) }

    pub fn height(&mut self, height: Val) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut node| node.height = height);
        self
    }

    pub fn height_px(&mut self, height: f32) -> &mut Self { self.height(Val::Px(height)) }
    pub fn height_percent(&mut self, height: f32) -> &mut Self { self.height(Val::Percent(height)) }
    pub fn height_auto(&mut self) -> &mut Self { self.height(Val::Auto) }

    pub fn flex_direction(&mut self, direction: FlexDirection) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut node| node.flex_direction = direction);
        self
    }

    pub fn flex_direction_row(&mut self) -> &mut Self { self.flex_direction(FlexDirection::Row) }
    pub fn flex_direction_column(&mut self) -> &mut Self { self.flex_direction(FlexDirection::Column) }

    pub fn align_items(&mut self, align: AlignItems) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut node| node.align_items = align);
        self
    }

    pub fn padding(&mut self, padding: UiRect) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut node| node.padding = padding);
        self
    }

    pub fn padding_all_px(&mut self, value: f32) -> &mut Self { self.padding(UiRect::all(Val::Px(value))) }
    pub fn padding_all_percent(&mut self, value: f32) -> &mut Self { self.padding(UiRect::all(Val::Percent(value))) }
    pub fn padding_zero(&mut self) -> &mut Self { self.padding(UiRect::ZERO) }
    pub fn padding_btm_px(&mut self, value: f32) -> &mut Self { self.padding(UiRect::bottom(Val::Px(value))) }
    pub fn padding_btm_percent(&mut self, value: f32) -> &mut Self { self.padding(UiRect::bottom(Val::Percent(value))) }
    pub fn padding_top_px(&mut self, value: f32) -> &mut Self { self.padding(UiRect::top(Val::Px(value))) }
    pub fn padding_top_percent(&mut self, value: f32) -> &mut Self { self.padding(UiRect::top(Val::Percent(value))) }
    pub fn padding_left_px(&mut self, value: f32) -> &mut Self { self.padding(UiRect::left(Val::Px(value))) }
    pub fn padding_left_percent(&mut self, value: f32) -> &mut Self { self.padding(UiRect::left(Val::Percent(value))) }
    pub fn padding_right_px(&mut self, value: f32) -> &mut Self { self.padding(UiRect::right(Val::Px(value))) }
    pub fn padding_right_percent(&mut self, value: f32) -> &mut Self { self.padding(UiRect::right(Val::Percent(value))) }

    pub fn with_padding_btm_px(&mut self, value: f32) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut node| node.padding.bottom = Val::Px(value));
        self
    }

    pub fn with_padding_btm_percent(&mut self, value: f32) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut node| node.padding.bottom = Val::Percent(value));
        self
    }

    pub fn with_padding_top_px(&mut self, value: f32) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut node| node.padding.top = Val::Px(value));
        self
    }

    pub fn with_padding_top_percent(&mut self, value: f32) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut node| node.padding.top = Val::Percent(value));
        self
    }

    pub fn with_padding_left_px(&mut self, value: f32) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut node| node.padding.left = Val::Px(value));
        self
    }

    pub fn with_padding_left_percent(&mut self, value: f32) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut node| node.padding.left = Val::Percent(value));
        self
    }

    pub fn with_padding_right_px(&mut self, value: f32) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut node| node.padding.right = Val::Px(value));
        self
    }

    pub fn with_padding_right_percent(&mut self, value: f32) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut node| node.padding.right = Val::Percent(value));
        self
    }

    pub fn margin(&mut self, margin: UiRect) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut node| node.margin = margin);
        self
    }

    pub fn margin_all_px(&mut self, value: f32) -> &mut Self { self.margin(UiRect::all(Val::Px(value))) }
    pub fn margin_all_percent(&mut self, value: f32) -> &mut Self { self.margin(UiRect::all(Val::Percent(value))) }
    pub fn margin_zero(&mut self) -> &mut Self { self.margin(UiRect::ZERO) }
    pub fn margin_btm_px(&mut self, value: f32) -> &mut Self { self.margin(UiRect::bottom(Val::Px(value))) }
    pub fn margin_btm_percent(&mut self, value: f32) -> &mut Self { self.margin(UiRect::bottom(Val::Percent(value))) }
    pub fn margin_top_px(&mut self, value: f32) -> &mut Self { self.margin(UiRect::top(Val::Px(value))) }
    pub fn margin_top_percent(&mut self, value: f32) -> &mut Self { self.margin(UiRect::top(Val::Percent(value))) }
    pub fn margin_left_px(&mut self, value: f32) -> &mut Self { self.margin(UiRect::left(Val::Px(value))) }
    pub fn margin_left_percent(&mut self, value: f32) -> &mut Self { self.margin(UiRect::left(Val::Percent(value))) }
    pub fn margin_right_px(&mut self, value: f32) -> &mut Self { self.margin(UiRect::right(Val::Px(value))) }
    pub fn margin_right_percent(&mut self, value: f32) -> &mut Self { self.margin(UiRect::right(Val::Percent(value))) }

    pub fn with_margin_btm_px(&mut self, value: f32) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut node| node.margin.bottom = Val::Px(value));
        self
    }

    pub fn with_margin_btm_percent(&mut self, value: f32) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut node| node.margin.bottom = Val::Percent(value));
        self
    }

    pub fn with_margin_top_px(&mut self, value: f32) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut node| node.margin.top = Val::Px(value));
        self
    }

    pub fn with_margin_top_percent(&mut self, value: f32) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut node| node.margin.top = Val::Percent(value));
        self
    }

    pub fn with_margin_left_px(&mut self, value: f32) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut node| node.margin.left = Val::Px(value));
        self
    }

    pub fn with_margin_left_percent(&mut self, value: f32) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut node| node.margin.left = Val::Percent(value));
        self
    }

    pub fn with_margin_right_px(&mut self, value: f32) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut node| node.margin.right = Val::Px(value));
        self
    }

    pub fn with_margin_right_percent(&mut self, value: f32) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut node| node.margin.right = Val::Percent(value));
        self
    }

    pub fn border(&mut self, border: UiRect, border_color: Color) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut node| node.border = border);

        self.commands.entity(self.current_entity).entry::<BorderColor>()
            .and_modify(move |mut b_color| *b_color = BorderColor::all(border_color))
            .or_insert(BorderColor::all(border_color));
        self
    }

    pub fn border_all_px(&mut self, width: f32, border_color: Color) -> &mut Self {
        self.border(UiRect::all(Val::Px(width)), border_color)
    }

    pub fn border_radius_all_px(&mut self, value: f32) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.border_radius = BorderRadius::all(Val::Px(value)); });
        self
    }

    pub fn border_radius_all_percent(&mut self, value: f32) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.border_radius = BorderRadius::all(Val::Percent(value)); });
        self
    }

    pub fn border_radius_zero(&mut self) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.border_radius = BorderRadius::ZERO; });
        self
    }

    pub fn with_box_shadow(&mut self, offset: Vec2, spread: f32, blur: f32) -> &mut Self {
        self.commands.entity(self.current_entity).insert(BoxShadow::new(
            Color::BLACK.with_alpha(0.8),
            Val::Percent(offset.x),
            Val::Percent(offset.y),
            Val::Percent(spread),
            Val::Px(blur),
        ));
        self
    }

    pub fn node(mut self, node: Node) -> Self {
        self.commands.entity(self.current_entity).insert(node);
        self
    }

    pub fn bg_color(&mut self, color: Color) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<BackgroundColor>()
            .and_modify(move |mut background| *background = BackgroundColor(color))
            .or_insert(BackgroundColor(color));
        self
    }

    pub fn bg_color_srgba(&mut self, r: f32, g: f32, b: f32, a: f32) -> &mut Self {
        self.bg_color(Color::srgba(r, g, b, a))
    }

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

    pub fn build_text<F>(&mut self, text: impl Into<String>, f: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        self.with_child(|ui| {
            ui.default_text(text);
            f(ui);
        })
    }

    pub fn text_with_width(&mut self, text: impl Into<String>, width: f32) -> &mut Self {
        self.build_text(text, |ui| {
            ui.width_px(width);
        })
    }

    pub fn add_centered_text(&mut self, text: impl Into<String>, width: f32, component: impl Component) -> &mut Self {
        self.with_child(|ui| {
            ui.width_px(width)
                .display_flex()
                .align_items_center()
                .justify_center();

            ui.build_text(text, |ui| {
                ui
                    .width_px(width)
                    .text_justify_center()
                    .insert(component);
            });
        })
    }

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

    pub fn text_justify(&mut self, justify: Justify) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<TextLayout>()
            .and_modify(move |mut tl| tl.justify = justify)
            .or_insert(TextLayout::new(justify, LineBreak::default()));
        self
    }

    pub fn text_align(&mut self, justify: Justify) -> &mut Self {
        self.commands.entity(self.current_entity).entry::<TextLayout>()
            .and_modify(move |mut tl| tl.justify = justify)
            .or_insert(TextLayout::new(justify, LineBreak::default()));
        self
    }

    pub fn text_justify_center(&mut self) -> &mut Self { self.text_justify(Justify::Center) }
    pub fn text_justify_left(&mut self) -> &mut Self { self.text_justify(Justify::Left) }
    pub fn text_justify_right(&mut self) -> &mut Self { self.text_justify(Justify::Right) }

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

        self.commands.entity(self.current_entity).insert(text_bundle);
        self
    }

    pub fn child(&mut self) -> &mut Self {
        self.parent_stack.push_back(self.current_entity);

        let child = self.commands.spawn_empty().id();
        self.commands.entity(child).insert(self.defaults.get_node_bundle(None));
        self.commands.entity(self.current_entity).add_child(child);
        self.current_entity = child;
        self
    }

    pub fn parent(&mut self) -> &mut Self {
        if let Some(parent) = self.parent_stack.pop_back() {
            self.current_entity = parent;
        }
        self
    }

    pub fn build(self) -> (Entity, Commands<'w, 's>) {
        if !self.parent_stack.is_empty() {
            (self.parent_stack[0], self.commands)
        } else {
            (self.current_entity, self.commands)
        }
    }

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

        self.child();
        let button_entity = self.current_entity;

        let btn_def = self.defaults.get_btn_def(None);

        let mut node = btn_def.get_node();
        node.width = Val::Px(width);
        node.height = Val::Px(height);
        node.border_radius = BorderRadius::all(Val::Px(border_radius));
        self.commands.entity(button_entity)
            .insert(node)
            .insert(btn_def.get_button_bundle())
            .insert(BackgroundColor(bg_color))
            .insert(bundle);

        let text_bundle = TextBundle {
            text: Text(text_str),
            text_font: TextFont::default()
                .with_font(btn_def.font.clone())
                .with_font_size(font_size),
            text_color: btn_def.get_text_color(),
        };
        let text_entity = self.commands.spawn(text_bundle).id();
        self.commands.entity(button_entity).add_child(text_entity);

        self.current_entity = original_entity;
        while self.parent_stack.len() > original_stack_len {
            self.parent_stack.pop_back();
        }

        self
    }

    pub fn build_button<T, F>(&mut self, component: T, f: F) -> &mut Self
    where
        T: Component,
        F: FnOnce(&mut ButtonBuilder),
    {
        let original_entity = self.current_entity;
        let original_stack_len = self.parent_stack.len();

        self.child();
        let button_entity = self.current_entity;

        let btn_def = self.defaults.get_btn_def(None);

        let mut node = btn_def.get_node();
        node.border_radius = btn_def.get_border_radius();
        self.commands.entity(button_entity)
            .insert(node)
            .insert(btn_def.get_button_bundle())
            .insert(component);

        let text_entity = self.commands.spawn(btn_def.get_text_bundle()).id();
        self.commands.entity(button_entity).add_child(text_entity);

        let mut button_builder = ButtonBuilder {
            ui: self,
            text_entity: Some(text_entity),
        };

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            f(&mut button_builder);
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
}
