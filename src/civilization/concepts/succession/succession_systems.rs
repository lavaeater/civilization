use crate::civilization::concepts::civ_cards::{AvailableCivCards, CivCardType, PlayerCivilizationCards};
use crate::civilization::concepts::succession::succession_components::{AstEpoch, AstPosition};
use crate::civilization::{PlayerCities};
use crate::player::Player;
use crate::GameActivity;
use bevy::prelude::{info, NextState, Query, Res, ResMut, With};
use enumflags2::BitFlags;

/// Advances (or retreats) each player's AST marker at the end of a round (rule 33.1–33.4).
///
/// Called on `OnEnter(GameActivity::MoveSuccessionMarkers)`.
pub fn advance_succession_markers(
    mut player_query: Query<
        (
            &mut AstPosition,
            Option<&PlayerCivilizationCards>,
            &PlayerCities,
        ),
        With<Player>,
    >,
    civ_card_defs: Option<Res<AvailableCivCards>>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    for (mut position, civ_cards, player_cities) in player_query.iter_mut() {
        let city_count = player_cities.number_of_cities();

        if city_count == 0 {
            // No cities → retreat one space, but never below space 1 (rule 33.4)
            if position.space > 1 {
                position.space -= 1;
                info!("[AST] Player retreated to space {} (no cities)", position.space);
            }
            continue;
        }

        // Attempt to advance one space
        let target_space = position.space + 1;
        let target_epoch = AstEpoch::for_space(target_space);

        let can_advance = city_count >= target_epoch.min_cities()
            && check_card_requirements(
                &target_epoch,
                civ_cards.as_deref(),
                civ_card_defs.as_deref(),
            );

        if can_advance {
            position.space = target_space;
            info!("[AST] Player advanced to space {}", position.space);
        } else {
            info!(
                "[AST] Player frozen at space {} (needs {} cities, has {})",
                position.space,
                target_epoch.min_cities(),
                city_count
            );
        }
    }

    next_state.set(GameActivity::CollectTaxes);
}

/// Returns true if the player's civ cards meet the epoch's group and count requirements.
fn check_card_requirements(
    epoch: &AstEpoch,
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
