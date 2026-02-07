use crate::civilization::components::Population;
use crate::civilization::components::{CityFlood, CitySite, FloodPlain, GameArea, GameCamera, LandPassage, NeedsConnections, StartArea, Volcano};
use crate::civilization::enums::GameFaction;
use crate::civilization::general_systems::setup_players;
use crate::loading::TextureAssets;
use crate::{GameActivity, GameState};
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::{in_state, App, AssetServer, Assets, ButtonInput, Camera, Commands, Handle, Image, IntoScheduleConfigs, KeyCode, MessageReader, Name, OnEnter, Plugin, Projection, Query, Res, ResMut, Resource, Sprite, Startup, Transform, Update, Vec2, Vec3, Window, With};
use bevy::time::Time;
use bevy::window::{PrimaryWindow, WindowResized};
use bevy_common_assets::ron::RonAssetPlugin;
use rand::seq::IteratorRandom;
use crate::civilization::start_game_after_player_setup;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AvailableFactions>()
            .add_plugins(RonAssetPlugin::<Map>::new(&["map.ron"]))
            .add_systems(Startup, setup)
            .add_systems(
                OnEnter(GameActivity::PrepareGame),
                (load_map, setup_players, start_game_after_player_setup).chain(),
            )
            .add_systems(
                Update,
                (
                    fit_map_camera_on_resize,
                    handle_map_camera_controls,
                ).run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Resource, Clone, Copy)]
struct MapViewConfig {
    map_size: Vec2,
    map_center: Vec3,
}

fn fit_camera_to_map(
    map_size: Vec2,
    window_size: Vec2,
    projection: &mut Projection,
) {
    let padding = 1.02;
    let window_w = window_size.x.max(1.0);
    let window_h = window_size.y.max(1.0);

    // With `ScalingMode::WindowSize(1.0)`, the camera shows `window_size * scale` world units.
    let needed_scale_w = map_size.x / window_w;
    let needed_scale_h = map_size.y / window_h;
    let needed_scale = needed_scale_w.max(needed_scale_h) * padding;

    if let Projection::Orthographic(ref mut ortho) = *projection {
        ortho.scale = needed_scale;
    }
}

// #[derive(Default)]
// struct MyState {
//     id: i32,
// }
//
// fn mouse_button_input(
//     buttons: Res<ButtonInput<MouseButton>>,
//     q_windows: Query<&Window, With<PrimaryWindow>>,
//     map: Res<MapHandle>,
//     mut maps: ResMut<Assets<Map>>,
//     q_camera: Query<(&Camera, &GlobalTransform)>,
//     mut local: Local<MyState>,
// ) {
//     if buttons.just_pressed(MouseButton::Left) {
//         if local.id == 0 {
//             local.id = 1;
//         } else {
//             local.id += 1;
//         }
//         let (camera, camera_transform) = q_camera.single();
//         if let Some(position) = q_windows.single().cursor_position()
//             .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
//             .map(|ray| ray.origin.truncate())
//         {
//             if let Some(level) = maps.get_mut(map.0.id()) {
//                 if let Some(area) = level.areas.iter_mut().find(|a| a.id == local.id) {
//                     area.x = position.x;
//                     area.y = position.y;
//                 }
//             }
//         }
//     } else if buttons.just_pressed(MouseButton::Right) {
//         if let Some(level) = maps.get(map.0.id()) {
//             info!("{}", ron::ser::to_string_pretty(&level, Default::default()).unwrap());
//         }
//     }
// }

#[derive(
    serde::Deserialize, serde::Serialize, bevy::asset::Asset, bevy::reflect::TypePath, Clone,
)]
pub struct Map {
    pub areas: Vec<Area>,
}

#[derive(Resource)]
struct MapHandle(Handle<Map>);

#[derive(Resource, Default)]
pub struct AvailableFactions {
    pub factions: HashSet<GameFaction>,
    pub remaining_factions: HashSet<GameFaction>,
    pub faction_icons: HashMap<GameFaction, Handle<Image>>,
    pub faction_city_icons: HashMap<GameFaction, Handle<Image>>,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    //debug!("1. Setting up map resource");
    let map = MapHandle(asset_server.load("maps/civilization.map.ron"));
    commands.insert_resource(map);
}

