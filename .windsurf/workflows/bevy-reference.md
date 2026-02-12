---
description: Bevy 0.18 API reference and patterns used in this project
---

# Bevy 0.18 API Reference

This project uses **Bevy 0.18.0**. Many APIs changed significantly from 0.14/0.15/0.16. Always use the patterns below.

## Messages (replaces Events)

Bevy 0.18 replaced `Event` with `Message`. Do NOT use `#[derive(Event)]`, `EventWriter`, `EventReader`, or `app.add_event::<T>()`.

```rust
// Define
#[derive(Message, Debug)]
pub struct MyMessage {
    pub data: i32,
}

// Send
fn sender(mut writer: MessageWriter<MyMessage>) {
    writer.write(MyMessage { data: 42 });
}

// Receive
fn receiver(mut reader: MessageReader<MyMessage>) {
    for msg in reader.read() {
        info!("Got: {:?}", msg);
    }
}
```

Messages are auto-registered — no `app.add_event()` needed.

## Observer Triggers (Component Lifecycle)

Use `On<Add, T>` and `On<Remove, T>` for component lifecycle hooks:

```rust
fn on_add_my_component(
    trigger: On<Add, MyComponent>,
    query: Query<&MyComponent>,
) {
    let entity = trigger.event().entity;
    // ...
}

// Register in plugin:
app.add_observer(on_add_my_component);
```

## States and SubStates

```rust
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
}

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default, Reflect)]
#[source(GameState = GameState::Playing)]
pub enum GameActivity {
    #[default]
    PrepareGame,
    Movement,
    // ...
}
```

Use `app.init_state::<GameState>()` and `app.add_sub_state::<GameActivity>()`.

Schedule systems with:
```rust
app.add_systems(OnEnter(GameActivity::Movement), start_movement);
app.add_systems(Update, my_system.run_if(in_state(GameActivity::Movement)));
```

## Components and Bundles

Bundles are removed. Just use tuples of components:

```rust
commands.spawn((
    Camera2d,
    IsDefaultUiCamera,
    Projection::Orthographic(OrthographicProjection::default_2d()),
    Msaa::Off,
));

commands.spawn((
    Sprite {
        image: texture_handle,
        ..default()
    },
    Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
));
```

No `SpriteBundle`, `Camera2dBundle`, `NodeBundle`, etc.

## Collections

```rust
use bevy::platform::collections::HashMap;
use bevy::platform::collections::HashSet;
```

NOT `bevy::utils::HashMap`.

## UI

No `NodeBundle`, `ButtonBundle`, `TextBundle`. Use component tuples:

```rust
commands.spawn((
    Node { /* layout */ },
    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
));
```

This project uses `lava_ui_builder` crate with `UIBuilder` and `UiTheme` for UI construction.

## World Access

Do NOT use `world.send_event(...)` — it doesn't exist in 0.18.
For deferred world access use `commands.queue`:

```rust
commands.queue(|w: &mut World| {
    // direct world manipulation
});
```

But prefer `MessageWriter` for inter-system communication.

## Query Changes

`Has<T>` returns `bool` in query tuples:
```rust
fn my_system(query: Query<(Entity, &Name, Has<IsHuman>)>) {
    for (entity, name, is_human) in query.iter() {
        if is_human { /* ... */ }
    }
}
```

## Common Imports

```rust
use bevy::prelude::*;  // Most things
use bevy::prelude::{
    Commands, Entity, Query, Res, ResMut,
    MessageReader, MessageWriter,
    NextState, State,
    With, Without, Has,
    OnEnter, Update,
    info, debug, warn, error,
    Name, Transform, Sprite,
    Message, Component, Resource, Reflect,
};
```

## Key Differences Summary

| Old (0.14-0.16)              | New (0.18)                              |
|------------------------------|-----------------------------------------|
| `#[derive(Event)]`          | `#[derive(Message)]`                    |
| `EventWriter<T>`            | `MessageWriter<T>`                      |
| `EventReader<T>`            | `MessageReader<T>`                      |
| `app.add_event::<T>()`      | Not needed (auto-registered)            |
| `Trigger<OnAdd, T>`         | `On<Add, T>`                            |
| `SpriteBundle`              | `(Sprite { .. }, Transform { .. })`     |
| `Camera2dBundle`            | `(Camera2d, ..)`                        |
| `NodeBundle`                | `(Node { .. }, ..)`                     |
| `bevy::utils::HashMap`      | `bevy::platform::collections::HashMap`  |
| `world.send_event(e)`       | Use `MessageWriter` in systems          |
