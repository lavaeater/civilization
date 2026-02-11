use bevy::prelude::*;
use bevy::render::camera::CameraRenderGraph;
use bevy::render::sync_world::SyncToRenderWorld;
use bevy_camera::visibility::VisibilityClass;
use moonshine_save::prelude::*;

pub struct SaveGamePlugin;

impl Plugin for SaveGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(save_on_default_event)
            .add_observer(load_on_default_event)
            .add_systems(Update, (save_on_key, load_on_key));
    }
}

/// Creates a SaveWorld event with problematic components excluded.
/// These components contain types that can't be serialized (asset handles, TypeId, etc.)
pub fn create_save_event() -> SaveWorld {
    SaveWorld::default_into_file("savegame.ron")
        .exclude_component::<VisibilityClass>()
        .exclude_component::<CameraRenderGraph>()
        .exclude_component::<Sprite>()
        .exclude_component::<Transform>()
        .exclude_component::<GlobalTransform>()
        .exclude_component::<Visibility>()
        .exclude_component::<InheritedVisibility>()
        .exclude_component::<ViewVisibility>()
        .exclude_component::<SyncToRenderWorld>()
}

fn save_on_key(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::F5) {
        info!("Saving game...");
        commands.trigger_save(create_save_event());
    }
}

fn load_on_key(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::F9) {
        info!("Loading game...");
        commands.trigger_load(LoadWorld::default_from_file("savegame.ron"));
    }
}
