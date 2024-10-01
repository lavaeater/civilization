use crate::GameState;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::AudioSource;

pub struct LoadingPlugin;

/// This plugin loads all assets using [`AssetLoader`] from a third party bevy plugin
/// Alternatively you can write the logic to load assets yourself
/// If interested, take a look at <https://bevy-cheatbook.github.io/features/assets.html>
impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Menu)
                .load_collection::<AudioAssets>()
                .load_collection::<TextureAssets>(),
        );
    }
}

// the following asset collections will be loaded during the State `GameState::Loading`
// when done loading, they will be inserted as resources (see <https://github.com/NiklasEi/bevy_asset_loader>)

#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    #[asset(path = "audio/flying.ogg")]
    pub flying: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
    #[asset(path = "textures/bevy.png")]
    pub bevy: Handle<Image>,
    #[asset(path = "textures/github.png")]
    pub github: Handle<Image>,
    #[asset(path = "maps/adv_civ_map.png")]
    pub map: Handle<Image>,
    #[asset(path = "textures/dot.png")]
    pub dot: Handle<Image>,
    #[asset(path = "textures/africa.png")]
    pub africa: Handle<Image>,
    #[asset(path = "textures/asia.png")]
    pub asia: Handle<Image>,
    #[asset(path = "textures/assyria.png")]
    pub assyria: Handle<Image>,
    #[asset(path = "textures/babylon.png")]
    pub babylon: Handle<Image>,
    #[asset(path = "textures/crete.png")]
    pub crete: Handle<Image>,
    #[asset(path = "textures/egypt.png")]
    pub egypt: Handle<Image>,
    #[asset(path = "textures/iberia.png")]
    pub iberia: Handle<Image>,
    #[asset(path = "textures/illyria.png")]
    pub illyria: Handle<Image>,
    #[asset(path = "textures/thrace.png")]
    pub thrace: Handle<Image>,
    #[asset(path = "textures/africa-city.png")]
    pub africa_city: Handle<Image>,
    #[asset(path = "textures/asia-city.png")]
    pub asia_city: Handle<Image>,
    #[asset(path = "textures/assyria-city.png")]
    pub assyria_city: Handle<Image>,
    #[asset(path = "textures/babylon-city.png")]
    pub babylon_city: Handle<Image>,
    #[asset(path = "textures/crete-city.png")]
    pub crete_city: Handle<Image>,
    #[asset(path = "textures/egypt-city.png")]
    pub egypt_city: Handle<Image>,
    #[asset(path = "textures/iberia-city.png")]
    pub iberia_city: Handle<Image>,
    #[asset(path = "textures/illyria-city.png")]
    pub illyria_city: Handle<Image>,
    #[asset(path = "textures/thrace-city.png")]
    pub thrace_city: Handle<Image>,
}
