use crate::GameActivity;
use crate::civilization::PlayerCities;
use crate::civilization::components::{Faction, Treasury};
use crate::civilization::concepts::acquire_trade_cards::PlayerTradeCards;
use crate::civilization::concepts::census::GameInfoAndStuff;
use crate::civilization::concepts::civ_cards::{
    AvailableCivCards, CivCardType, PlayerCivilizationCards,
};
use crate::civilization::concepts::round_summary::RoundSummary;
use crate::civilization::concepts::succession::succession_components::{
    AstEpoch, AstPosition, AstTrack, GameResult, RoundLimit,
};
use crate::player::Player;
use bevy::prelude::{Commands, Name, NextState, Query, Res, ResMut, With, info};
use enumflags2::BitFlags;

/// Advances (or retreats) each player's AST marker at the end of a round (rule 33.1–33.4).
///
/// Called on `OnEnter(GameActivity::MoveSuccessionMarkers)`.
///
/// Per rule:
/// - A marker normally moves one space right (33.1).
/// - It may not enter a new epoch unless that epoch's requirements are met — it is
///   then *frozen* (33.11, 33.3). A player still advances freely within the Stone
///   Age, which has no requirements.
/// - A player with no cities retreats one space per turn (33.4), **unless** the
///   marker is in the Stone Age, where it simply advances/freezes normally.
/// - A marker never advances past FINISH.
pub fn advance_succession_markers(
    mut player_query: Query<
        (
            &mut AstPosition,
            Option<&PlayerCivilizationCards>,
            &PlayerCities,
            &Faction,
        ),
        With<Player>,
    >,
    track: Res<AstTrack>,
    civ_card_defs: Option<Res<AvailableCivCards>>,
    game_info: Res<GameInfoAndStuff>,
    round_limit: Option<Res<RoundLimit>>,
    mut next_state: ResMut<NextState<GameActivity>>,
    mut round_summary: ResMut<RoundSummary>,
) {
    for (mut position, civ_cards, player_cities, faction) in &mut player_query {
        let city_count = player_cities.number_of_cities();
        let finish = track.finish_index(faction.faction);

        if position.space >= finish {
            // Already at FINISH — nothing to do (win handled elsewhere).
            continue;
        }

        let current_epoch = AstEpoch::for_space(position.space);
        let target_space = position.space + 1;
        let target_epoch = AstEpoch::for_space(target_space);

        let can_advance = city_count >= target_epoch.min_cities()
            && check_card_requirements(target_epoch, civ_cards, civ_card_defs.as_deref());

        if can_advance {
            position.space = target_space;
            info!(
                "[AST] {:?} advanced to space {}",
                faction.faction, position.space
            );
            if position.space >= finish {
                // Rule 34.1A — reaching a finish square ends the game (final scoring
                // by rule 35 is a separate feature; this is the trigger condition).
                info!(
                    "[AST] {:?} reached FINISH ({}) — game-end condition met (rule 34.1A)",
                    faction.faction, finish
                );
                round_summary.push(format!("{:?} reached FINISH!", faction.faction));
            } else {
                round_summary.push(format!(
                    "{:?} advanced on the A.S.T. to space {}",
                    faction.faction, position.space
                ));
            }
        } else if city_count == 0 && current_epoch != AstEpoch::StoneAge {
            // No cities outside the Stone Age → retreat one space (rule 33.4).
            position.space = position.space.saturating_sub(1);
            info!(
                "[AST] {:?} retreated to space {} (no cities)",
                faction.faction, position.space
            );
            round_summary.push(format!(
                "{:?} has no cities and retreated on the A.S.T. to space {}",
                faction.faction, position.space
            ));
        } else {
            info!(
                "[AST] {:?} frozen at space {} (entering {} needs {} cities, has {})",
                faction.faction,
                position.space,
                target_epoch.name(),
                target_epoch.min_cities(),
                city_count
            );
            round_summary.push(format!(
                "{:?} could not advance on the A.S.T. — entering {} needs {} cities, has {}",
                faction.faction,
                target_epoch.name(),
                target_epoch.min_cities(),
                city_count
            ));
        }
    }

    // Rule 34.1A: if any marker is on its finish square, the game ends now (the
    // move onto finish is the last step). Rule 34.1B: alternatively, a
    // predetermined round limit (set before the game starts) also ends it —
    // this check runs at the end of a round, so 34.2's "complete the final
    // turn" requirement is satisfied for free by the phase ordering.
    let ast_finished = player_query
        .iter()
        .any(|(pos, _, _, faction)| pos.space >= track.finish_index(faction.faction));
    let time_limit_reached = round_limit
        .and_then(|limit| limit.0)
        .is_some_and(|limit| game_info.round >= limit);

    if ast_finished || time_limit_reached {
        if time_limit_reached && !ast_finished {
            info!(
                "[AST] Round {} reached the configured time limit — game-end condition met (rule 34.1B)",
                game_info.round
            );
        }
        next_state.set(GameActivity::GameOver);
    } else {
        next_state.set(GameActivity::CollectTaxes);
    }
}

