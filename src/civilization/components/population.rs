use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::{Component, Entity, Reflect, default};

#[derive(Component, Debug, Reflect, Default)]
pub struct Population {
    player_tokens: HashMap<Entity, HashSet<Entity>>,
    pub max_population: usize,
}

impl Population {
    pub fn new(max_population: usize) -> Self {
        Population {
            max_population,
            ..default()
        }
    }

    pub fn player_tokens(&self) -> &HashMap<Entity, HashSet<Entity>> {
        &self.player_tokens
    }

    pub fn tokens_for_player(&self, player: &Entity) -> Option<&HashSet<Entity>> {
        self.player_tokens.get(player)
    }

    pub fn has_more_than_one_player(&self) -> bool {
        self.player_tokens.keys().len() > 1
    }

    pub fn players(&self) -> HashSet<Entity> {
        self.player_tokens.keys().cloned().collect()
    }

    pub fn number_of_players(&self) -> usize {
        self.player_tokens.keys().len()
    }

    pub fn all_lengths_equal(&self) -> bool {
        let first_length = self.player_tokens.values().next().map(|v| v.len());
        self.player_tokens
            .values()
            .all(|v| Some(v.len()) == first_length)
    }

    pub fn remove_surplus(&mut self) -> HashSet<Entity> {
        assert_eq!(self.number_of_players(), 1); // this should never, ever, not happen
        let surplus_count = self.surplus_count();

        let player_tokens = self.player_tokens.values_mut().next().unwrap();
        let tokens: HashSet<Entity> = player_tokens.iter().take(surplus_count).copied().collect();

        for token in tokens.iter() {
            player_tokens.remove(token);
        }
        tokens
    }

    pub fn remove_all_tokens(&mut self) -> HashSet<Entity> {
        let mut flattened_set = HashSet::new();

        for set in self.player_tokens.clone().into_values() {
            flattened_set.extend(set);
        }
        self.player_tokens.clear();

        flattened_set
    }

    pub fn remove_all_tokens_for_player(&mut self, player: &Entity) -> HashSet<Entity> {
        self.player_tokens.remove(player).unwrap_or_default()
    }

    pub fn has_surplus(&self, has_city: bool) -> bool {
        (has_city && self.has_population()) || self.surplus_count() > 0
    }

    pub fn surplus_count(&self) -> usize {
        if self.total_population() > self.max_population {
            self.total_population() - self.max_population
        } else {
            0
        }
    }

    pub fn is_conflict_zone(&self, has_city: bool) -> bool {
        (self.number_of_players() > 1 && self.has_too_many_tokens())
            || (has_city && self.has_population())
    }

    pub fn has_too_many_tokens(&self) -> bool {
        self.total_population() > self.max_population
    }

    pub fn total_population(&self) -> usize {
        self.player_tokens.values().map(|set| set.len()).sum()
    }

    pub fn has_population(&self) -> bool {
        self.total_population() > 0
    }

    pub fn max_expansion_for_player(&self, player: Entity) -> usize {
        if let Some(player_tokens) = self.player_tokens.get(&player) {
            match player_tokens.len() {
                0 => 0,
                1 => 1,
                _ => 2,
            }
        } else {
            0
        }
    }

    pub fn population_for_player(&self, player: Entity) -> usize {
        if let Some(player_tokens) = self.player_tokens.get(&player) {
            player_tokens.len()
        } else {
            0
        }
    }

    pub fn has_player(&self, player: &Entity) -> bool {
        self.player_tokens.contains_key(player)
    }

    pub fn has_other_players(&self, player: &Entity) -> bool {
        self.player_tokens.keys().filter(|k| *k != player).count() > 0
    }

    pub fn remove_all_but_n_tokens(
        &mut self,
        player: &Entity,
        n: usize,
    ) -> Option<HashSet<Entity>> {
        let mut tokens_to_remove: usize = 0;
        if let Some(player_tokens) = self.player_tokens.get(player) {
            tokens_to_remove = if player_tokens.len() > n {
                player_tokens.len() - n
            } else {
                0
            };
        }
        self.remove_tokens_from_area(player, tokens_to_remove)
    }

