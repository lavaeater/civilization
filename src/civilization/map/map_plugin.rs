use bevy::core::Name;
use bevy::prelude::{in_state, App, AssetServer, Assets, Commands, Handle, IntoSystemConfigs, OnEnter, Plugin, Res, ResMut, Resource, SpriteBundle, Startup, Transform, Update};
use bevy::utils::HashSet;
use bevy_common_assets::ron::RonAssetPlugin;
use crate::civilization::general::general_components::{CityFlood, CitySite, FloodPlain, GameArea, LandPassage, NeedsConnections, Population, StartArea, Volcano};
use crate::civilization::general::general_enums::GameFaction;
use crate::GameState;
use rand::seq::IteratorRandom;
use crate::loading::TextureAssets;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<AvailableFactions>()
            .add_plugins(
                RonAssetPlugin::<Map>::new(&["map.ron"]))
            .add_systems(Startup, setup)
            .add_systems(OnEnter(GameState::Playing), load_map)
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

fn remove_random_place(places: &mut HashSet<String>) -> Option<String> {
    // Randomly pick an item from the HashSet
    let selected_place = places.iter().choose(&mut rand::thread_rng()).cloned();

    if let Some(place) = selected_place {
        // Remove it from the HashSet
        places.remove(&place);
        // Return the removed item
        Some(place)
    } else {
        None
    }
}

#[derive(serde::Deserialize, bevy::asset::Asset, bevy::reflect::TypePath)]
pub struct Area {
    pub id: i32,
    pub x: f32,
    pub y: f32,
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
            textures: Res<TextureAssets>,
) {
    if let Some(level) = maps.remove(map.0.id()) {
        let mut ancient_places: HashSet<String> = vec!["Assyria", "Numidia", "Carthage", "Troy", "Sparta", "Babylon", "Thebes",
                                                       "Alexandria", "Athens", "Byzantium", "Pompeii", "Ephesus", "Ctesiphon",
                                                       "Jerusalem", "Nineveh", "Sidon", "Tyre", "Memphis", "Heliopolis",
                                                       "Pergamum", "Delphi", "Corinth", "Argos", "Syracuse", "Palmyra", "Damascus",
                                                       "Antioch", "Petra", "Gadara", "Sidonia", "Susa", "Knossos", "Rhodes",
                                                       "Pella", "Gortyn", "Leptis Magna", "Cyrene", "Tingis", "Volubilis", "Utica",
                                                       "Sabratha", "Tanais", "Amarna", "Hattusa", "Ugarit", "Mari", "Arpad",
                                                       "Qatna", "Alalakh", "Emar", "Aleppo", "Homs", "Edessa", "Tarsus",
                                                       "Miletus", "Pergamon", "Amphipolis", "Mycenae", "Abydos", "Phaselis",
                                                       "Halicarnassus", "Hierapolis", "Sardis", "Perge", "Gades", "Saguntum",
                                                       "Tarraco", "Corduba", "Emerita Augusta", "Hispalis", "Lusitania", "Aquae Sulis",
                                                       "Lutetia", "Massilia", "Nemausus", "Arelate", "Arretium", "Capua", "Neapolis",
                                                       "Ravenna", "Tarentum", "Brundisium", "Venusia", "Cremona", "Mediolanum",
                                                       "Patavium", "Aquileia", "Polis", "Teotoburgum", "Vindobona", "Carnuntum",
                                                       "Sirmium", "Trebizond", "Chalcedon", "Nicopolis", "Heraclea", "Philippi",
                                                       "Beroea", "Dura-Europos", "Seleucia", "Apamea", "Raphia", "Avaris",
                                                       "Tanis", "Bubastis", "Herakleopolis", "Olynthus", "Thapsus", "Bulla Regia",
                                                       "Hippo Regius", "Lepcis Magna", "Cirta", "Timgad", "Zama", "Thugga",
                                                       "Kart Hadasht", "Rhegium", "Croton", "Selinus", "Acragas", "Himera",
                                                       "Naxos", "Messina", "Segesta", "Catana", "Syracuse", "Thasos", "Amphipolis",
                                                       "Potidaea", "Apollonia", "Abdera", "Athribis", "Berenice", "Oxyrhynchus",
                                                       "Hermopolis", "Canopus", "Thonis", "Heracleion", "Marsa Matruh", "Baalbek",
                                                       "Ebla", "Arwad", "Ashkelon", "Ascalon", "Gaza", "Megiddo", "Joppa",
                                                       "Beersheba", "Hebron", "Aelia Capitolina", "Neapolis", "Hierapolis"
        ].into_iter().map(|s| s.to_string()).collect();


        commands.spawn(SpriteBundle {
            texture: textures.map.clone(),
            transform: Transform::from_xyz(1250.0, 662.5, 0.0),
            ..Default::default()
        });

        for area in level.areas {
            let n = remove_random_place(&mut ancient_places).unwrap_or("STANDARD_NAME".to_string());

            let entity = commands.spawn(
                (
                    Name::new(format!("{}:{}", area.id, n)),
                    GameArea::new(area.id),
                    LandPassage::default(),
                    NeedsConnections {
                        land_connections: area.land_connections,
                        sea_connections: area.sea_connections,
                    },
                    Population::new(area.max_population),
                    SpriteBundle {
                        texture: textures.dot.clone(),
                        transform: Transform::from_xyz(area.x, area.y, 1.),
                        ..Default::default()
                    }
                )
            ).id();
            if area.city_site {
                commands.entity(entity).insert(CitySite);
            }

            if area.city_flood {
                commands.entity(entity).insert(CityFlood);
            }

            if area.flood_plain {
                commands.entity(entity).insert(FloodPlain);
            }
            if area.volcano {
                commands.entity(entity).insert(Volcano);
            }

            if let Some(faction) = area.start_area {
                available_factions.factions.insert(faction);
                available_factions.remaining_factions.insert(faction);
                commands.entity(entity).insert(StartArea::new(faction));
            }
        }
    }
}