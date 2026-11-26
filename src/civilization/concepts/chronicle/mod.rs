mod chronicle_core;
mod chronicle_watchers;
mod narration_ui;

pub use chronicle_core::*;
pub use chronicle_watchers::*;
pub use narration_ui::*;

use crate::{GameActivity, GameState};
use bevy::prelude::{App, IntoScheduleConfigs, Last, OnExit, Plugin, Res, State, Update, in_state};

/// Run condition: the game is in one of the replay phases.
pub fn in_replay(activity: Option<Res<State<GameActivity>>>) -> bool {
    activity.is_some_and(|a| {
        matches!(
            a.get(),
            GameActivity::BattleReplay | GameActivity::CalamityReplay
        )
    })
}

/// The game's story: collects [`Chronicle`] entries, watches the board for
/// city changes and calamities, and writes the log. No UI.
pub struct ChroniclePlugin;

impl Plugin for ChroniclePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<Chronicle>()
            .init_resource::<NarrationQueue>()
            .init_resource::<CityChanges>()
            .init_resource::<CalamityTally>()
            .add_observer(on_city_discarded)
            .add_observer(on_city_inserted)
            .add_systems(
                Update,
                chronicle_calamities.run_if(in_state(GameActivity::ResolveCalamities)),
            )
            .add_systems(
                OnExit(GameActivity::ResolveCalamities),
                close_calamity_on_exit,
            )
            .add_systems(Last, (chronicle_city_changes, record_chronicle).chain())
            .add_systems(Update, finish_replay.run_if(in_replay));
    }
}

/// Shows the chronicle's toasts to the player.
pub struct NarrationUiPlugin;

impl Plugin for NarrationUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Narrator>()
            .add_systems(Update, play_narration.run_if(in_state(GameState::Playing)))
            .add_systems(Update, skip_replay.run_if(in_replay))
            .add_systems(OnExit(GameState::Playing), clear_narration);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civilization::components::{BuiltCity, Faction};
    use crate::civilization::enums::GameFaction;
    use bevy::prelude::{AppExtStates, Name};
    use bevy::state::app::StatesPlugin;

    fn told(app: &mut App) -> Vec<String> {
        app.world_mut()
            .resource_mut::<NarrationQueue>()
            .0
            .drain(..)
            .map(|n| n.entry.text)
            .collect()
    }

    /// The tax revolt that took 8 of Egypt's 9 cities said nothing at all.
    /// Whatever moves a city now leaves a line in the story.
    #[test]
    fn every_city_that_changes_hands_or_falls_is_told() {
        let mut app = App::new();
        app.add_plugins((StatesPlugin, ChroniclePlugin))
            .insert_state(GameState::Playing);
        let egypt = app.world_mut().spawn(Faction::new(GameFaction::Egypt)).id();
        let thrace = app
            .world_mut()
            .spawn(Faction::new(GameFaction::Thrace))
            .id();
        let area = app.world_mut().spawn(Name::new("133:Thebes")).id();
        let (city_a, city_b) = (
            app.world_mut().spawn_empty().id(),
            app.world_mut().spawn_empty().id(),
        );

        app.world_mut()
            .entity_mut(area)
            .insert(BuiltCity::new(city_a, egypt));
        app.update();
        assert!(told(&mut app).is_empty(), "building is logged, not toasted");

        app.world_mut()
            .entity_mut(area)
            .insert(BuiltCity::new(city_b, thrace));
        app.update();
        assert_eq!(told(&mut app), ["Thrace took over Egypt's city in Thebes"]);

        app.world_mut().entity_mut(area).remove::<BuiltCity>();
        app.update();
        assert_eq!(told(&mut app), ["Thrace lost their city in Thebes"]);
    }

    /// Conflict over, one battle's toast still to show.
    fn replay_app(narrator: bool) -> App {
        let mut app = App::new();
        app.add_plugins((StatesPlugin, ChroniclePlugin))
            .insert_state(GameState::Playing)
            .add_sub_state::<GameActivity>();
        if narrator {
            app.init_resource::<Narrator>();
        }
        app.world_mut()
            .resource_mut::<NarrationQueue>()
            .0
            .push_back(Narration {
                entry: Chronicle::toast("Battle in Thebes"),
                when: String::new(),
            });
        app.world_mut()
            .resource_mut::<bevy::prelude::NextState<GameActivity>>()
            .set(GameActivity::BattleReplay);
        app.update();
        app
    }

    fn activity(app: &App) -> GameActivity {
        app.world()
            .resource::<bevy::prelude::State<GameActivity>>()
            .get()
            .clone()
    }

    /// The replay waits for the last toast, then cities are built.
    #[test]
    fn a_battle_replay_lasts_until_the_story_is_told() {
        let mut app = replay_app(true);
        app.update();
        app.update();
        assert_eq!(activity(&app), GameActivity::BattleReplay);
        app.world_mut().resource_mut::<NarrationQueue>().0.clear();
        app.update();
        app.update();
        assert_eq!(activity(&app), GameActivity::CityConstruction);
    }

    /// Nobody watches the headless server: its replays pass at once.
    #[test]
    fn without_a_screen_the_replay_passes_at_once() {
        let mut app = replay_app(false);
        app.update();
        app.update();
        assert_eq!(activity(&app), GameActivity::CityConstruction);
    }

    #[test]
    fn cities_changing_outside_a_game_are_not_news() {
        let mut app = App::new();
        app.add_plugins((StatesPlugin, ChroniclePlugin))
            .insert_state(GameState::Menu);
        let egypt = app.world_mut().spawn(Faction::new(GameFaction::Egypt)).id();
        let area = app.world_mut().spawn(Name::new("1:Thebes")).id();
        let city = app.world_mut().spawn_empty().id();
        app.world_mut()
            .entity_mut(area)
            .insert(BuiltCity::new(city, egypt));
        app.update();
        app.world_mut().despawn(area);
        app.update();
        assert_eq!(told(&mut app), Vec::<String>::new());
        assert!(app.world().resource::<CityChanges>().is_empty());
    }
}
