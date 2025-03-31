use bevy::prelude::*;
use bevy::utils::HashMap;
use std::collections::VecDeque;

/// Fluent UI Builder for creating Bevy UI elements
pub struct UIBuilder<'w, 's> {
    commands: Commands<'w, 's>,
    current_entity: Entity,
    parent_stack: VecDeque<Entity>,
    tagged_nodes: HashMap<String, Entity>,
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
            tagged_nodes: HashMap::new(),
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
            tagged_nodes: HashMap::new(),
        }
    }
    
    pub fn with_props(&mut self, 
                        display: Option<Display>,
                        position_type: Option<PositionType>,
                        left: Option<Val>,
                        right: Option<Val>,
                        top: Option<Val>,
                        bottom: Option<Val>,
                        flex_direction: Option<FlexDirection>,
                        flex_wrap: Option<FlexWrap>,
                        align_items: Option<AlignItems>,
                        justify_items: Option<JustifyItems>,
                        align_self: Option<AlignSelf>,
                        justify_self: Option<JustifySelf>,
                        align_content: Option<AlignContent>,
                        justify_content: Option<JustifyContent>,
                        margin: Option<UiRect>,
                        padding: Option<UiRect>,
                        border: Option<UiRect>,
                        flex_grow: Option<f32>,
                        flex_shrink: Option<f32>,
                        flex_basis: Option<Val>,
                        width: Option<Val>,
                        height: Option<Val>,
                        min_width: Option<Val>,
                        min_height: Option<Val>,
                        max_width: Option<Val>,
                        max_height: Option<Val>,
                        overflow: Option<Overflow>,
                        overflow_clip_margin: Option<OverflowClipMargin>,
                        row_gap: Option<Val>,
                        column_gap: Option<Val>,
                        grid_auto_flow: Option<GridAutoFlow>,
                        grid_auto_columns: Option<Vec<GridTrack>>,
                        grid_column: Option<GridPlacement>,
                        grid_row: Option<GridPlacement>,
                        
    ) -> &mut Self {
        self
            .commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                n.display = display.unwrap_or(n.display);
                n.position_type = position_type.unwrap_or(n.position_type);
                n.left = left.unwrap_or(n.left);
                n.right = right.unwrap_or(n.right);
                n.top = top.unwrap_or(n.top);
                n.bottom = bottom.unwrap_or(n.bottom);
                n.flex_direction = flex_direction.unwrap_or(n.flex_direction);
                n.flex_wrap = flex_wrap.unwrap_or(n.flex_wrap);
                n.align_items = align_items.unwrap_or(n.align_items);
                n.justify_items = justify_items.unwrap_or(n.justify_items);
                n.align_self = align_self.unwrap_or(n.align_self);
                n.justify_self = justify_self.unwrap_or(n.justify_self);
                n.align_content = align_content.unwrap_or(n.align_content);
                n.justify_content = justify_content.unwrap_or(n.justify_content);
                n.margin = margin.unwrap_or(n.margin);
                n.padding = padding.unwrap_or(n.padding);
                n.border = border.unwrap_or(n.border);
                n.flex_grow = flex_grow.unwrap_or(n.flex_grow);
                n.flex_shrink = flex_shrink.unwrap_or(n.flex_shrink);
                n.flex_basis = flex_basis.unwrap_or(n.flex_basis);
                n.width = width.unwrap_or(n.width);
                n.height = height.unwrap_or(n.height);
                n.min_width = min_width.unwrap_or(n.min_width);
                n.min_height = min_height.unwrap_or(n.min_height);
                n.max_width = max_width.unwrap_or(n.max_width);
                n.max_height = max_height.unwrap_or(n.max_height);
                n.overflow = overflow.unwrap_or(n.overflow);
                n.overflow_clip_margin = overflow_clip_margin.unwrap_or(n.overflow_clip_margin);
                n.row_gap = row_gap.unwrap_or(n.row_gap);
                n.column_gap = column_gap.unwrap_or(n.column_gap);
                n.grid_auto_flow = grid_auto_flow.unwrap_or(n.grid_auto_flow);
                n.grid_auto_columns = grid_auto_columns.unwrap_or(n.grid_auto_columns.clone());
                n.grid_column = grid_column.unwrap_or(n.grid_column);
                n.grid_row = grid_row.unwrap_or(n.grid_row);
            });
        self
    }
    
    pub fn apply_node(&mut self, node: Node) -> &mut Self {
        self
            .commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
            n.display = node.display;
            n.position_type = node.position_type;
            n.left = node.left;
            n.right = node.right;
            n.top = node.top;
            n.bottom = node.bottom;
            n.flex_direction = node.flex_direction;
            n.flex_wrap = node.flex_wrap;
            n.align_items = node.align_items;
            n.justify_items = node.justify_items;
            n.align_self = node.align_self;
            n.justify_self = node.justify_self;
            n.align_content = node.align_content;
            n.justify_content = node.justify_content;
            n.margin = node.margin;
            n.padding = node.padding;
            n.border = node.border;
            n.flex_grow = node.flex_grow;
            n.flex_shrink = node.flex_shrink;
            n.flex_basis = node.flex_basis;
            n.width = node.width;
            n.height = node.height;
            n.min_width = node.min_width;
            n.min_height = node.min_height;
            n.max_width = node.max_width;
            n.max_height = node.max_height;
            n.aspect_ratio = node.aspect_ratio;
            n.overflow = node.overflow;
            n.overflow_clip_margin = node.overflow_clip_margin;
            n.row_gap = node.row_gap;
            n.column_gap = node.column_gap;
            n.grid_auto_flow = node.grid_auto_flow;
            n.grid_template_rows = node.grid_template_rows.clone();
            n.grid_template_columns = node.grid_template_columns.clone();
            n.grid_auto_rows = node.grid_auto_rows.clone();
            n.grid_auto_columns = node.grid_auto_columns.clone();
            n.grid_column = node.grid_column;
            n.grid_row = node.grid_row;
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