/// Rule 35: final scoring once the game has ended. Adds, for each player, the face
/// value of civilization cards (A), commodity-card set value (B), treasury tokens
/// (C, 1 pt each), A.S.T. position (D, 100 pts/space from the start arrow at 0) and
/// cities (E, 50 pts each). Highest total wins; ties broken by furthest A.S.T.
/// position (35.2). Runs once on entering `GameOver`.
pub fn determine_winner(
    players: Query<
        (
            &Name,
            &AstPosition,
            &PlayerCities,
            Option<&PlayerCivilizationCards>,
            Option<&PlayerTradeCards>,
            Option<&Treasury>,
        ),
        With<Player>,
    >,
    civ_card_defs: Option<Res<AvailableCivCards>>,
    mut commands: Commands,
) {
    info!("[GAME OVER] Final scoring (rule 35.1):");
    // (name, total, ast_space) — ast_space is the tiebreaker.
    let mut scored: Vec<(String, u32, u32)> = Vec::new();
    for (name, position, cities, civ_cards, trade_cards, treasury) in players.iter() {
        let civ_value: u32 = match (civ_cards, civ_card_defs.as_deref()) {
            (Some(cc), Some(defs)) => defs.cards_for_names(&cc.cards).iter().map(|d| d.cost).sum(),
            _ => 0,
        };
        let commodity_value = trade_cards.map_or(0, |t| t.total_stack_value() as u32);
        let treasury_value = treasury.map_or(0, |t| t.tokens_in_treasury() as u32);
        let ast_value = position.space * 100;
        let city_value = cities.number_of_cities() as u32 * 50;
        let total = civ_value + commodity_value + treasury_value + ast_value + city_value;
        info!(
            "[GAME OVER] {}: {} pts  (civ {} + commodities {} + treasury {} + AST {} + cities {})",
            name, total, civ_value, commodity_value, treasury_value, ast_value, city_value
        );
        scored.push((name.to_string(), total, position.space));
    }
    // Highest total wins; tiebreak by furthest along the A.S.T. (rule 35.2).
    scored.sort_by(|a, b| b.1.cmp(&a.1).then(b.2.cmp(&a.2)));
    if let Some((winner, score, space)) = scored.first() {
        info!(
            "[GAME OVER] 🏆 WINNER: {} — {} points (A.S.T. space {})",
            winner, score, space
        );
    }
    // Hand the standings to the victory screen.
    commands.insert_resource(GameResult { standings: scored });
}

/// Returns true if the player's civ cards meet the epoch's group and count requirements.
fn check_card_requirements(
    epoch: AstEpoch,
    civ_cards: Option<&PlayerCivilizationCards>,
    civ_card_defs: Option<&AvailableCivCards>,
) -> bool {
    let min_groups = epoch.min_card_groups();
    let min_count = epoch.min_card_count();

    if min_groups == 0 && min_count == 0 {
        return true;
    }

    let (Some(cards), Some(defs)) = (civ_cards, civ_card_defs) else {
        return false;
    };

    if cards.cards.len() < min_count {
        return false;
    }

    if min_groups > 0 {
        let groups = count_distinct_card_groups(cards, defs);
        if groups < min_groups {
            return false;
        }
    }

    true
}

/// Counts distinct CivCardType groups represented in the player's cards.
/// Dual-type cards count toward all groups they belong to (rule 31.551).
fn count_distinct_card_groups(
    civ_cards: &PlayerCivilizationCards,
    defs: &AvailableCivCards,
) -> usize {
    let mut groups: BitFlags<CivCardType> = BitFlags::empty();
    for def in defs.cards_for_names(&civ_cards.cards) {
        groups |= def.card_type;
    }
    groups.iter().count()
}
