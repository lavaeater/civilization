use bevy::prelude::*;

/// Fluent UI Builder for creating Bevy UI elements
pub struct UIBuilder {
    commands: Commands,
    current_entity: Entity,
}

impl UIBuilder {
    /// Create a new root UI container
    pub fn new(mut commands: Commands) -> Self {
        let entity = commands.spawn(NodeBundle::default()).id();
        Self {
            commands,
            current_entity: entity,
        }
    }

    /// Set the UI element's style
    pub fn style(mut self, style: Style) -> Self {
        self.commands.entity(self.current_entity).insert(style);
        self
    }

    /// Add background color
    pub fn background_color(mut self, color: Color) -> Self {
        self.commands.entity(self.current_entity).insert(BackgroundColor(color));
        self
    }

    /// Add text to the current UI element
    pub fn text(mut self, text: impl Into<String>, font: Handle<Font>, font_size: f32) -> Self {
        self.commands.entity(self.current_entity).insert(
            TextBundle::from_section(
                text.into(),
                TextStyle {
                    font,
                    font_size,
                    color: Color::BLACK,
                }
            )
        );
        self
    }

    /// Create a child container
    pub fn container(mut self) -> Self {
        let child = self.commands.spawn(NodeBundle::default()).id();
        self.commands.entity(self.current_entity).add_child(child);

        Self {
            commands: self.commands,
            current_entity: child,
        }
    }

    /// Return to parent container
    pub fn parent(mut self) -> Self {
        // This is a placeholder. In a real implementation, 
        // you'd need to track parent hierarchy
        self
    }

    /// Finalize and get the root entity
    pub fn build(self) -> Entity {
        self.current_entity
    }
}

/// Example system demonstrating UI builder usage
fn create_ui_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    let root_ui = UIBuilder::new(commands)
        .style(Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .background_color(Color::GRAY)
        .container()
        .style(Style {
            width: Val::Px(200.0),
            height: Val::Px(100.0),
            ..default()
        })
        .background_color(Color::WHITE)
        .text("Hello, UI!", font.clone(), 24.0)
        .parent()
        .build();
}
