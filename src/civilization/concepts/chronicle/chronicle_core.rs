//! The game's story: one sentence per notable thing that happened.
//!
//! Anything can write a [`Chronicle`]; [`record_chronicle`] is the single
//! sink. Every entry goes to the console as `[CHRONICLE]` and, when the app
//! has a [`ChronicleFile`], to this session's chronicle log on disk.
//! [`Importance::Toast`] entries also go to the round summary pane and the
//! [`NarrationQueue`], which the narration UI replays one at a time.

use crate::GameActivity;
use crate::civilization::components::Faction;
use crate::civilization::concepts::census::GameInfoAndStuff;
use crate::civilization::concepts::round_summary::RoundSummary;
use bevy::prelude::{
    Entity, Message, MessageReader, Name, Query, Res, ResMut, Resource, State, info,
};
use std::collections::VecDeque;
use std::io::Write;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Importance {
    /// Written to the log only.
    Log,
    /// Also shown to the player.
    Toast,
}

#[derive(Message, Clone, Debug)]
pub struct Chronicle {
    pub text: String,
    /// Where it happened, for the camera.
    pub area: Option<Entity>,
    pub importance: Importance,
}

impl Chronicle {
    pub fn toast(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            area: None,
            importance: Importance::Toast,
        }
    }

    pub fn log(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            area: None,
            importance: Importance::Log,
        }
    }

    #[must_use]
    pub fn at(mut self, area: Entity) -> Self {
        self.area = Some(area);
        self
    }
}

/// A toast waiting to be shown, with when it happened.
#[derive(Clone, Debug)]
pub struct Narration {
    pub entry: Chronicle,
    /// "Round 10 · Collect Taxes"
    pub when: String,
}

/// Toasts waiting to be shown, oldest first.
#[derive(Resource, Default, Debug)]
pub struct NarrationQueue(pub VecDeque<Narration>);

/// This session's chronicle log. Only the game binary inserts one, so tests
/// and the headless server do not write files.
#[derive(Resource)]
pub struct ChronicleFile(pub std::fs::File);

/// "CollectTaxes" -> "Collect Taxes".
pub fn phase_label(activity: &GameActivity) -> String {
    let raw = format!("{activity:?}");
    let mut out = String::with_capacity(raw.len() + 4);
    for (i, c) in raw.chars().enumerate() {
        if i > 0 && c.is_uppercase() {
            out.push(' ');
        }
        out.push(c);
    }
    out
}

/// A player as the story names them: their faction, or "Pirates".
pub fn player_label(player: Entity, factions: &Query<&Faction>) -> String {
    factions
        .get(player)
        .map_or_else(|_| "Pirates".to_string(), |f| f.faction.to_string())
}

/// Areas are named "133:Nile Delta"; the story only wants "Nile Delta".
pub fn short_area_name(name: &Name) -> String {
    let n = name.as_str();
    n.split_once(':').map_or(n, |(_, rest)| rest).to_string()
}

pub fn area_label(area: Entity, names: &Query<&Name>) -> String {
    names
        .get(area)
        .map_or_else(|_| format!("{area:?}"), short_area_name)
}

/// "Battle in Thrace: Illyria 5 vs Thrace 4. Illyria keeps 2, Thrace is
/// wiped out". `sides` is (player, tokens before, tokens after), largest
/// army first.
pub fn describe_battle(area: &str, sides: &[(String, usize, usize)]) -> String {
    let armies: Vec<String> = sides
        .iter()
        .map(|(p, before, _)| format!("{p} {before}"))
        .collect();
    let outcomes: Vec<String> = sides
        .iter()
        .map(|(p, _, after)| match after {
            0 => format!("{p} is wiped out"),
            n => format!("{p} keeps {n}"),
        })
        .collect();
    format!(
        "Battle in {area}: {}. {}",
        armies.join(" vs "),
        outcomes.join(", ")
    )
}

