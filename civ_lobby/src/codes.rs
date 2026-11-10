//! Readable game codes: `adjective-animal-NN`, e.g. `brisk-otter-42`.
//! 40 × 40 × 90 = 144 000 codes: easy to read aloud, hard enough to guess
//! for an invite-your-friends site (the link is a bearer capability, see
//! docs/multiplayer.md).

use rand::Rng;
use rand::seq::IndexedRandom;

const ADJECTIVES: [&str; 40] = [
    "amber", "bold", "brave", "brisk", "bronze", "calm", "clever", "copper", "crimson", "dusty",
    "eager", "fierce", "gentle", "golden", "grand", "hidden", "honest", "jolly", "keen", "lucky",
    "mighty", "misty", "noble", "olive", "proud", "quick", "quiet", "royal", "rusty", "sandy",
    "silent", "silver", "steady", "stormy", "sunny", "swift", "tidy", "wild", "wise", "young",
];

const ANIMALS: [&str; 40] = [
    "ant", "badger", "bear", "bison", "camel", "cobra", "crane", "crow", "deer", "dolphin",
    "eagle", "falcon", "fox", "gecko", "goat", "hare", "hawk", "heron", "horse", "ibis",
    "jackal", "lion", "lynx", "mole", "moose", "otter", "owl", "panther", "raven", "seal",
    "shark", "snake", "stork", "tiger", "toad", "viper", "walrus", "whale", "wolf", "yak",
];

pub fn generate(rng: &mut impl Rng) -> String {
    let adjective = ADJECTIVES.choose(rng).copied().unwrap_or("brisk");
    let animal = ANIMALS.choose(rng).copied().unwrap_or("otter");
    let number: u8 = rng.random_range(10..100);
    format!("{adjective}-{animal}-{number}")
}

/// Accepts what `generate` produces (and nothing that could escape a path):
/// lowercase ASCII letters, digits and hyphens, at most 40 characters.
/// Also tolerates the user typing it in upper case or with spaces around it.
pub fn normalize(input: &str) -> Option<String> {
    let code = input.trim().to_ascii_lowercase();
    let valid = !code.is_empty()
        && code.len() <= 40
        && code
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    valid.then_some(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_codes_are_valid() {
        let mut rng = rand::rng();
        for _ in 0..200 {
            let code = generate(&mut rng);
            assert_eq!(normalize(&code).as_deref(), Some(code.as_str()));
            assert_eq!(code.split('-').count(), 3);
        }
    }

    #[test]
    fn normalize_rejects_path_tricks_and_accepts_sloppy_typing() {
        assert_eq!(normalize("  Brisk-Otter-42 ").as_deref(), Some("brisk-otter-42"));
        assert_eq!(normalize("../etc"), None);
        assert_eq!(normalize("a/b"), None);
        assert_eq!(normalize(""), None);
    }
}
