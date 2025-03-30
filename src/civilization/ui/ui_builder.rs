use bevy::prelude::*;
use std::collections::VecDeque;

/// Fluent UI Builder for creating Bevy UI elements
pub struct UIBuilder<'w, 's> {
    commands: Commands<'w, 's>,
    current_entity: Entity,
    parent_stack: VecDeque<Entity>,
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
    
    pub fn from_entity(mut commands: Commands<'w, 's>, entity: Entity, clear_children: bool) -> Self {
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

    pub fn block_with<T: Component + Default>(self, width_percent: f32, height_percent: f32, bg_color: Color) -> Self {
        let mut snake = self.block(width_percent, height_percent, bg_color);
            snake.commands
            .entity(snake.current_entity)
            .entry::<T>()
            .or_default();
        
        snake
    }
    
    /// Add a default instance of the given component to the current entity.
    ///
    /// If the entity does not already have the given component, this method will
    /// add a default instance of it. This is a shorthand for calling
    /// `commands.entity(self.current_entity).entry::<T>().or_default();`.
    pub fn add_component<T: Component + Default>(mut self) -> Self {
        self.commands.entity(self.current_entity).entry::<T>().or_default();
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
    pub fn flex_row(mut self) -> Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| {
                node.display = Display::Flex;
                node.flex_direction= FlexDirection::Row;
            }).or_insert(Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                ..default()
        });
        self.container()
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
    pub fn flex_column(mut self) -> Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| {
                node.display = Display::Flex;
                node.flex_direction= FlexDirection::Column;
            }).or_insert(Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            ..default()
        });
        self.container()
    }
    
    pub fn flex_column_with_props(mut self, width_percent: f32, height_percent: f32, bg_color: Color) -> Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| {
                node.display = Display::Flex;
                node.flex_direction= FlexDirection::Column;
                node.align_items = AlignItems::FlexStart;
                node.align_content= AlignContent::FlexStart;
                node.max_height = Val::Percent(height_percent);
                node.justify_content = JustifyContent::FlexStart;
                node.width = Val::Percent(width_percent);
                node.height = Val::Percent(height_percent / 4.0);
            }).or_insert(Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            align_content: AlignContent::FlexStart,
            justify_content: JustifyContent::FlexStart,
            width: Val::Percent(width_percent),
            height: Val::Percent(height_percent),
            ..default()
        });
        self.commands
            .entity(self.current_entity)
            .entry::<BackgroundColor>()
            .and_modify(move |mut bg| {
                *bg = BackgroundColor(bg_color); 
            }).or_insert(BackgroundColor(bg_color));
        self.container()
    }

    pub fn block(mut self, width_percent: f32, height_percent: f32, bg_color: Color) -> Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| { 
                node.width = Val::Percent(width_percent);
                node.height = Val::Percent(height_percent);
                node.display = Display::Block;
            })
            .or_insert(Node { 
                width: Val::Percent(width_percent),
                height: Val::Percent(height_percent),    
                display: Display::Block,
                ..default() });

        self.commands
            .entity(self.current_entity)
            .entry::<BackgroundColor>()
            .and_modify(move |mut bg| {
                *bg = BackgroundColor(bg_color);
            })
            .or_insert(BackgroundColor(bg_color));
        
        self.container()
    }

    /// Set width Node
    pub fn width(mut self, width: Val) -> Self {
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
    pub fn height(mut self, height: Val) -> Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.height = height)
            .or_insert(Node { height, ..default() });
        self
    }

    /// Set flex direction
    pub fn flex_direction(mut self, direction: FlexDirection) -> Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.flex_direction = direction)
            .or_insert(Node { flex_direction: direction, ..default() });
        self
    }

    /// Set justify content
    pub fn justify_content(mut self, justify: JustifyContent) -> Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.justify_content = justify)
            .or_insert(Node { justify_content: justify, ..default() });
        self
    }

    /// Set align items
    pub fn align_items(mut self, align: AlignItems) -> Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.align_items = align)
            .or_insert(Node { align_items: align, ..default() });
        self
    }

    /// Set padding
    pub fn padding(mut self, padding: UiRect) -> Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.padding = padding)
            .or_insert(Node { padding, ..default() });
        self
    }

    /// Set margin
    pub fn margin(mut self, margin: UiRect) -> Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(move |mut node| node.margin = margin)
            .or_insert(Node { margin, ..default() });
        self
    }

    /// Apply a complete Node
    pub fn node(mut self, node: Node) -> Self {
        self.commands.entity(self.current_entity).insert(node);
        self
    }

    /// Add background color
    pub fn background_color(mut self, color: Color) -> Self {
        self.commands
            .entity(self.current_entity)
            .entry::<BackgroundColor>()
            .and_modify(move |mut background| *background = BackgroundColor(color))
            .or_insert(BackgroundColor(color));
        self
    }

    /// Add text to the current UI element
    pub fn text(
        mut self,
        text: impl Into<String>,
        font: Handle<Font>,
        font_size: f32,
        color: Option<Color>,
    ) -> Self {
        let text_color = color.unwrap_or(Color::BLACK);

        // Create a text component
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
    pub fn container(mut self) -> Self {
        // Push current entity to parent stack
        self.parent_stack.push_back(self.current_entity);

        // Spawn a new node entity as child
        let child = self.commands.spawn_empty().id();
        self.commands.entity(child).insert(Node::default());

        // Add the child to the current entity
        self.commands.entity(self.current_entity).add_child(child);

        // Return a new builder with the child as the current entity
        Self {
            commands: self.commands,
            current_entity: child,
            parent_stack: self.parent_stack,
        }
    }

    /// Return to parent container
    pub fn parent(mut self) -> Self {
        if let Some(parent) = self.parent_stack.pop_back() {
            self.current_entity = parent;
        }
        self
    }

    pub fn for_each<I, F, T>(mut self, items: I, mut f: F) -> Self
    where
        I: Iterator<Item = T>,
        F: FnMut(T, &mut Self),
    {
        for item in items {
            f(item, &mut self);
        }

        self
    }

    /// Finalize and get the root entity
    pub fn build(self) -> Entity {
        // Return the top-level entity (first one created)
        if !self.parent_stack.is_empty() {
            // If we have parents, the first one is the root
            self.parent_stack[0]
        } else {
            // Otherwise return the current entity
            self.current_entity
        }
    }
    
    pub fn build_command(self) -> (Commands<'w, 's>, Entity) {
        (self.commands, if !self.parent_stack.is_empty() {
            // If we have parents, the first one is the root
            self.parent_stack[0]
        } else {
            // Otherwise return the current entity
            self.current_entity
        })
    }
}