fn remove_random_place(places: &mut HashSet<String>) -> Option<String> {
    // Randomly pick an item from the HashSet
    let selected_place = places.iter().choose(&mut rand::rng()).cloned();

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
    serde::Deserialize, serde::Serialize, bevy::asset::Asset, bevy::reflect::TypePath, Clone, Debug,
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

fn load_map(
    mut commands: Commands,
    map: Res<MapHandle>,
    maps: Res<Assets<Map>>,
    mut available_factions: ResMut<AvailableFactions>,
    textures: Res<TextureAssets>,
    images: Res<Assets<Image>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut camera: Query<(&Camera, &mut Projection, &mut Transform), With<GameCamera>>,
) {
    //debug!("2. Loading map");
    if let Some(level) = maps.get(map.0.id()).clone() {
        let mut ancient_places: HashSet<String> = vec![
            "Assyria",
            "Numidia",
            "Carthage",
            "Troy",
            "Sparta",
            "Babylon",
            "Thebes",
            "Alexandria",
            "Athens",
            "Byzantium",
            "Pompeii",
            "Ephesus",
            "Ctesiphon",
            "Jerusalem",
            "Nineveh",
            "Sidon",
            "Tyre",
            "Memphis",
            "Heliopolis",
            "Pergamum",
            "Delphi",
            "Corinth",
            "Argos",
            "Syracuse",
            "Palmyra",
            "Damascus",
            "Antioch",
            "Petra",
            "Gadara",
            "Sidonia",
            "Susa",
            "Knossos",
            "Rhodes",
            "Pella",
            "Gortyn",
            "Leptis Magna",
            "Cyrene",
            "Tingis",
            "Volubilis",
            "Utica",
            "Sabratha",
            "Tanais",
            "Amarna",
            "Hattusa",
            "Ugarit",
            "Mari",
            "Arpad",
            "Qatna",
            "Alalakh",
            "Emar",
            "Aleppo",
            "Homs",
            "Edessa",
            "Tarsus",
            "Miletus",
            "Pergamon",
            "Amphipolis",
            "Mycenae",
            "Abydos",
            "Phaselis",
            "Halicarnassus",
            "Hierapolis",
            "Sardis",
            "Perge",
            "Gades",
            "Saguntum",
            "Tarraco",
            "Corduba",
            "Emerita Augusta",
            "Hispalis",
            "Lusitania",
            "Aquae Sulis",
            "Lutetia",
            "Massilia",
            "Nemausus",
            "Arelate",
            "Arretium",
            "Capua",
            "Neapolis",
            "Ravenna",
            "Tarentum",
            "Brundisium",
            "Venusia",
            "Cremona",
            "Mediolanum",
            "Patavium",
            "Aquileia",
            "Polis",
            "Teotoburgum",
            "Vindobona",
            "Carnuntum",
            "Sirmium",
            "Trebizond",
            "Chalcedon",
            "Nicopolis",
            "Heraclea",
            "Philippi",
            "Beroea",
            "Dura-Europos",
            "Seleucia",
            "Apamea",
            "Raphia",
            "Avaris",
            "Tanis",
            "Bubastis",
            "Herakleopolis",
            "Olynthus",
            "Thapsus",
            "Bulla Regia",
            "Hippo Regius",
            "Lepcis Magna",
            "Cirta",
            "Timgad",
            "Zama",
            "Thugga",
            "Kart Hadasht",
            "Rhegium",
            "Croton",
            "Selinus",
            "Acragas",
            "Himera",
            "Naxos",
            "Messina",
            "Segesta",
            "Catana",
            "Syracuse",
            "Thasos",
            "Amphipolis",
            "Potidaea",
            "Apollonia",
            "Abdera",
            "Athribis",
            "Berenice",
            "Oxyrhynchus",
            "Hermopolis",
            "Canopus",
            "Thonis",
            "Heracleion",
            "Marsa Matruh",
            "Baalbek",
            "Ebla",
            "Arwad",
            "Ashkelon",
            "Ascalon",
            "Gaza",
            "Megiddo",
            "Joppa",
            "Beersheba",
            "Hebron",
            "Aelia Capitolina",
            "Neapolis",
            "Hierapolis",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect();

        let map_center = Vec3::new(1250.0, 662.5, 0.0);
        let (_cam, mut projection, mut transform) = camera.single_mut().unwrap();
        transform.translation = map_center;

        if let (Some(img), Ok(window)) = (images.get(&textures.map), windows.single()) {
            let img_size = img.texture_descriptor.size;
            let map_size = Vec2::new(img_size.width as f32, img_size.height as f32);
            let window_size = Vec2::new(window.resolution.width(), window.resolution.height());
            fit_camera_to_map(map_size, window_size, &mut projection);
            commands.insert_resource(MapViewConfig { map_size, map_center });
        }

        commands.spawn((
            Sprite {
                image: textures.map.clone(),

                ..Default::default()
            },
            Transform::from_xyz(1250.0, 662.5, -1.0),
        ));

        for area in level.areas.clone() {
            let n = remove_random_place(&mut ancient_places).unwrap_or("STANDARD_NAME".to_string());

            let entity = commands
                .spawn((
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
                ))
                .id();
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
                        available_factions
                            .faction_icons
                            .insert(GameFaction::Egypt, textures.egypt.clone());
                        available_factions
                            .faction_city_icons
                            .insert(GameFaction::Egypt, textures.egypt_city.clone());
                    }
                    GameFaction::Crete => {
                        available_factions
                            .faction_icons
                            .insert(GameFaction::Crete, textures.crete.clone());
                        available_factions
                            .faction_city_icons
                            .insert(GameFaction::Crete, textures.crete_city.clone());
                    }
                    GameFaction::Africa => {
                        available_factions
                            .faction_icons
                            .insert(GameFaction::Africa, textures.africa.clone());
                        available_factions
                            .faction_city_icons
                            .insert(GameFaction::Africa, textures.africa_city.clone());
                    }
                    GameFaction::Asia => {
                        available_factions
                            .faction_icons
                            .insert(GameFaction::Asia, textures.asia.clone());
                        available_factions
                            .faction_city_icons
                            .insert(GameFaction::Asia, textures.asia_city.clone());
                    }
                    GameFaction::Assyria => {
                        available_factions
                            .faction_icons
                            .insert(GameFaction::Assyria, textures.assyria.clone());
                        available_factions
                            .faction_city_icons
                            .insert(GameFaction::Assyria, textures.assyria_city.clone());
                    }
                    GameFaction::Babylon => {
                        available_factions
                            .faction_icons
                            .insert(GameFaction::Babylon, textures.babylon.clone());
                        available_factions
                            .faction_city_icons
                            .insert(GameFaction::Babylon, textures.babylon_city.clone());
                    }
                    GameFaction::Illyria => {
                        available_factions
                            .faction_icons
                            .insert(GameFaction::Illyria, textures.illyria.clone());
                        available_factions
                            .faction_city_icons
                            .insert(GameFaction::Illyria, textures.illyria_city.clone());
                    }
                    GameFaction::Iberia => {
                        available_factions
                            .faction_icons
                            .insert(GameFaction::Iberia, textures.iberia.clone());
                        available_factions
                            .faction_city_icons
                            .insert(GameFaction::Iberia, textures.iberia_city.clone());
                    }
                    GameFaction::Thrace => {
                        available_factions
                            .faction_icons
                            .insert(GameFaction::Thrace, textures.thrace.clone());
                        available_factions
                            .faction_city_icons
                            .insert(GameFaction::Thrace, textures.thrace_city.clone());
                    }
                }
            }
        }
    }
}

