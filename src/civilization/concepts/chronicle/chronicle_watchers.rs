//! Chronicle entries that need no help from the rules code: they watch the
//! board and say what changed. Nothing can take a city without
//! [`on_city_discarded`] noticing, whatever the reason.

use super::chronicle_core::{Chronicle, area_label, player_label};
use crate::GameState;
use crate::civilization::components::{BuiltCity, Faction, PlayerAreas, PlayerCities};
use crate::civilization::concepts::resolve_calamities::context::ActiveCalamityResolution;
use crate::civilization::concepts::resolve_calamities::resolve_calamities_events::CalamityResolved;
use crate::civilization::concepts::save_game::LoadingFromSave;
use crate::player::Player;
use bevy::platform::collections::HashMap;
use bevy::prelude::{
    Added, Discard, Entity, Insert, MessageReader, MessageWriter, Name, On, Query, Res, ResMut,
    Resource, State, With,
};

/// City changes seen this frame, per area: who lost one and who got one.
#[derive(Resource, Default, Debug)]
pub struct CityChanges(Vec<(Entity, Option<Entity>, Option<Entity>)>);

impl CityChanges {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn entry(&mut self, area: Entity) -> &mut (Entity, Option<Entity>, Option<Entity>) {
        if let Some(i) = self.0.iter().position(|(a, ..)| *a == area) {
            &mut self.0[i]
        } else {
            self.0.push((area, None, None));
            self.0.last_mut().expect("just pushed")
        }
    }
}

pub fn on_city_discarded(
    trigger: On<Discard<BuiltCity>>,
    cities: Query<&BuiltCity>,
    mut changes: ResMut<CityChanges>,
) {
    let area = trigger.event().entity;
    if let Ok(city) = cities.get(area) {
        let entry = changes.entry(area);
        // Lost and rebuilt by the same player within a frame is no news.
        entry.1.get_or_insert(city.player);
    }
}

pub fn on_city_inserted(
    trigger: On<Insert<BuiltCity>>,
    cities: Query<&BuiltCity>,
    mut changes: ResMut<CityChanges>,
) {
    let area = trigger.event().entity;
    if let Ok(city) = cities.get(area) {
        changes.entry(area).2 = Some(city.player);
    }
}

/// What one area's city change reads as, if it is news at all.
pub fn describe_city_change(
    lost_by: Option<&str>,
    gained_by: Option<&str>,
    area: &str,
) -> Option<Chronicle> {
    match (lost_by, gained_by) {
        (Some(old), Some(new)) if old == new => None,
        (Some(old), Some(new)) => Some(Chronicle::toast(format!(
            "{new} took over {old}'s city in {area}"
        ))),
        (Some(old), None) => Some(Chronicle::toast(format!("{old} lost their city in {area}"))),
        (None, Some(new)) => Some(Chronicle::log(format!("{new} built a city in {area}"))),
        (None, None) => None,
    }
}

pub fn chronicle_city_changes(
    mut changes: ResMut<CityChanges>,
    state: Option<Res<State<GameState>>>,
    loading: Option<Res<LoadingFromSave>>,
    factions: Query<&Faction>,
    names: Query<&Name>,
    mut chronicle: MessageWriter<Chronicle>,
) {
    if changes.0.is_empty() {
        return;
    }
    let changes = std::mem::take(&mut changes.0);
    // Loading a save builds every city; leaving the game despawns them.
    let playing = state.is_some_and(|s| *s.get() == GameState::Playing);
    if !playing || loading.is_some() {
        return;
    }
    for (area, lost_by, gained_by) in changes {
        let lost_by = lost_by.map(|p| player_label(p, &factions));
        let gained_by = gained_by.map(|p| player_label(p, &factions));
        let area_name = area_label(area, &names);
        if let Some(entry) =
            describe_city_change(lost_by.as_deref(), gained_by.as_deref(), &area_name)
        {
            chronicle.write(entry.at(area));
        }
    }
}

/// Each player's tokens on the board and cities.
pub type Tally = HashMap<Entity, (usize, usize)>;

pub fn take_tally(players: &Query<(Entity, &PlayerAreas, &PlayerCities), With<Player>>) -> Tally {
    players
        .iter()
        .map(|(p, areas, cities)| (p, (areas.total_population(), cities.number_of_cities())))
        .collect()
}

/// "Egypt: -2 tokens; Thrace: -1 city, -4 tokens", by name, or `None`
/// when nobody's board changed. Reduced cities leave tokens behind, so
/// token counts can go up.
pub fn describe_tally_change(
    before: &Tally,
    after: &Tally,
    label: impl Fn(Entity) -> String,
) -> Option<String> {
    let mut parts: Vec<String> = before
        .keys()
        .filter_map(|p| {
            let (t0, c0) = before[p];
            let (t1, c1) = after.get(p).copied().unwrap_or((0, 0));
            let mut bits = Vec::new();
            let dc = c1 as i64 - c0 as i64;
            let dt = t1 as i64 - t0 as i64;
            if dc != 0 {
                bits.push(format!(
                    "{dc:+} {}",
                    if dc.abs() == 1 { "city" } else { "cities" }
                ));
            }
            if dt != 0 {
                bits.push(format!(
                    "{dt:+} {}",
                    if dt.abs() == 1 { "token" } else { "tokens" }
                ));
            }
            (!bits.is_empty()).then(|| format!("{}: {}", label(*p), bits.join(", ")))
        })
        .collect();
    parts.sort();
    (!parts.is_empty()).then(|| parts.join("; "))
}

