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
