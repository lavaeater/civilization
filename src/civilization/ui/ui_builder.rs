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

    pub fn from_entity(
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

    pub fn block_with<T: Component + Default>(
        &mut self,
        width: Val,
        height: Val,
        bg_color: Color,
    ) -> &mut Self {
        self.block(width, height, bg_color).add_component::<T>()
    }
    
    pub fn at(&mut self, left: Val, top: Val, position_type: PositionType) -> &mut Self {
        self.commands.entity(self.current_entity)
            .entry::<Node>().and_modify(move |mut node| { 
            node.position_type = position_type;
            node.left = left;
            node.top = top;
        }).or_insert(Node {
            position_type,
            left,
            top,
            ..Default::default()
        });
        self
    }

    /// Add a default instance of the given component to the current entity.
    ///
    /// If the entity does not already have the given component, this method will
    /// add a default instance of it. This is a shorthand for calling
    /// `commands.entity(self.current_entity).entry::<T>().or_default();`.
    pub fn add_component<T: Component + Default>(&mut self) -> &mut Self {
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
    pub fn flex_row(&mut self) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| {
                node.display = Display::Flex;
                node.flex_direction = FlexDirection::Row;
            })
            .or_insert(Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                ..default()
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
    pub fn flex_column(&mut self) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| {
                node.display = Display::Flex;
                node.flex_direction = FlexDirection::Column;
            })
            .or_insert(Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                ..default()
            });
        self
    }

    pub fn flex_column_with_props(
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
            })
            .or_insert(Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                align_content: AlignContent::FlexStart,
                justify_content: JustifyContent::FlexStart,
                width,
                height,
                ..default()
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

    pub fn block(&mut self, width: Val, height: Val, bg_color: Color) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| {
                node.width = width;
                node.height = height;
                node.display = Display::Block;
            })
            .or_insert(Node {
                width,
                height,
                display: Display::Block,
                ..default()
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
            // Modify the component if it exists
            .and_modify(move |mut node| node.width = width)
            // Otherwise insert a default value
            .or_insert(Node { width, ..default() });
        self
    }

    /// Set height Node
    pub fn height(&mut self, height: Val) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.height = height)
            .or_insert(Node {
                height,
                ..default()
            });
        self
    }

    /// Set flex direction
    pub fn flex_direction(&mut self, direction: FlexDirection) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.flex_direction = direction)
            .or_insert(Node {
                flex_direction: direction,
                ..default()
            });
        self
    }

    /// Set justify content
    pub fn justify_content(&mut self, justify: JustifyContent) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.justify_content = justify)
            .or_insert(Node {
                justify_content: justify,
                ..default()
            });
        self
    }

    /// Set align items
    pub fn align_items(&mut self, align: AlignItems) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.align_items = align)
            .or_insert(Node {
                align_items: align,
                ..default()
            });
        self
    }

    /// Set padding
    pub fn padding(&mut self, padding: UiRect) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.padding = padding)
            .or_insert(Node {
                padding,
                ..default()
            });
        self
    }

    /// Set margin
    pub fn margin(&mut self, margin: UiRect) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.margin = margin)
            .or_insert(Node {
                margin,
                ..default()
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
        font: Handle<Font>,
        font_size: f32,
        color: Option<Color>,
    ) -> &mut Self {
        self.move_to_new_child()
            .add_text(text, font, font_size, color)
            .parent()
    }
    
    pub fn add_text(&mut self, text: impl Into<String>,
                    font: Handle<Font>,
                    font_size: f32,
                    color: Option<Color>,) -> &mut Self {
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