fn fit_map_camera_on_resize(
    mut window_resized: MessageReader<WindowResized>,
    map_view: Option<Res<MapViewConfig>>,
    mut camera: Query<(&mut Projection, &mut Transform), With<GameCamera>>,
) {
    let Some(map_view) = map_view else {
        return;
    };

    let mut last_size: Option<Vec2> = None;
    for ev in window_resized.read() {
        last_size = Some(Vec2::new(ev.width, ev.height));
    }
    let Some(window_size) = last_size else {
        return;
    };

    if let Ok((mut projection, mut transform)) = camera.single_mut() {
        transform.translation = map_view.map_center;
        fit_camera_to_map(map_view.map_size, window_size, &mut projection);
    }
}

fn handle_map_camera_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut camera: Query<(&mut Projection, &mut Transform), With<GameCamera>>,
) {
    let Ok((mut projection, mut transform)) = camera.single_mut() else {
        return;
    };
    
    let Projection::Orthographic(ref mut ortho) = *projection else {
        return;
    };
    
    let dt = time.delta_secs();
    
    // Zoom controls: Z to zoom in, X to zoom out
    let zoom_speed = 1.5;
    let min_scale = 0.2;
    let max_scale = 3.0;
    
    if keyboard.pressed(KeyCode::KeyZ) {
        ortho.scale = (ortho.scale / (1.0 + zoom_speed * dt)).max(min_scale);
    }
    if keyboard.pressed(KeyCode::KeyX) {
        ortho.scale = (ortho.scale * (1.0 + zoom_speed * dt)).min(max_scale);
    }
    
    // Pan controls: Arrow keys
    let pan_speed = 500.0 * ortho.scale; // Scale pan speed with zoom level
    
    if keyboard.pressed(KeyCode::ArrowUp) {
        transform.translation.y += pan_speed * dt;
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        transform.translation.y -= pan_speed * dt;
    }
    if keyboard.pressed(KeyCode::ArrowLeft) {
        transform.translation.x -= pan_speed * dt;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        transform.translation.x += pan_speed * dt;
    }
}