/// The calamity being resolved and the board before it struck.
#[derive(Resource, Default)]
pub struct CalamityTally {
    open: Option<OpenCalamity>,
}

struct OpenCalamity {
    title: String,
    before: Tally,
    /// Frames left before the damage is counted, once it is resolved. Its
    /// effects are markers other systems act on over the next frame or two.
    closing_in: Option<u8>,
}

const SETTLE_FRAMES: u8 = 3;

fn close(
    open: OpenCalamity,
    players: &Query<(Entity, &PlayerAreas, &PlayerCities), With<Player>>,
    factions: &Query<&Faction>,
    chronicle: &mut MessageWriter<Chronicle>,
) {
    let after = take_tally(players);
    let damage = describe_tally_change(&open.before, &after, |p| player_label(p, factions));
    chronicle.write(Chronicle::toast(match damage {
        Some(damage) => format!("{} is over. {damage}", open.title),
        None => format!("{} is over. Nobody lost anything", open.title),
    }));
}

pub fn chronicle_calamities(
    mut tally: ResMut<CalamityTally>,
    started: Query<&ActiveCalamityResolution, Added<ActiveCalamityResolution>>,
    mut resolved: MessageReader<CalamityResolved>,
    players: Query<(Entity, &PlayerAreas, &PlayerCities), With<Player>>,
    factions: Query<&Faction>,
    mut chronicle: MessageWriter<Chronicle>,
) {
    if resolved.read().count() > 0
        && let Some(open) = tally.open.as_mut()
    {
        open.closing_in.get_or_insert(SETTLE_FRAMES);
    }
    for resolution in &started {
        if let Some(open) = tally.open.take() {
            close(open, &players, &factions, &mut chronicle);
        }
        let context = &resolution.context;
        let victim = player_label(context.primary_victim, &factions);
        let title = format!("{} ({victim})", context.calamity);
        let traded = context.traded_by.map_or_else(String::new, |by| {
            format!(", traded by {}", player_label(by, &factions))
        });
        chronicle.write(Chronicle::toast(format!(
            "Calamity! {} strikes {victim}{traded}",
            context.calamity
        )));
        tally.open = Some(OpenCalamity {
            title,
            before: take_tally(&players),
            closing_in: None,
        });
    }
    let due = tally
        .open
        .as_mut()
        .is_some_and(|open| match &mut open.closing_in {
            Some(0) => true,
            Some(n) => {
                *n -= 1;
                false
            }
            None => false,
        });
    if due && let Some(open) = tally.open.take() {
        close(open, &players, &factions, &mut chronicle);
    }
}

/// A calamity still open when the phase ends is counted then.
pub fn close_calamity_on_exit(
    mut tally: ResMut<CalamityTally>,
    players: Query<(Entity, &PlayerAreas, &PlayerCities), With<Player>>,
    factions: Query<&Faction>,
    mut chronicle: MessageWriter<Chronicle>,
) {
    if let Some(open) = tally.open.take() {
        close(open, &players, &factions, &mut chronicle);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civilization::concepts::chronicle::chronicle_core::Importance;

    #[test]
    fn city_changes_read_as_news() {
        let took = describe_city_change(Some("Egypt"), Some("Thrace"), "Thebes").unwrap();
        assert_eq!(took.text, "Thrace took over Egypt's city in Thebes");
        assert_eq!(took.importance, Importance::Toast);
        let lost = describe_city_change(Some("Egypt"), None, "Thebes").unwrap();
        assert_eq!(lost.text, "Egypt lost their city in Thebes");
        let built = describe_city_change(None, Some("Egypt"), "Thebes").unwrap();
        assert_eq!(built.importance, Importance::Log);
        assert!(describe_city_change(Some("Egypt"), Some("Egypt"), "Thebes").is_none());
    }

    #[test]
    fn tally_changes_name_who_lost_what() {
        let (a, b, c) = (
            Entity::from_raw_u32(1).unwrap(),
            Entity::from_raw_u32(2).unwrap(),
            Entity::from_raw_u32(3).unwrap(),
        );
        let before: Tally = [(a, (10, 3)), (b, (5, 0)), (c, (4, 1))]
            .into_iter()
            .collect();
        let after: Tally = [(a, (13, 2)), (b, (4, 0)), (c, (4, 1))]
            .into_iter()
            .collect();
        let label =
            |p: Entity| ["", "Thrace", "Egypt", "Crete"][p.index_u32() as usize].to_string();
        assert_eq!(
            describe_tally_change(&before, &after, label).unwrap(),
            "Egypt: -1 token; Thrace: -1 city, +3 tokens"
        );
        assert!(describe_tally_change(&before, &before, label).is_none());
    }
}
