//! Headless boot of the *real* game: `CivLogicPlugins` (the full rules
//! engine, AI included) running under `MinimalPlugins` + `AssetPlugin`.
//!
//! The phase plugins still contain UI systems; they are inert here because
//! no windows/cameras produce work for them — but several read input/asset
//! resources unconditionally, so this plugin inserts inert stand-ins
//! (`ButtonInput`, `Touches`, `TextureAssets` with dead handles, `LavaTheme`).
//!
//! State flow: `Loading` (wait for map + civ-card RON assets) → `Menu`
//! (lobby; clients claim seats) → `Playing` (normal `GameActivity` flow).
//! Seats reserve their factions via `DebugOptions::reserved_factions`, and
//! `bind_seats` swaps those players from AI to human at `StartGame`.

use adv_civ::civilization::*;
use adv_civ::loading::TextureAssets;
use adv_civ::player::Player;
use adv_civ::stupid_ai::{AgentControlled, IsHuman, StupidAi};
use adv_civ::{GameActivity, GameState};
use adv_civ_protocol::GameFaction;
use bevy::asset::AssetPlugin;
use bevy::input::ButtonInput;
use bevy::input::touch::Touches;
use bevy::prelude::*;

/// Faction claim order for human seats.
const SEAT_FACTION_ORDER: [GameFaction; 9] = [
    GameFaction::Egypt,
    GameFaction::Crete,
    GameFaction::Africa,
    GameFaction::Asia,
    GameFaction::Assyria,
    GameFaction::Babylon,
    GameFaction::Illyria,
    GameFaction::Iberia,
    GameFaction::Thrace,
];

/// One playable human seat. `player` is bound once `setup_players` has
/// spawned the real player entities (at `StartGame`).
pub struct Seat {
    pub faction: GameFaction,
    pub player: Option<Entity>,
    /// The lightyear connection entity currently holding this seat.
    pub client: Option<Entity>,
    pub peer: Option<lightyear::prelude::PeerId>,
    pub name: Option<String>,
    /// Reconnect secret bound to this seat's identity, once one has been
    /// issued (see `find_seat_for_join`). Session-token hardening, not real
    /// security: this is a friends-and-family game, not an adversarial one —
    /// the goal is stopping a typo'd or guessed display name from silently
    /// stealing someone's seat, not defeating a determined attacker.
    pub reconnect_token: Option<String>,
}

#[derive(Resource, Default)]
pub struct Seats(pub Vec<Seat>);

impl Seats {
    pub fn by_client(&self, client: Entity) -> Option<&Seat> {
        self.0.iter().find(|s| s.client == Some(client))
    }

    pub fn by_player(&self, player: Entity) -> Option<&Seat> {
        self.0.iter().find(|s| s.player == Some(player))
    }

    pub fn all_claimed(&self) -> bool {
        self.0.iter().all(|s| s.client.is_some())
    }
}

/// Picks which seat a `(name, token)` join should claim.
///
/// A seat with no `reconnect_token` bound yet (never claimed, or claimed by
/// an old client that predates this) accepts a plain name match — that keeps
/// the "share a magic link, type your name" flow working for first-time
/// joins. Once a token *is* bound, reclaiming that specific identity requires
/// presenting the matching token; a name-only (or mismatched-token) join
/// falls through to a genuinely unclaimed seat instead of stealing it.
pub fn find_seat_for_join(seats: &[Seat], name: &str, token: Option<&str>) -> Option<usize> {
    seats
        .iter()
        .position(|s| {
            s.client.is_none()
                && s.name.as_deref() == Some(name)
                && match &s.reconnect_token {
                    None => true,
                    Some(bound) => Some(bound.as_str()) == token,
                }
        })
        .or_else(|| {
            seats
                .iter()
                .position(|s| s.client.is_none() && s.name.is_none())
        })
}

#[cfg(test)]
mod seat_join_tests {
    use super::*;

    fn seat(faction: GameFaction, name: Option<&str>, token: Option<&str>) -> Seat {
        Seat {
            faction,
            player: None,
            client: None,
            peer: None,
            name: name.map(str::to_string),
            reconnect_token: token.map(str::to_string),
        }
    }

