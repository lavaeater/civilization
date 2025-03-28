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
        // Get the current entity
        let mut entity = self.commands.entity(self.current_entity);

        // Create a new Node with the specified height
        let mut node = Node::default();
        node.height = height;

        // Insert or replace the Node component
        entity.insert(node);

        self
    }

    /// Set flex direction
    pub fn flex_direction(mut self, direction: FlexDirection) -> Self {
        // Get the current entity
        let mut entity = self.commands.entity(self.current_entity);

        // Create a new Node with the specified flex direction
        let mut node = Node::default();
        node.flex_direction = direction;

        // Insert or replace the Node component
        entity.insert(node);

        self
    }

    /// Set justify content
    pub fn justify_content(mut self, justify: JustifyContent) -> Self {
        // Get the current entity
        let mut entity = self.commands.entity(self.current_entity);

        // Create a new Node with the specified justify content
        let mut node = Node::default();
        node.justify_content = justify;

        // Insert or replace the Node component
        entity.insert(node);

        self
    }

    /// Set align items
    pub fn align_items(mut self, align: AlignItems) -> Self {
        // Get the current entity
        let mut entity = self.commands.entity(self.current_entity);

        // Create a new Node with the specified align items
        let mut node = Node::default();
        node.align_items = align;

        // Insert or replace the Node component
        entity.insert(node);

        self
    }

    /// Set padding
    pub fn padding(mut self, padding: UiRect) -> Self {
        // Get the current entity
        let mut entity = self.commands.entity(self.current_entity);

        // Create a new Node with the specified padding
        let mut node = Node::default();
        node.padding = padding;

        // Insert or replace the Node component
        entity.insert(node);

        self
    }

    /// Set margin
    pub fn margin(mut self, margin: UiRect) -> Self {
        // Get the current entity
        let mut entity = self.commands.entity(self.current_entity);

        // Create a new Node with the specified margin
        let mut node = Node::default();
        node.margin = margin;

        // Insert or replace the Node component
        entity.insert(node);

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
            .insert(BackgroundColor(color));
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
}

/// Example system demonstrating UI builder usage
fn create_ui_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    let root_ui = UIBuilder::new(commands)
        .width(Val::Percent(100.0))
        .height(Val::Percent(100.0))
        .flex_direction(FlexDirection::Column)
        .background_color(Color::rgb(0.5, 0.5, 0.5))
        .container()
        .width(Val::Px(200.0))
        .height(Val::Px(100.0))
        .background_color(Color::WHITE)
        .text("Hello, UI!", font.clone(), 24.0, Some(Color::BLACK))
        .parent()
        .build();
}
