use crate::civilization::general::general_components::{CityFlood, CitySite, FloodPlain, GameArea, LandPassage, NeedsConnections, Population, StartArea, Volcano};
use crate::civilization::general::general_enums::GameFaction;
use crate::loading::TextureAssets;
use crate::GameState;
use bevy::core::Name;
use bevy::prelude::{info, App, AssetServer, Assets, ButtonInput, Camera, Commands, GlobalTransform, Handle, Image, Local, MouseButton, OnEnter, Plugin, Query, Res, ResMut, Resource, SpriteBundle, Startup, Transform, Vec3, Window, With};
use bevy::utils::{HashMap, HashSet};
use bevy::window::PrimaryWindow;
use bevy_common_assets::ron::RonAssetPlugin;
use rand::seq::IteratorRandom;

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

#[derive(Default)]
struct MyState {
    id: i32,
}

fn mouse_button_input(
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    map: Res<MapHandle>,
    mut maps: ResMut<Assets<Map>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut local: Local<MyState>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        if local.id == 0 {
            local.id = 1;
        } else {
            local.id += 1;
        }
        let (camera, camera_transform) = q_camera.single();
        if let Some(position) = q_windows.single().cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            if let Some(level) = maps.get_mut(map.0.id()) {
                if let Some(area) = level.areas.iter_mut().find(|a| a.id == local.id) {
                    area.x = position.x;
                    area.y = position.y;
                }
            }
        }
    } else if buttons.just_pressed(MouseButton::Right) {
        if let Some(level) = maps.get(map.0.id()) {
            info!("{}", ron::ser::to_string_pretty(&level, Default::default()).unwrap());
        }
    }
}


#[derive(serde::Deserialize, serde::Serialize, bevy::asset::Asset, bevy::reflect::TypePath, Clone)]
pub struct Map {
    pub areas: Vec<Area>,
}

#[derive(Resource)]
struct MapHandle(Handle<Map>);

#[derive(Resource, Default)]
pub struct AvailableFactions {
    factions: HashSet<GameFaction>,
    pub remaining_factions: HashSet<GameFaction>,
    pub faction_icons: HashMap<GameFaction, Handle<Image>>,
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

#[derive(
    serde::Deserialize,
    serde::Serialize,
    bevy::asset::Asset,
    bevy::reflect::TypePath,
    Clone,
    Debug
)]
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
            maps: Res<Assets<Map>>,
            mut available_factions: ResMut<AvailableFactions>,
            textures: Res<TextureAssets>,
            mut camera: Query<(&Camera, &mut Transform)>,
) {
    if let Some(level) = maps.get(map.0.id()).clone() {
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


        let (_, mut transform) = camera.single_mut();
        transform.translation = Vec3::new(1250.0, 662.5, 0.0);

        commands.spawn(SpriteBundle {
            texture: textures.map.clone(),
            transform: Transform::from_xyz(1250.0, 662.5, -1.0),
            ..Default::default()
        });

        for area in level.areas.clone() {
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
                    Transform::from_xyz(area.x, area.y, 1.),
                    // SpriteBundle {
                    //     texture: textures.dot.clone(),
                    //     transform: Transform::from_xyz(area.x, area.y, 1.),
                    //     ..Default::default()
                    // }
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
                match faction {
                    GameFaction::Egypt => {
                        available_factions.faction_icons.insert(GameFaction::Egypt, textures.egypt.clone());
                    }
                    GameFaction::Crete => {
                        available_factions.faction_icons.insert(GameFaction::Crete, textures.crete.clone());
                    }
                    GameFaction::Africa => {
                        available_factions.faction_icons.insert(GameFaction::Africa, textures.africa.clone());
                    }
                    GameFaction::Asia => {
                        available_factions.faction_icons.insert(GameFaction::Asia, textures.asia.clone());
                    }
                    GameFaction::Assyria => {
                        available_factions.faction_icons.insert(GameFaction::Assyria, textures.assyria.clone());
                    }
                    GameFaction::Babylon => {
                        available_factions.faction_icons.insert(GameFaction::Babylon, textures.babylon.clone());
                    }
                    GameFaction::Illyria => {
                        available_factions.faction_icons.insert(GameFaction::Illyria, textures.illyria.clone());
                    }
                    GameFaction::Iberia => {
                        available_factions.faction_icons.insert(GameFaction::Iberia, textures.iberia.clone());
                    }
                    GameFaction::Thrace => {
                        available_factions.faction_icons.insert(GameFaction::Thrace, textures.thrace.clone());
                    }
                }
            }
        }
    }
}