    #[test]
    fn first_join_to_an_untouched_seat_matches_by_name_alone() {
        let seats = vec![seat(GameFaction::Egypt, Some("Alice"), None)];
        assert_eq!(find_seat_for_join(&seats, "Alice", None), Some(0));
    }

    #[test]
    fn reclaiming_a_token_bound_seat_requires_the_right_token() {
        let seats = vec![seat(GameFaction::Egypt, Some("Alice"), Some("secret"))];
        assert_eq!(
            find_seat_for_join(&seats, "Alice", Some("secret")),
            Some(0)
        );
    }

    #[test]
    fn a_stranger_typing_a_taken_name_without_the_token_does_not_steal_the_seat() {
        let seats = vec![
            seat(GameFaction::Egypt, Some("Alice"), Some("real-secret")),
            seat(GameFaction::Crete, None, None),
        ];
        // Wrong (or absent) token on a bound seat falls through to a fresh one
        // instead of claiming Alice's.
        assert_eq!(
            find_seat_for_join(&seats, "Alice", Some("guessed")),
            Some(1)
        );
        assert_eq!(find_seat_for_join(&seats, "Alice", None), Some(1));
    }

    #[test]
    fn no_seats_available_at_all_returns_none() {
        let seats = vec![seat(
            GameFaction::Egypt,
            Some("Alice"),
            Some("real-secret"),
        )];
        let mut taken = seats;
        taken[0].client = Some(Entity::from_raw_u32(1).unwrap());
        assert_eq!(find_seat_for_join(&taken, "Bob", None), None);
    }

    #[test]
    fn a_brand_new_name_claims_a_fresh_seat_directly() {
        let seats = vec![
            seat(GameFaction::Egypt, Some("Alice"), Some("real-secret")),
            seat(GameFaction::Crete, None, None),
        ];
        assert_eq!(find_seat_for_join(&seats, "Bob", None), Some(1));
    }
}

pub struct HeadlessGamePlugin;

impl Plugin for HeadlessGamePlugin {
    fn build(&self, app: &mut App) {
        // Seat/player counts: SEATS human seats (default 2; 0 = AI-only
        // self-play), NUM_PLAYERS total players (default 5); the difference
        // is AI-controlled.
        let human_seats = env_count("SEATS", 2).min(SEAT_FACTION_ORDER.len());
        let total_players = env_count("NUM_PLAYERS", 5).clamp(human_seats.max(1), 9);
        let seat_factions: Vec<GameFaction> = SEAT_FACTION_ORDER
            .iter()
            .copied()
            .take(human_seats)
            .collect();
        info!("Hosting {total_players} players, {human_seats} human seat(s): {seat_factions:?}");

        app.add_plugins(AssetPlugin::default())
            .init_asset::<Image>()
            // HUD-flavoured systems inside the phase plugins allocate font
            // handles for IsHuman players; the type must exist even though
            // nothing ever loads or renders text headless.
            .init_asset::<bevy::text::Font>()
            .add_plugins(CivLogicPlugins);

        // Override the env-derived defaults: no local human, our seat
        // factions reserved, no debug niceties.
        app.insert_resource(DebugOptions {
            add_human_player: false,
            number_of_players: total_players,
            reserved_factions: seat_factions.clone(),
            show_debug_ui: false,
            print_selected_moves: false,
            ..DebugOptions::default()
        });

        // Inert stand-ins for resources/messages that UI-flavoured systems
        // read; without windows or input devices they stay empty forever.
        app.init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<Touches>()
            .add_message::<bevy::window::WindowResized>()
            .init_resource::<bevy::gizmos::config::GizmoConfigStore>()
            .insert_resource(lava_ui_builder::LavaTheme::default())
            .insert_resource(dead_texture_assets());
        // Highlight systems draw via `Gizmos<DefaultGizmoConfigGroup>`;
        // register the group so the param validates (nothing renders it).
        {
            use bevy::gizmos::AppGizmoBuilder;
            app.init_asset::<bevy::gizmos::GizmoAsset>();
            app.init_gizmo_group::<bevy::gizmos::config::DefaultGizmoConfigGroup>();
        }

        app.insert_resource(Seats(
            seat_factions
                .into_iter()
                .map(|faction| Seat {
                    faction,
                    player: None,
                    client: None,
                    peer: None,
                    name: None,
                    reconnect_token: None,
                })
                .collect(),
        ));

        app.add_systems(Startup, spawn_dummy_camera)
            .add_systems(
                Update,
                open_lobby_when_assets_ready.run_if(in_state(GameState::Loading)),
            )
            .add_systems(OnEnter(GameState::Menu), start_if_no_seats)
            .add_systems(
                OnEnter(GameActivity::StartGame),
                bind_seats.before(start_game),
            );
    }
}

