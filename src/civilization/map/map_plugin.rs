use bevy::core::Name;
use bevy::prelude::{in_state, App, AssetServer, Assets, Commands, Handle, IntoSystemConfigs, Plugin, Res, ResMut, Resource, Startup, Update};
use bevy_common_assets::ron::RonAssetPlugin;
use crate::civilization::general::general_components::{CitySite, Faction, GameArea, LandPassage, NeedsConnections, Population, StartArea};
use crate::civilization::general::general_enums::GameFaction;
use crate::GameState;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            RonAssetPlugin::<Map>::new(&["map.ron"]))
            .add_systems(Startup, setup)
            .add_systems(Update, load_map.run_if(in_state(GameState::Loading)))
        ;
    }
}

#[derive(serde::Deserialize, bevy::asset::Asset, bevy::reflect::TypePath)]
pub struct Map {
    pub areas: Vec<Area>,
}

#[derive(Resource)]
struct MapHandle(Handle<Map>);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let map = MapHandle(asset_server.load("maps/civilization.map.ron"));
    commands.insert_resource(map);
}

#[derive(serde::Deserialize, bevy::asset::Asset, bevy::reflect::TypePath)]
pub struct Area {
    pub name: String,
    pub land_connections: Vec<String>,
    pub max_population: usize,
    pub sea_connections: Vec<String>,
    pub city_site: bool,
    pub start_area: Option<GameFaction>,
}

fn load_map(mut commands: Commands,
            map: Res<MapHandle>,
            mut maps: ResMut<Assets<Map>>, ) {
    if let Some(level) = maps.remove(map.0.id()) {
        for area in level.areas {
            let entity = commands.spawn((Name::new(area.name),
                                         GameArea::default(),
                                         LandPassage::default(),
                                         NeedsConnections {
                                             land_connections: area.land_connections,
                                             sea_connections: area.sea_connections,
                                         },
                                         Population::new(area.max_population))).id();
            if area.city_site {
                commands.entity(entity).insert(CitySite::default());
            }
            if let Some(faction) = area.start_area {
                commands.entity(entity).insert(StartArea::new(faction));
            }
        }
    }
}