pub fn record_chronicle(
    mut entries: MessageReader<Chronicle>,
    activity: Option<Res<State<GameActivity>>>,
    game_info: Option<Res<GameInfoAndStuff>>,
    mut summary: Option<ResMut<RoundSummary>>,
    mut queue: Option<ResMut<NarrationQueue>>,
    mut file: Option<ResMut<ChronicleFile>>,
) {
    for entry in entries.read() {
        let round = game_info.as_ref().map_or(0, |g| g.round);
        let phase = activity
            .as_ref()
            .map_or_else(String::new, |a| phase_label(a.get()));
        info!("[CHRONICLE] R{round} {phase}: {}", entry.text);
        if let Some(file) = file.as_mut() {
            let _ = writeln!(file.0, "R{round:<3} {phase:<28} {}", entry.text);
        }
        if entry.importance == Importance::Toast {
            if let Some(summary) = summary.as_mut() {
                summary.push(entry.text.clone());
            }
            if let Some(queue) = queue.as_mut() {
                queue.0.push_back(Narration {
                    entry: entry.clone(),
                    when: format!("Round {round} · {phase}"),
                });
            }
        }
    }
}

/// Session log files: `logs/<UTC date>_<time>-<kind>.log`, all with the
/// same stamp so one game's logs sort together. `None` where there is no
/// file system (the web build) or it cannot be written.
pub fn create_session_log(kind: &str) -> Option<std::fs::File> {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = kind;
        None
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        static STAMP: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        let stamp = STAMP.get_or_init(|| {
            let secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_secs());
            utc_stamp(secs)
        });
        std::fs::create_dir_all("logs").ok()?;
        std::fs::File::create(format!("logs/{stamp}-{kind}.log")).ok()
    }
}

/// `2026-11-26_15-44-26` for seconds since the Unix epoch (UTC).
fn utc_stamp(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    // Howard Hinnant's days-to-civil.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}_{:02}-{:02}-{:02}",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::{Messages, World};

    #[test]
    fn stamps_are_utc_dates() {
        assert_eq!(utc_stamp(0), "1970-01-01_00-00-00");
        assert_eq!(utc_stamp(1_795_707_866), "2026-11-26_15-44-26");
        assert_eq!(utc_stamp(951_782_400), "2000-02-29_00-00-00");
    }

    #[test]
    fn battles_say_who_fought_and_who_is_left() {
        let sides = [("Illyria".to_string(), 5, 2), ("Thrace".to_string(), 4, 0)];
        assert_eq!(
            describe_battle("Thrace", &sides),
            "Battle in Thrace: Illyria 5 vs Thrace 4. Illyria keeps 2, Thrace is wiped out"
        );
    }

    #[test]
    fn phases_read_as_words() {
        assert_eq!(phase_label(&GameActivity::CollectTaxes), "Collect Taxes");
        assert_eq!(
            phase_label(&GameActivity::CheckCitySupportAfterResolveCalamities),
            "Check City Support After Resolve Calamities"
        );
    }

    #[test]
    fn toasts_are_queued_and_summarised_but_log_lines_are_not() {
        let mut world = World::new();
        world.init_resource::<Messages<Chronicle>>();
        world.init_resource::<NarrationQueue>();
        world.init_resource::<RoundSummary>();
        world.insert_resource(State::new(GameActivity::CollectTaxes));
        world.insert_resource(GameInfoAndStuff {
            round: 10,
            ..Default::default()
        });
        let area = world.spawn_empty().id();
        let mut messages = world.resource_mut::<Messages<Chronicle>>();
        messages.write(Chronicle::log("Thrace built a city"));
        messages.write(Chronicle::toast("8 of Egypt's cities revolt").at(area));

        world.run_system_once(record_chronicle).unwrap();

        let queue = &world.resource::<NarrationQueue>().0;
        assert_eq!(queue.len(), 1);
        assert_eq!(queue[0].entry.area, Some(area));
        assert_eq!(queue[0].when, "Round 10 · Collect Taxes");
        assert_eq!(
            world.resource::<RoundSummary>().entries(),
            ["8 of Egypt's cities revolt"]
        );
    }
}