    pub fn remove_tokens_from_area(
        &mut self,
        player: &Entity,
        number_of_tokens: usize,
    ) -> Option<HashSet<Entity>> {
        if let Some(player_tokens) = self.player_tokens.get_mut(player) {
            if number_of_tokens > 0 {
                if player_tokens.len() >= number_of_tokens {
                    let tokens: HashSet<Entity> = player_tokens
                        .iter()
                        .take(number_of_tokens)
                        .copied()
                        .collect();
                    for token in tokens.iter() {
                        player_tokens.remove(token);
                    }
                    if player_tokens.is_empty() {
                        self.player_tokens.remove(player);
                    }
                    Some(tokens)
                } else {
                    let tokens = player_tokens.drain().collect();
                    self.player_tokens.remove(player);
                    Some(tokens)
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn add_token_to_area(&mut self, player: Entity, token: Entity) {
        if let Some(tokens) = self.player_tokens.get_mut(&player) {
            tokens.insert(token);
        } else {
            self.player_tokens.insert(player, HashSet::from([token]));
        }
    }
    pub fn add_tokens_to_area(&mut self, player: Entity, tokens: HashSet<Entity>) {
        if let Some(token_set) = self.player_tokens.get_mut(&player) {
            token_set.extend(tokens);
        } else {
            self.player_tokens.insert(player, tokens);
        }
    }

    pub fn remove_token_from_area(&mut self, player: Entity, token: Entity) {
        if let Some(tokens) = self.player_tokens.get_mut(&player) {
            tokens.remove(&token);
            if tokens.is_empty() {
                self.player_tokens.remove(&player);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::entity::Entity;
    use std::cell::RefCell;

    thread_local! {
        static ENTITY_COUNTER: RefCell<u32> = RefCell::new(0);
    }
    fn create_entity() -> Entity {
        ENTITY_COUNTER.with(|counter| {
            let index = *counter.borrow();
            *counter.borrow_mut() += 1; // Increment the counter for the next entity
            Entity::from_raw_u32(index).unwrap()
        })
    }

    #[test]
    fn test_new_population() {
        let max_population = 100;
        let population = Population::new(max_population);
        assert_eq!(population.max_population, max_population);
        assert!(population.player_tokens.is_empty());
    }

    #[test]
    fn test_add_token_to_area() {
        let mut population = Population::new(10);
        let player = create_entity();
        let token = create_entity();

        population.add_token_to_area(player, token);
        assert!(population.player_tokens.contains_key(&player));
        assert!(
            population
                .player_tokens
                .get(&player)
                .unwrap()
                .contains(&token)
        );
    }

    #[test]
    fn test_remove_token_from_area() {
        let mut population = Population::new(10);
        let player = create_entity();
        let token = create_entity();

        population.add_token_to_area(player, token);
        population.remove_token_from_area(player, token);

        assert!(population.tokens_for_player(&player).is_none());
        assert!(!population.has_player(&player));
    }

    #[test]
    fn test_has_more_than_one_player() {
        let mut population = Population::new(10);
        let player1 = create_entity();
        let player2 = create_entity();
        let token = create_entity();
        let token2 = create_entity();

        population.add_token_to_area(player1, token);
        assert!(!population.has_more_than_one_player());

        population.add_token_to_area(player2, token2);
        assert!(population.has_more_than_one_player());
    }

    #[test]
    fn test_has_other_players() {
        let mut population = Population::new(10);
        let player1 = create_entity();
        let player2 = create_entity();

        population.add_token_to_area(player1, create_entity());
        assert!(!population.has_other_players(&player1));

        population.add_token_to_area(player2, create_entity());
        assert!(population.has_other_players(&player1));
    }

    #[test]
    fn test_all_lengths_equal() {
        let mut population = Population::new(10);
        let player1 = create_entity();
        let player2 = create_entity();
        let token1 = create_entity();
        let token2 = create_entity();

        population.add_token_to_area(player1, token1);
        population.add_token_to_area(player2, token2);

        assert!(population.all_lengths_equal());

        let token3 = create_entity();
        population.add_token_to_area(player2, token3);

        assert!(!population.all_lengths_equal());
    }

    #[test]
    fn test_remove_surplus() {
        let mut population = Population::new(2);
        let player = create_entity();
        let token1 = create_entity();
        let token2 = create_entity();
        let token3 = create_entity();

        population.add_token_to_area(player, token1);
        population.add_token_to_area(player, token2);
        population.add_token_to_area(player, token3);

        let surplus = population.remove_surplus();
        assert_eq!(surplus.len(), 1); // Since the max_population is 2, 1 token should be removed.
        assert_eq!(population.total_population(), 2);
    }

    #[test]
    fn test_remove_all_tokens() {
        let mut population = Population::new(10);
        let player1 = create_entity();
        let player2 = create_entity();
        let token1 = create_entity();
        let token2 = create_entity();

        population.add_token_to_area(player1, token1);
        population.add_token_to_area(player2, token2);

        let all_tokens = population.remove_all_tokens();
        assert!(all_tokens.contains(&token1));
        assert!(all_tokens.contains(&token2));
        assert!(population.player_tokens.is_empty());
    }

    #[test]
    fn test_remove_all_tokens_for_player() {
        let mut population = Population::new(10);
        let player = create_entity();
        let token1 = create_entity();
        let token2 = create_entity();

        population.add_token_to_area(player, token1);
        population.add_token_to_area(player, token2);

        let tokens = population.remove_all_tokens_for_player(&player);
        assert!(tokens.contains(&token1));
        assert!(tokens.contains(&token2));
        assert!(!population.has_player(&player));
    }

    #[test]
    fn test_remove_all_but_n_tokens() {
        let mut population = Population::new(10);
        let player = create_entity();
        let token1 = create_entity();
        let token2 = create_entity();
        let token3 = create_entity();
        let token4 = create_entity();

        population.add_token_to_area(player, token1);
        population.add_token_to_area(player, token2);
        population.add_token_to_area(player, token3);
        population.add_token_to_area(player, token4);

        let removed_tokens = population.remove_all_but_n_tokens(&player, 2).unwrap();
        assert!(
            removed_tokens.contains(&token1)
                || removed_tokens.contains(&token2)
                || removed_tokens.contains(&token3)
                || removed_tokens.contains(&token4)
        );
        assert_eq!(population.population_for_player(player), 2);
    }

    #[test]
    fn test_is_conflict_zone() {
        let mut population = Population::new(2);
        let player1 = create_entity();
        let player2 = create_entity();

        population.add_token_to_area(player1, create_entity());
        population.add_token_to_area(player1, create_entity());
        population.add_token_to_area(player2, create_entity());

        assert!(population.is_conflict_zone(false));
        assert!(population.is_conflict_zone(true));
    }
}
