use bevy::prelude::*;
use bevy::utils::HashMap;
use std::collections::VecDeque;
use cucumber::WriterExt;

/// Fluent UI Builder for creating Bevy UI elements
pub struct UIBuilder<'w, 's> {
    commands: Commands<'w, 's>,
    current_entity: Entity,
    parent_stack: VecDeque<Entity>
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
}

impl<'w, 's> UIBuilder<'w, 's> {
    /// Create a new root UI container
    pub fn new(mut commands: Commands<'w, 's>) -> Self {
        // Create a basic node entity with default settings
        let entity = commands.spawn_empty().id();
        commands.entity(entity).insert(Node::default());

        Self {
            commands,
            current_entity: entity,
            parent_stack: VecDeque::new(),
        }
    }

    pub fn start_from_entity(
        mut commands: Commands<'w, 's>,
        entity: Entity,
        clear_children: bool,
    ) -> Self {
        if clear_children {
            commands.entity(entity).despawn_descendants();
        }
        commands.entity(entity).entry::<Node>().or_default();
        Self {
            commands,
            current_entity: entity,
            parent_stack: VecDeque::new(),
        }
    }
    
    /// Add a Button component to the current entity
    pub fn with_button(&mut self) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Button>().or_default();
        self
    }
    
    /// Add a Button component to the current entity with a specific component type
    /// 
    /// This version adds a default instance of the component type T
    pub fn with_button_and_component<T: Component + Default>(&mut self) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Button>().or_default()
            .entry::<T>().or_default();
        self
    }
    
    /// Add a Button component to the current entity with a specific component instance
    /// 
    /// This version adds the provided component instance
    pub fn with_button_and<T: Component>(&mut self, component: T) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Button>().or_default()
            .insert(component);
        self
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

    /// Set the current entity to be a flexbox container with a column flex direction
    ///
    /// The entity will be set to be a block with a flexbox display mode and a column
    /// flex direction. This means that any children of the entity will be laid out
    /// vertically from top to bottom.
    ///
    /// # Example
    ///
    ///
    pub fn as_flex_col(&mut self) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| {
                node.display = Display::Flex;
                node.flex_direction = FlexDirection::Column;
            });
        self
    }

    pub fn as_flex_col_with_props(
        &mut self,
        width: Val,
        height: Val,
        bg_color: Color,
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
        self.commands
            .entity(self.current_entity)
            .entry::<BackgroundColor>()
            .and_modify(move |mut bg| {
                *bg = BackgroundColor(bg_color);
            })
            .or_insert(BackgroundColor(bg_color));

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
        font: Handle<Font>,
        font_size: f32,
        color: Option<Color>,
    ) -> &mut Self {
        self.move_to_new_child()
            .with_text(text, font, font_size, color)
            .parent()
    }

    pub fn with_text(
        &mut self,
        text: impl Into<String>,
        font: Handle<Font>,
        font_size: f32,
        color: Option<Color>,
    ) -> &mut Self {
        let text_color = color.unwrap_or(Color::BLACK);

        let text_bundle = (
            Text::new(text.into()),
            TextFont::from_font(font).with_font_size(font_size),
            TextColor(text_color),
        );

        // Add the text component to the entity
        self.commands
            .entity(self.current_entity)
            .insert(text_bundle);
        self
    }

    /// Create a child container
    pub fn move_to_new_child(&mut self) -> &mut Self {
        // Push current entity to parent stack
        self.parent_stack.push_back(self.current_entity);

        // Spawn a new node entity as child
        let child = self.commands.spawn_empty().id();
        self.commands.entity(child).insert(Node::default());

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