/// SEATS=0: nothing to wait for — pure AI self-play starts immediately.
fn start_if_no_seats(seats: Res<Seats>, mut next: ResMut<NextState<GameState>>) {
    if seats.0.is_empty() {
        info!("No human seats configured — starting AI self-play");
        next.set(GameState::Playing);
    }
}

fn env_count(var: &str, default: usize) -> usize {
    std::env::var(var)
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(default)
}

/// `load_map` unconditionally unwraps the GameCamera query; give it one.
/// Nothing renders, the projection is never used for real.
fn spawn_dummy_camera(mut commands: Commands) {
    commands.spawn((
        GameCamera,
        Camera::default(),
        Projection::default(),
        Transform::default(),
        Name::new("Headless dummy camera"),
    ));
}

/// All-dead handles: sprites spawned by game systems simply never resolve.
fn dead_texture_assets() -> TextureAssets {
    TextureAssets {
        bevy: Handle::default(),
        github: Handle::default(),
        map: Handle::default(),
        dot: Handle::default(),
        africa: Handle::default(),
        asia: Handle::default(),
        assyria: Handle::default(),
        babylon: Handle::default(),
        crete: Handle::default(),
        egypt: Handle::default(),
        iberia: Handle::default(),
        illyria: Handle::default(),
        thrace: Handle::default(),
        africa_city: Handle::default(),
        asia_city: Handle::default(),
        assyria_city: Handle::default(),
        babylon_city: Handle::default(),
        crete_city: Handle::default(),
        egypt_city: Handle::default(),
        iberia_city: Handle::default(),
        illyria_city: Handle::default(),
        thrace_city: Handle::default(),
        ship: Handle::default(),
    }
}

/// The map and civ-card decks load from RON in `assets/`; the lobby opens
/// once both are in. (`load_map` runs once on entering PrepareGame — if the
/// assets weren't loaded by then the board would silently be empty.)
fn open_lobby_when_assets_ready(
    maps: Res<Assets<Map>>,
    civ_cards: Res<Assets<AvailableCivCards>>,
    mut next: ResMut<NextState<GameState>>,
) {
    if !maps.is_empty() && !civ_cards.is_empty() {
        info!("Assets loaded — lobby open");
        next.set(GameState::Menu);
    }
}

/// Once `setup_players` has spawned the player entities (PrepareGame), bind
/// each seat to the player of its reserved faction and make it human: the
/// AI stops driving it, and the phase gates wait for the remote player.
fn bind_seats(
    mut seats: ResMut<Seats>,
    players: Query<(Entity, &Faction), With<Player>>,
    mut commands: Commands,
) {
    for seat in seats.0.iter_mut() {
        let Some((player, _)) = players.iter().find(|(_, f)| f.faction == seat.faction) else {
            error!(
                "No player spawned for reserved seat faction {}",
                seat.faction
            );
            continue;
        };
        seat.player = Some(player);
        let mut entity = commands.entity(player);
        // IsHuman: the phase gates wait for this player instead of the AI
        // driving it. AgentControlled: phases without a remote endpoint yet
        // (ship placement) route down the AI auto path instead of waiting
        // for a local UI that doesn't exist — same trick the agent API uses.
        entity
            .remove::<StupidAi>()
            .insert((IsHuman, AgentControlled));
        if let Some(name) = &seat.name {
            entity.insert(Name::new(name.clone()));
        }
        info!("Seat {} bound to player {player:?}", seat.faction);
    }
}
