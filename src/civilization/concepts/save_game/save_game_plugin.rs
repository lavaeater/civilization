use bevy::prelude::*;
use moonshine_save::prelude::*;

pub struct SaveGamePlugin;

impl Plugin for SaveGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(save_on_default_event)
            .add_observer(load_on_default_event)
            .add_systems(Update, (save_on_key, load_on_key));
    }
}

fn save_on_key(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::F5) {
        info!("Saving game...");
        commands.trigger_save(SaveWorld::default_into_file("savegame.ron"));
    }
}

fn load_on_key(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::F9) {
        info!("Loading game...");
        commands.trigger_load(LoadWorld::default_from_file("savegame.ron"));
    }
}
