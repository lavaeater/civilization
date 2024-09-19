use bevy::core::Name;
use bevy::prelude::{in_state, App, AssetServer, Assets, Commands, Handle, IntoSystemConfigs, Plugin, Res, ResMut, Resource, Startup, Update};
use bevy::utils::HashSet;
use bevy_common_assets::ron::RonAssetPlugin;
use crate::civilization::general::general_components::{CityFlood, CitySite, FloodPlain, GameArea, LandPassage, NeedsConnections, Population, StartArea, Volcano};
use crate::civilization::general::general_enums::GameFaction;
use crate::GameState;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<AvailableFactions>()
            .add_plugins(
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

#[derive(Resource, Default)]
pub struct AvailableFactions {
    factions: HashSet<GameFaction>,
    pub remaining_factions: HashSet<GameFaction>,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let map = MapHandle(asset_server.load("maps/civilization.map.ron"));
    commands.insert_resource(map);
}

#[derive(serde::Deserialize, bevy::asset::Asset, bevy::reflect::TypePath)]
pub struct Area {
    pub id: i32,
    pub max_population: usize,
    pub land_connections: Vec<i32>,
    pub sea_connections: Vec<i32>,
    pub start_area: Option<GameFaction>,
    pub city_site: bool,
    pub flood_plain: bool,
    pub city_flood: bool,
    pub volcano: bool,
}

fn load_map(mut commands: Commands,
            map: Res<MapHandle>,
            mut maps: ResMut<Assets<Map>>,
            mut available_factions: ResMut<AvailableFactions>,
) {
    if let Some(level) = maps.remove(map.0.id()) {
        for area in level.areas {
            let entity = commands.spawn((Name::new(format!("Area {}", area.id)),
                                         GameArea::new(area.id),
                                         LandPassage::default(),
                                         NeedsConnections {
                                             land_connections: area.land_connections,
                                             sea_connections: area.sea_connections,
                                         },
                                         Population::new(area.max_population))).id();
            if area.city_site {
                commands.entity(entity).insert(CitySite::default());
            }

            if area.city_flood {
                commands.entity(entity).insert(CityFlood::default());
            }

            if area.flood_plain {
                commands.entity(entity).insert(FloodPlain::default());
            }
            if area.volcano {
                commands.entity(entity).insert(Volcano::default());
            }
            if let Some(faction) = area.start_area {
                available_factions.factions.insert(faction);
                available_factions.remaining_factions.insert(faction);
                commands.entity(entity).insert(StartArea::new(faction));
            }
        }
    }
}