use bevy::prelude::Resource;

/// Short, human-readable log of "what happened this round", collected as
/// phases run, for the Game Info / Round Info HUD pane (see
/// `docs/roadmap.md`'s "Phases and Summaries" wishlist). Cleared at the
/// start of each round on `OnEnter(GameActivity::CollectTaxes)`, the first
/// phase in the round order (`GameActivity` in `lib.rs`).
#[derive(Resource, Default, Debug, Clone)]
pub struct RoundSummary {
    entries: Vec<String>,
}

impl RoundSummary {
    pub fn push(&mut self, line: impl Into<String>) {
        self.entries.push(line.into());
    }

    pub fn entries(&self) -> &[String] {
        &self.entries
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_empty() {
        assert!(RoundSummary::default().entries().is_empty());
    }

    #[test]
    fn push_appends_in_order() {
        let mut summary = RoundSummary::default();
        summary.push("first");
        summary.push(format!("second {}", 2));
        assert_eq!(summary.entries(), ["first", "second 2"]);
    }

    #[test]
    fn clear_empties_it() {
        let mut summary = RoundSummary::default();
        summary.push("something happened");
        summary.clear();
        assert!(summary.entries().is_empty());
    }
}
