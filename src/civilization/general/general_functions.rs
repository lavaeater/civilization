use crate::civilization::general::general_components::{PlayerAreas, TokenStock};
use crate::civilization::general::prelude::Population;
use bevy::prelude::Entity;

pub fn move_from_stock_to_area(player: Entity, area: Entity, at_most_tokens: usize, population: &mut Population, token_stock: &mut TokenStock, player_areas: &mut PlayerAreas) {
    let tokens = token_stock.remove_at_most_n_tokens_from_stock(at_most_tokens).unwrap_or_default();
    population.add_tokens_to_area(player, tokens.clone());
    player_areas.add_tokens_to_area(area, tokens);
}

pub fn return_all_tokens_from_area_to_player(player: Entity, area: Entity, population: &mut Population, token_stock: &mut TokenStock, player_areas: &mut PlayerAreas) {
    let tokens = population.remove_all_tokens_for_player(player);
    token_stock.return_tokens_to_stock(tokens.clone());
    player_areas.remove_area(area);
}