use crate::civilization::*;
use crate::player::Player;
use crate::stupid_ai::*;
use bevy::platform::collections::HashMap;
use bevy::prelude::{
    Commands, Entity, Has, MessageReader, MessageWriter, Name, Query, Res, ResMut, With, debug,
    error, warn,
};
use rand::prelude::SliceRandom;

pub fn setup_stupid_ai(
    mut stupid_ai_event: MessageReader<StupidAiMessage>,
    mut commands: Commands,
) {
    for e in stupid_ai_event.read() {
        commands.entity(e.player).insert(StupidAi);
    }
}

pub fn assign_some_foolish_cards() {}

/// Clear the per-player movement selection counts when a new movement phase begins.
pub fn reset_movement_loop_guard(mut loop_guard: ResMut<MovementLoopGuard>) {
    loop_guard.counts.clear();
}

pub fn select_stupid_pop_exp(
    mut event_reader: MessageReader<SelectStupidMove>,
    player_moves: Query<(&Name, &AvailableMoves, &Personality)>,
    area_info_query: Query<(
        Entity,
        &Population,
        &LandPassage,
        Option<&BuiltCity>,
        Has<CitySite>,
    )>,
    mut expand_writer: MessageWriter<ExpandPopulationManuallyCommand>,
    debug_options: Res<DebugOptions>,
) {
    for event in event_reader.read() {
        if let Ok((player_name, available_moves, personality)) = player_moves.get(event.player) {
            let areas = gather_area_summaries(event.player, &area_info_query);
            let scored: Vec<(usize, f32)> = available_moves
                .moves
                .iter()
                .map(|(i, m)| {
                    (
                        *i,
                        score_population_expansion(m, &areas, &personality.weights),
                    )
                })
                .collect();
            let mut rng = rand::rng();
            if let Some(chosen) = pick(&scored, personality.picker, &mut rng) {
                let selected_move = &available_moves.moves[&chosen];
                if debug_options.print_selected_moves {
                    debug!("{} selects {:#?}", player_name, selected_move);
                }
                match selected_move {
                    GameMove::PopulationExpansion(pop_exp_move) => {
                        expand_writer.write(ExpandPopulationManuallyCommand::new(
                            event.player,
                            pop_exp_move.area,
                            pop_exp_move.max_tokens,
                        ));
                    }
                    _ => {
                        debug!("In pop exp, move was: {:#?}", selected_move);
                    }
                }
            }
        }
    }
}

pub fn select_stupid_movement(
    mut event_reader: MessageReader<SelectStupidMove>,
    player_moves: Query<(
        &Name,
        &AvailableMoves,
        &Personality,
        &PlayerCities,
        &AstPosition,
        &CityTokenStock,
        Option<&PlayerCivilizationCards>,
    )>,
    mut move_tokens_writer: MessageWriter<MoveTokenFromAreaToAreaCommand>,
    mut ship_ferry_writer: MessageWriter<ShipFerryCommand>,
    mut end_movement_writer: MessageWriter<PlayerMovementEnded>,
    area_info_query: Query<(
        Entity,
        &Population,
        &LandPassage,
        Option<&BuiltCity>,
        Has<CitySite>,
    )>,
    debug_options: Res<DebugOptions>,
    mut loop_guard: ResMut<MovementLoopGuard>,
    standings_query: Query<StandingsData, With<Player>>,
    civ_card_defs: Option<Res<AvailableCivCards>>,
) {
    // Everyone's rule-35 score estimate, once per frame (only computed when
    // somebody is actually deciding a move this frame).
    let mut score_table: Option<HashMap<Entity, u32>> = None;
    for event in event_reader.read() {
        if let Ok((
            player_name,
            available_moves,
            personality,
            player_cities,
            ast_position,
            city_token_stock,
            civ_cards,
        )) = player_moves.get(event.player)
        {
            let mut areas = gather_area_summaries(event.player, &area_info_query);
            // Rule 24.31: Engineering makes a city need 8 attackers, not 7.
            for summary in areas.values_mut() {
                if let Some(owner) = summary.city_owner
                    && standings_query
                        .get(owner)
                        .is_ok_and(|(_, _, _, c, ..)| c.is_some_and(|c| c.owns(&CivCardName::Engineering)))
                {
                    summary.city_defence = CITY_ATTACK_TOKENS + 1;
                }
            }

            // Movement is the only way a stack reaches the city threshold (no
            // area's printed max is >= 6), so plan which sites to pile into.
            let has_architecture = civ_cards.is_some_and(|c| c.owns(&CivCardName::Architecture));
            let threshold = if has_architecture { 5 } else { 6 };
            let drive = if city_token_stock.has_tokens() {
                personality
                    .weights
                    .city_income
                    .max(city_urgency(player_cities, ast_position))
                    .max(personality.strategy.city_floor())
            } else {
                0.0
            };
            let plan = CityPlan::build(
                &available_moves.moves,
                &areas,
                threshold,
                drive,
                Some(player_cities.number_of_cities()),
            );

            // Conqueror: how far ahead each rival is, to aim attacks at the leader.
            let conquest = personality.strategy.conquest();
            let sieges = SiegePlan::build(&available_moves.moves, &areas);
            let mut rivals = if conquest > 0.0 {
                let scores = score_table
                    .get_or_insert_with(|| standings(&standings_query, civ_card_defs.as_deref()));
                let me = scores.get(&event.player).copied().unwrap_or(0);
                Rivals {
                    conquest,
                    standing: scores
                        .iter()
                        .filter(|(p, _)| **p != event.player)
                        .map(|(p, s)| (*p, rival_standing(me, *s)))
                        .collect(),
                    ..Default::default()
                }
            } else {
                Rivals::default()
            };
            rivals.sieges = sieges;

            let scored: Vec<(usize, f32)> = available_moves
                .moves
                .iter()
                .map(|(i, m)| {
                    (
                        *i,
                        score_movement(
                            m,
                            event.player,
                            &areas,
                            &personality.weights,
                            &plan,
                            &rivals,
                        ),
                    )
                })
                .collect();

            // Ending movement ends the whole turn's movement, not just this
            // pick. A soft picker draws again for every token stack moved, so
            // leaving `EndMovement` in the draw almost guaranteed an early
            // stop: growth went unspread and was lost to surplus removal
            // (~30 tokens a round in 9-player self-play). It's only a
            // candidate when no real move beats it.
            let candidates = without_premature_end(&scored, &available_moves.moves);

            let mut rng = rand::rng();
            let Some(chosen) = pick(&candidates, personality.picker, &mut rng) else {
                continue;
            };
            let selected_move = &available_moves.moves[&chosen];
            // Opt-in trace (`RUST_LOG=info,ai_move=debug`): why did the AI pick this?
            if bevy::log::tracing::enabled!(target: "ai_move", bevy::log::Level::DEBUG) {
                let mut ranked = scored.clone();
                ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
                let describe = |i: usize| match &available_moves.moves[&i] {
                    GameMove::Movement(m) => {
                        format!("Move {:?}->{:?} x{}", m.source, m.target, m.max_tokens)
                    }
                    GameMove::ShipFerry(m) => {
                        format!("Ferry {:?}->{:?} x{}", m.source, m.target, m.max_tokens)
                    }
                    GameMove::AttackArea(m) => format!(
                        "AttackArea {:?}->{:?} x{}",
                        m.source, m.target, m.max_tokens
                    ),
                    GameMove::AttackCity(m) => format!(
                        "AttackCity {:?}->{:?} x{}",
                        m.source, m.target, m.max_tokens
                    ),
                    other => format!("{other:?}"),
                };
                let top: Vec<String> = ranked
                    .iter()
                    .take(3)
                    .map(|(i, s)| format!("{} = {s:.2}", describe(*i)))
                    .collect();
                if matches!(selected_move, GameMove::EndMovement) {
                    // Overfull areas left behind lose tokens to surplus removal:
                    // what could still have moved out of them?
                    for (area, a) in &areas {
                        if a.my_pop <= a.max_population || plan.is_target(*area) {
                            continue;
                        }
                        let outs: Vec<String> = scored
                            .iter()
                            .filter_map(|(i, sc)| match &available_moves.moves[i] {
                                GameMove::Movement(m) | GameMove::ShipFerry(m)
                                | GameMove::AttackArea(m)
                                    if m.source == *area =>
                                {
                                    let room = areas.get(&m.target).map_or(-1, |t| {
                                        t.max_population as i64 - (t.my_pop + t.enemy_pop) as i64
                                    });
                                    Some(format!("->{:?} x{} room {room} = {sc:.2}", m.target, m.max_tokens))
                                }
                                _ => None,
                            })
                            .collect();
                        debug!(
                            target: "ai_move",
                            "{} OVERFULL {:?} {}/{} neighbours {} outs [{}]",
                            player_name,
                            area,
                            a.my_pop,
                            a.max_population,
                            a.neighbours.len(),
                            outs.join("; ")
                        );
                    }
                }
                debug!(
                    target: "ai_move",
                    "{} picks {} | plan drive={:.2} {:?} | {} moves, top: {}",
                    player_name,
                    describe(chosen),
                    plan.drive,
                    plan,
                    scored.len(),
                    top.join("; ")
                );
            }

            // Loop guard: count consecutive selections this player has made without
            // ending movement. A no-op move (one the executor silently drops) would
            // otherwise spin forever under a deterministic picker.
            let count = loop_guard.counts.entry(event.player).or_insert(0);
            *count += 1;
            let count = *count;
            if count == MovementLoopGuard::WARN_AT {
                warn!(
                    "{} has made {} consecutive movement selections without ending. \
                     Looping move (score {:.3}): {:#?}",
                    player_name,
                    count,
                    scored
                        .iter()
                        .find(|(i, _)| *i == chosen)
                        .map_or(f32::NAN, |(_, s)| *s),
                    selected_move,
                );
            }
            if count >= MovementLoopGuard::FORCE_END_AT
                && !matches!(selected_move, GameMove::EndMovement)
            {
                error!(
                    "{} exceeded {} movement selections — forcing end of movement. \
                     Last move would have been: {:#?}",
                    player_name,
                    MovementLoopGuard::FORCE_END_AT,
                    selected_move,
                );
                loop_guard.counts.remove(&event.player);
                end_movement_writer.write(PlayerMovementEnded::new(event.player));
                continue;
            }

            {
                if debug_options.print_selected_moves {
                    debug!("{} selects {:#?}", player_name, selected_move);
                }
                match selected_move {
                    GameMove::Movement(movement_move) => {
                        if let Some(n) =
                            plan.tokens_to_send(movement_move.target, movement_move.max_tokens)
                        {
                            // Feeding a planned city site: send exactly what's missing
                            // (capped at a complete source's spare tokens).
                            let n = plan
                                .spare_at(movement_move.source)
                                .map_or(n, |spare| n.min(spare.max(1)));
                            move_tokens_writer.write(MoveTokenFromAreaToAreaCommand::new(
                                movement_move.source,
                                movement_move.target,
                                n,
                                event.player,
                            ));
                        } else if let Some(spare) = plan.spare_at(movement_move.source) {
                            // Leaving a buildable site: never drop it below the threshold.
                            move_tokens_writer.write(MoveTokenFromAreaToAreaCommand::new(
                                movement_move.source,
                                movement_move.target,
                                spare.clamp(1, movement_move.max_tokens),
                                event.player,
                            ));
                        } else {
                            // Send 1-2 tokens, but no more than the target can
                            // hold: the rest would only be removed as surplus.
                            let wanted = if movement_move.max_tokens <= 2 { 1 } else { 2 };
                            move_tokens_writer.write(MoveTokenFromAreaToAreaCommand::new(
                                movement_move.source,
                                movement_move.target,
                                fit_to_room(wanted, movement_move.target, &areas),
                                event.player,
                            ));
                        }
                    }
                    GameMove::ShipFerry(ferry_move) => {
                        // Ferry up to max_tokens (≤5) tokens by ship. Keep 1 back if
                        // possible -- unless feeding a planned city site, then send
                        // exactly what's missing.
                        let n = plan
                            .tokens_to_send(ferry_move.target, ferry_move.max_tokens)
                            .unwrap_or_else(|| {
                                let wanted = ferry_move.max_tokens.saturating_sub(1).max(1);
                                fit_to_room(wanted, ferry_move.target, &areas)
                            });
                        ship_ferry_writer.write(ShipFerryCommand::new(
                            ferry_move.source,
                            ferry_move.target,
                            n,
                            event.player,
                        ));
                    }
                    GameMove::EndMovement => {
                        loop_guard.counts.remove(&event.player);
                        end_movement_writer.write(PlayerMovementEnded::new(event.player));
                    }
                    GameMove::AttackCity(movement_move)
                        if !rivals.sieges.launch(movement_move.target).is_empty() =>
                    {
                        // Planned siege: launch every allocated stack this pick, so
                        // the attack lands at full strength (rule 24.31) instead of
                        // trickling in and dying below 7.
                        for (source, n) in rivals.sieges.launch(movement_move.target) {
                            move_tokens_writer.write(MoveTokenFromAreaToAreaCommand::new(
                                *source,
                                movement_move.target,
                                *n,
                                event.player,
                            ));
                        }
                    }
                    GameMove::AttackArea(movement_move) | GameMove::AttackCity(movement_move) => {
                        //A little complexity here: If possible, leave two, but also, always make a move
                        send_movement_move(&mut move_tokens_writer, event, movement_move, true);
                    }
                    _ => {
                        debug!("In Movement, move was: {:#?}", selected_move);
                    }
                }
            }
        }
    }
}

pub fn select_stupid_city_building(
    mut event_reader: MessageReader<SelectStupidMove>,
    player_moves: Query<(
        &Name,
        &AvailableMoves,
        &Personality,
        &PlayerCities,
        &AstPosition,
    )>,
    area_info_query: Query<(
        Entity,
        &Population,
        &LandPassage,
        Option<&BuiltCity>,
        Has<CitySite>,
    )>,
    mut build_city_writer: MessageWriter<BuildCityCommand>,
    mut end_player_city_construction: MessageWriter<EndPlayerCityConstruction>,
    debug_options: Res<DebugOptions>,
) {
    for event in event_reader.read() {
        if let Ok((player_name, available_moves, personality, player_cities, ast_position)) =
            player_moves.get(event.player)
        {
            let areas = gather_area_summaries(event.player, &area_info_query);
            // Short of the cities needed to clear the *next* A.S.T. epoch gate?
            // Force city-building to the top of every playstyle's priorities.
            let urgency =
                city_urgency(player_cities, ast_position).max(personality.strategy.city_floor());
            let scored: Vec<(usize, f32)> = available_moves
                .moves
                .iter()
                .map(|(i, m)| {
                    (
                        *i,
                        score_city_construction(
                            m,
                            &areas,
                            &personality.weights,
                            urgency,
                            player_cities.number_of_cities(),
                        ),
                    )
                })
                .collect();
            let mut rng = rand::rng();

            if let Some(chosen) = pick(&scored, personality.picker, &mut rng) {
                let selected_move = &available_moves.moves[&chosen];
                if debug_options.print_selected_moves {
                    debug!("{} selects {:#?}", player_name, selected_move);
                }
                match selected_move {
                    GameMove::CityConstruction(build_city_move) => {
                        build_city_writer
                            .write(BuildCityCommand::new(event.player, build_city_move.target));
                    }
                    GameMove::EndCityConstruction => {
                        end_player_city_construction
                            .write(EndPlayerCityConstruction::new(event.player));
                    }
                    _ => {
                        debug!("In City Construction, move was: {:#?}", selected_move);
                    }
                }
            }
        }
    }
}

pub fn select_stupid_city_elimination(
    mut event_reader: MessageReader<SelectStupidMove>,
    player_moves: Query<(&Name, &AvailableMoves, &Personality)>,
    area_info_query: Query<(
        Entity,
        &Population,
        &LandPassage,
        Option<&BuiltCity>,
        Has<CitySite>,
    )>,
    mut eliminate_city: MessageWriter<EliminateCity>,
    debug_options: Res<DebugOptions>,
) {
    for event in event_reader.read() {
        if let Ok((player_name, available_moves, personality)) = player_moves.get(event.player) {
            let areas = gather_area_summaries(event.player, &area_info_query);
            let scored: Vec<(usize, f32)> = available_moves
                .moves
                .iter()
                .map(|(i, m)| (*i, score_city_elimination(m, &areas, &personality.weights)))
                .collect();
            let mut rng = rand::rng();

            if let Some(chosen) = pick(&scored, personality.picker, &mut rng) {
                let selected_move = &available_moves.moves[&chosen];
                if debug_options.print_selected_moves {
                    debug!("{} selects {:#?}", player_name, selected_move);
                }
                match selected_move {
                    GameMove::EliminateCity(el_move) => {
                        eliminate_city.write(EliminateCity::new(
                            el_move.player,
                            el_move.city,
                            el_move.area,
                            false,
                        ));
                    }
                    _ => {
                        debug!("In City Elimination, move was: {:#?}", selected_move);
                    }
                }
            }
        }
    }
}

pub fn select_stupid_trade_move(
    mut event_reader: MessageReader<SelectStupidMove>,
    player_moves: Query<(&Name, &AvailableMoves, &PlayerAreas)>,
    _commands: Commands,
    _trade_offer_query: Query<&mut TradeOffer>,
    player_trade_cards: Query<(&PlayerTradeCards, &PlayerTradeInterests)>,
    player_wants_query: Query<(&PlayerTradeInterests, &Name)>,
) {
    for event in event_reader.read() {
        // //debug!("Selecting stupid AI move for player {:#?}", event.player);
        if let Ok((player_name, available_moves, _player_areas)) = player_moves.get(event.player) {
            let trade_moves = available_moves
                .moves
                .values()
                .filter_map(|m| match m {
                    GameMove::Trade(trade_move) => Some(trade_move),
                    _ => None,
                })
                .cloned()
                .collect::<Vec<_>>();
            for trade_move in &trade_moves {
                let mut rng = rand::rng();
                match trade_move {
                    TradeMove::ProposeTrade(receiver, matching_cards) => {
                        if let Ok((player_trade_cards, player_wants)) =
                            player_trade_cards.get(event.player)
                            && let Ok((_receiver_wants, receiver_name)) =
                                player_wants_query.get(*receiver)
                        {
                            match matching_cards.len() {
                                1 => {
                                    /*
                                    This means we have ONE card in common with the other player.
                                     */
                                    let (card, no_of_cards) = matching_cards.iter().next().unwrap();
                                    // if this card is NOT our top commodity, we can offer it as a trade.
                                    if !player_trade_cards.is_top_commodity(*card) {
                                        let mut offer = TradeOffer::propose_trade(
                                            event.player,
                                            player_name,
                                            *receiver,
                                            receiver_name,
                                        );
                                        if let Some(lowest_commodity) =
                                            player_trade_cards.worst_commodity()
                                        {
                                            match no_of_cards {
                                                0 => {
                                                    //Shouldn't happen, but could happen. We should not trade for a card we don't have.
                                                }
                                                1 => {
                                                    offer.pay_even_more(*card, 1);
                                                    offer.pay_even_more(lowest_commodity, 2);
                                                }
                                                _ => {
                                                    offer.pay_even_more(*card, 2);
                                                    offer.pay_even_more(lowest_commodity, 1);
                                                }
                                            }
                                            //Now add what we want
                                            for (card, count) in
                                                player_wants.get_trade_thingie(&mut rng)
                                            {
                                                offer.get_even_more(card, count);
                                            }
                                        }
                                    }
                                }

                                2 => {
                                    /*
                                    This means we have TWO cards in common with the other player.
                                    We exclude any match that is our top commodity, then
                                    we add any remainders in order of number of cards.
                                     */
                                    let mut offer = TradeOffer::propose_trade(
                                        event.player,
                                        player_name,
                                        *receiver,
                                        receiver_name,
                                    );
                                    for (card, count) in player_wants.get_trade_thingie(&mut rng) {
                                        offer.get_even_more(card, count);
                                    }
                                }
                                3 => {
                                    // Suggest two random cards from these three and then the lowest commodity
                                    let mut cards: Vec<TradeCard> =
                                        matching_cards.keys().copied().collect();
                                    // Shuffle to get random selection
                                    cards.shuffle(&mut rng);
                                }
                                _ => {}
                            }
                        }
                    }
                    TradeMove::AcceptOrDeclineTrade(_trade_offer) => {}
                    TradeMove::AutoDeclineTrade(_trade_offer) => {}
                    TradeMove::StopTrading => {}
                    TradeMove::SettleTrade(_trade_offer) => {}
                }
            }
        }
    }
}

fn _debug_trade_move_info(trade_move: &TradeMove, trade_offer: &TradeOffer) {
    debug!(
        "Move: {:#?} on offer from {:#?} to {:#?}, gets: {:#?}, pays {:#?}",
        trade_move,
        trade_offer.initiator_name,
        trade_offer.receiver_name,
        trade_offer.initiator_gets,
        trade_offer.initiator_pays,
    );
}

pub fn select_stupid_civ_card_move(
    mut event_reader: MessageReader<SelectStupidMove>,
    player_moves: Query<(
        &Name,
        &AvailableMoves,
        &Personality,
        &AstPosition,
        &PlayerCities,
    )>,
    player_cards_query: Query<(
        &PlayerTradeCards,
        &PlayerCivilizationCards,
        Option<&crate::civilization::resolve_calamities::resolve_calamities_components::GrainLockedForPurchase>,
        Option<&crate::civilization::CardsHeldBeforePurchasing>,
        Option<&Treasury>,
    )>,
    cards: Res<AvailableCivCards>,
    mut done_writer: MessageWriter<PlayerDoneAcquiringCivilizationCards>,
    mut purchase_writer: MessageWriter<ConfirmCivCardPurchase>,
    debug_options: Res<DebugOptions>,
) {
    for event in event_reader.read() {
        if let Ok((player_name, available_moves, personality, ast_position, player_cities)) =
            player_moves.get(event.player)
        {
            // What the next A.S.T. gate still needs (cities, colour groups, card
            // count), for the strategy's card bonus.
            let needs = match player_cards_query.get(event.player) {
                Ok((_, civ_cards, ..)) => {
                    let owned = cards.cards_for_names(&civ_cards.cards);
                    let groups = owned
                        .iter()
                        .fold(enumflags2::BitFlags::empty(), |g, d| g | d.card_type);
                    AstNeeds::compute(
                        ast_position.space,
                        player_cities.number_of_cities(),
                        groups,
                        civ_cards.cards.len(),
                    )
                }
                Err(_) => AstNeeds::default(),
            };
            // Player wealth + existing credits, used to score each card option.
            // Rule 31.53: credits are computed against what was held BEFORE
            // this turn's acquiring phase began, not the live hand -- see
            // CardsHeldBeforePurchasing's doc comment. This matters here
            // specifically because this system re-runs iteratively as the AI
            // buys one card per move, so without the snapshot a card bought
            // earlier this turn would wrongly discount a later one.
            let (wealth, credits) = match player_cards_query.get(event.player) {
                Ok((trade_cards, civ_cards, _, cards_held_before, treasury)) => (
                    trade_cards.total_stack_value() as u32
                        + treasury.map_or(0, |t| t.tokens_in_treasury() as u32),
                    cards.total_credits(cards_held_before.map_or(&civ_cards.cards, |c| &c.0)),
                ),
                Err(_) => (0, Vec::new()),
            };

            let scored: Vec<(usize, f32)> = available_moves
                .moves
                .iter()
                .filter_map(|(i, m)| match m {
                    GameMove::AcquireCivilizationCards(civ_move) => {
                        let mut bonus = 0.0;
                        let option = match civ_move {
                            AcquireCivilizationCardsMove::AcquireCard(name) => {
                                cards.cards.iter().find(|c| c.name == *name).map(|def| {
                                    bonus = strategy_card_bonus(
                                        personality.strategy,
                                        def.name,
                                        def.card_type,
                                        def.cost,
                                        &needs,
                                    ) + crate::stupid_ai::scoring::calamity_defence_bonus(
                                        def.name,
                                        player_cities.number_of_cities(),
                                        personality.weights.risk,
                                    );
                                    CivCardOption {
                                        effective_cost: def.calculate_cost(&credits),
                                        credit_value: civ_card_credit_value(def),
                                        wealth,
                                    }
                                })
                            }
                            _ => None,
                        };
                        // Non-finite (unaffordable) stays non-finite.
                        Some((
                            *i,
                            score_civ_card(civ_move, option, &personality.weights) + bonus,
                        ))
                    }
                    _ => None,
                })
                .collect();

            if scored.is_empty() {
                // No civ-card move in this player's move set (e.g. a stale
                // set left over from another phase). Bailing out with `return`
                // used to drop every remaining SelectStupidMove in the batch,
                // stranding the other AI players -- and this player would
                // never be asked again either, hanging the phase. End this
                // player's acquisition and carry on with the rest.
                debug!("{player_name} has no civ card moves; ending acquisition");
                done_writer.write(PlayerDoneAcquiringCivilizationCards(event.player));
                continue;
            }

            let mut rng = rand::rng();
            if let Some(chosen) = pick(&scored, personality.picker, &mut rng) {
                let GameMove::AcquireCivilizationCards(selected_move) =
                    &available_moves.moves[&chosen]
                else {
                    continue;
                };
                // Opt-in trace (`RUST_LOG=info,ai_cards=debug`): why so few purchases?
                if bevy::log::tracing::enabled!(target: "ai_cards", bevy::log::Level::DEBUG) {
                    let offered: Vec<u32> = available_moves
                        .moves
                        .values()
                        .filter_map(|m| match m {
                            GameMove::AcquireCivilizationCards(
                                AcquireCivilizationCardsMove::AcquireCard(n),
                            ) => cards
                                .cards
                                .iter()
                                .find(|c| c.name == *n)
                                .map(|d| d.calculate_cost(&credits)),
                            _ => None,
                        })
                        .collect();
                    let hand = player_cards_query
                        .get(event.player)
                        .map_or(0, |(t, ..)| t.number_of_trade_cards());
                    debug!(
                        target: "ai_cards",
                        "{} [{:?}] wealth={} hand={} offered={} cheapest={:?} affordable={} picks {:?}",
                        player_name,
                        personality.strategy,
                        wealth,
                        hand,
                        offered.len(),
                        offered.iter().min(),
                        offered.iter().filter(|c| **c <= wealth).count(),
                        selected_move,
                    );
                }
                if debug_options.print_selected_moves {
                    debug!("{} selects {:#?}", player_name, selected_move);
                }
                match selected_move {
                    AcquireCivilizationCardsMove::DoneAcquiringCards => {
                        done_writer.write(PlayerDoneAcquiringCivilizationCards(event.player));
                    }
                    AcquireCivilizationCardsMove::AcquireCard(card_name) => {
                        if let Ok((
                            trade_cards,
                            civ_cards,
                            grain_locked,
                            cards_held_before,
                            treasury,
                        )) = player_cards_query.get(event.player)
                        {
                            let credits = cards.total_credits(
                                cards_held_before.map_or(&civ_cards.cards, |c| &c.0),
                            );
                            let card_def = cards.cards.iter().find(|c| c.name == *card_name);
                            if let Some(def) = card_def {
                                let grain = grain_locked.map_or(0, |l| l.0);
                                let treasury_n = treasury.map_or(0, Treasury::tokens_in_treasury);
                                // Rules 31.1/31.58: cards bought together are
                                // paid together, so a set too big for one card
                                // can pay for two instead of being broken up.
                                // Add every other card this player would rather
                                // buy than stop, while the hand still covers
                                // the total.
                                let done_score = scored
                                    .iter()
                                    .find(|(i, _)| {
                                        matches!(
                                            available_moves.moves.get(i),
                                            Some(GameMove::AcquireCivilizationCards(
                                                AcquireCivilizationCardsMove::DoneAcquiringCards
                                            ))
                                        )
                                    })
                                    .map_or(f32::NEG_INFINITY, |(_, s)| *s);
                                let mut others: Vec<(CivCardName, f32)> = scored
                                    .iter()
                                    .filter_map(|(i, score)| match available_moves.moves.get(i) {
                                        Some(GameMove::AcquireCivilizationCards(
                                            AcquireCivilizationCardsMove::AcquireCard(n),
                                        )) if n != card_name && *score > done_score => {
                                            Some((*n, *score))
                                        }
                                        _ => None,
                                    })
                                    .collect();
                                others.sort_by(|a, b| b.1.total_cmp(&a.1));
                                let mut to_buy = vec![*card_name];
                                let mut total = def.calculate_cost(&credits) as usize;
                                for (name, _) in others {
                                    let Some(extra) = cards
                                        .cards
                                        .iter()
                                        .find(|c| c.name == name)
                                        .map(|d| d.calculate_cost(&credits) as usize)
                                    else {
                                        continue;
                                    };
                                    if payment_covers(trade_cards, total + extra, grain, treasury_n) {
                                        to_buy.push(name);
                                        total += extra;
                                    }
                                }
                                let (payment, treasury_tokens) =
                                    compute_ai_payment(trade_cards, total, grain, treasury_n);
                                purchase_writer.write(ConfirmCivCardPurchase {
                                    player: event.player,
                                    cards_to_buy: to_buy,
                                    payment,
                                    treasury_tokens,
                                });
                            } else {
                                done_writer
                                    .write(PlayerDoneAcquiringCivilizationCards(event.player));
                            }
                        } else {
                            done_writer.write(PlayerDoneAcquiringCivilizationCards(event.player));
                        }
                    }
                    AcquireCivilizationCardsMove::AcquireCards(_) => {
                        // Not generated by recalculate_civ_card_moves_for_player; skip.
                        done_writer.write(PlayerDoneAcquiringCivilizationCards(event.player));
                    }
                }
            }
        }
    }
}

/// Sum the credit (future-discount) value a civ card hands out, regardless of which
/// card type / card it applies to — a coarse proxy for "how much tech synergy".
fn civ_card_credit_value(def: &CivCardDefinition) -> u32 {
    def.credits
        .iter()
        .map(|credit| match credit {
            Credits::ToType(_, c) | Credits::ToAll(c) | Credits::ToSpecificCard(_, c) => *c,
        })
        .sum()
}

/// Greedy minimum payment: take stacks sorted by descending face value, using just
/// enough cards from each to cover `cost`.  Returns a HashMap<TradeCard, count>.
///
/// `grain_locked` (rule 30.312) is how many of the player's Grain cards are
/// tied up face-up from a Famine/Pottery reduction this turn and therefore
/// unusable -- the Grain stack's effective count (and its count²×face
/// suite_value, recomputed here since the raw stack's is based on the full
/// count) is capped before scoring, so the AI never even considers spending
/// locked Grain.
/// Picks the commodity cards (and, for whatever they don't cover, the treasury
/// tokens) an AI hands over for a civ card costing `cost`.
///
/// Cards first, tokens only for the shortfall: rule 31.4 forbids intentionally
/// spending more treasury tokens than required. Returns the card payment and
/// how many tokens to spend.
pub fn compute_ai_payment(
    trade_cards: &PlayerTradeCards,
    cost: usize,
    grain_locked: usize,
    treasury_available: usize,
) -> (
    bevy::platform::collections::HashMap<TradeCard, usize>,
    usize,
) {
    let mut stacks = trade_cards.as_card_stacks_sorted_by_value();
    if grain_locked > 0
        && let Some(grain_stack) = stacks.iter_mut().find(|s| s.card_type == TradeCard::Grain)
    {
        grain_stack.count = crate::civilization::resolve_calamities::resolve_calamities_components::usable_grain_count(
            grain_stack.count,
            grain_locked,
        );
    }
    stacks.retain(|s| s.is_commodity && s.count > 0 && s.card_type.value() > 0);
    let types: Vec<(TradeCard, usize, usize)> = stacks
        .iter()
        .map(|s| (s.card_type, s.count, s.card_type.value()))
        .collect();
    let (counts, paid) = cheapest_payment(&types, cost, treasury_available);
    let payment = types
        .iter()
        .zip(counts)
        .filter(|(_, k)| *k > 0)
        .map(|((card, ..), k)| (*card, k))
        .collect();
    // Rule 31.4: tokens only for what the cards leave unpaid.
    let tokens = cost.saturating_sub(paid).min(treasury_available);
    (payment, tokens)
}

/// Whether `compute_ai_payment` can cover `cost` from this hand and treasury.
pub fn payment_covers(
    trade_cards: &PlayerTradeCards,
    cost: usize,
    grain_locked: usize,
    treasury_available: usize,
) -> bool {
    let (payment, tokens) = compute_ai_payment(trade_cards, cost, grain_locked, treasury_available);
    let paid: usize = payment.iter().map(|(card, k)| card.value() * k * k).sum();
    paid + tokens >= cost
}

/// Picks how many of each commodity `(card, held, face)` to spend on `cost`.
///
/// k cards of a type pay `face * k^2` (rule 28.2), but they take
/// `face * (held^2 - (held-k)^2)` off the hand: one card from a set of four
/// pays 1x face and costs 7x. So the cheapest payment is the one that loses
/// the least hand value, plus the treasury tokens it needs on top (rule
/// 31.4: tokens only cover the shortfall). Hands are small (a dozen cards),
/// so every combination is tried. Returns the counts and what the cards pay.
/// If even everything cannot cover `cost`, all cards are offered.
fn cheapest_payment(
    types: &[(TradeCard, usize, usize)],
    cost: usize,
    treasury: usize,
) -> (Vec<usize>, usize) {
    if cost == 0 {
        return (vec![0; types.len()], 0);
    }
    let combinations: usize = types.iter().map(|(_, n, _)| n + 1).product();
    let all: Vec<usize> = types.iter().map(|(_, n, _)| *n).collect();
    let paid_by = |ks: &[usize]| -> usize {
        types.iter().zip(ks).map(|((_, _, f), k)| f * k * k).sum()
    };
    if combinations > 500_000 {
        // Not a hand this game produces; spend everything rather than hang.
        let paid = paid_by(&all);
        return (all, paid);
    }
    let mut best: Option<(usize, usize, usize, Vec<usize>)> = None; // (loss, tokens, paid, ks)
    let mut ks = vec![0usize; types.len()];
    loop {
        let paid = paid_by(&ks);
        if paid + treasury >= cost {
            let lost: usize = types
                .iter()
                .zip(&ks)
                .map(|((_, n, f), k)| f * (n * n - (n - k) * (n - k)))
                .sum();
            let tokens = cost.saturating_sub(paid);
            let loss = lost + tokens;
            // Ties go to fewer tokens: they are population and score, while
            // spare cards past eight are shed anyway.
            if best.as_ref().is_none_or(|(l, t, _, _)| (loss, tokens) < (*l, *t)) {
                best = Some((loss, tokens, paid, ks.clone()));
            }
        }
        // Next combination (odometer over 0..=held per type).
        let mut i = 0;
        while i < ks.len() {
            if ks[i] < types[i].1 {
                ks[i] += 1;
                break;
            }
            ks[i] = 0;
            i += 1;
        }
        if i == ks.len() {
            break;
        }
    }
    best.map_or_else(
        || {
            let paid = paid_by(&all);
            (all, paid)
        },
        |(_, _, paid, ks)| (ks, paid),
    )
}

/// Per-player inputs to the rule-35 score estimate.
pub type StandingsData = (
    Entity,
    &'static AstPosition,
    &'static PlayerCities,
    Option<&'static PlayerCivilizationCards>,
    Option<&'static PlayerTradeCards>,
    Option<&'static Treasury>,
);

/// Every player's rule-35 score estimate (what they'd score if the game ended now).
pub fn standings(
    query: &Query<StandingsData, With<Player>>,
    defs: Option<&AvailableCivCards>,
) -> HashMap<Entity, u32> {
    query
        .iter()
        .map(|(e, ast, cities, civ, trade, treasury)| {
            let face = match (civ, defs) {
                (Some(c), Some(d)) => d.cards_for_names(&c.cards).iter().map(|x| x.cost).sum(),
                _ => 0,
            };
            (
                e,
                rule35_score(
                    ast.space,
                    cities.number_of_cities(),
                    face,
                    trade.map_or(0, |t| t.total_stack_value() as u32),
                    treasury.map_or(0, |t| t.tokens_in_treasury() as u32),
                ),
            )
        })
        .collect()
}

/// 1.0 when the player is short of the cities needed to clear their *next*
/// A.S.T. epoch gate, else 0.0. Overrides a low `city_income` personality knob in
/// both movement (pile tokens onto a site) and city construction (build on it).
fn city_urgency(player_cities: &PlayerCities, ast_position: &AstPosition) -> f32 {
    let next_epoch = AstEpoch::for_space(ast_position.space + 1);
    if player_cities.number_of_cities() < next_epoch.min_cities() {
        1.0
    } else {
        0.0
    }
}

/// Flatten every area into an [`AreaSummary`] keyed by area entity, from `player`'s
/// point of view (my_pop vs enemy_pop). Gathered once per movement decision so the
/// scoring functions stay pure.
fn gather_area_summaries(
    player: Entity,
    area_info_query: &Query<(
        Entity,
        &Population,
        &LandPassage,
        Option<&BuiltCity>,
        Has<CitySite>,
    )>,
) -> HashMap<Entity, AreaSummary> {
    let mut areas = HashMap::default();
    for (entity, population, land_passage, built_city, is_city_site) in area_info_query.iter() {
        let my_pop = population.population_for_player(player);
        let enemy_pop = population.total_population() - my_pop;
        areas.insert(
            entity,
            AreaSummary {
                max_population: population.max_population,
                is_city_site,
                has_city: built_city.is_some(),
                city_is_mine: built_city.is_some_and(|c| c.player == player),
                my_pop,
                enemy_pop,
                neighbours: land_passage.to_areas.clone(),
                city_owner: built_city.map(|c| c.player),
                main_enemy: population
                    .player_tokens()
                    .iter()
                    .filter(|(p, _)| **p != player)
                    .max_by_key(|(p, t)| (t.len(), **p))
                    .map(|(p, _)| *p),
                city_defence: 0, // rule default; movement raises it for Engineering
            },
        );
    }
    areas
}

/// Drops `EndMovement` from the scored moves if any other move scores higher.
fn without_premature_end(
    scored: &[(usize, f32)],
    moves: &HashMap<usize, GameMove>,
) -> Vec<(usize, f32)> {
    let is_end = |i: &usize| matches!(moves.get(i), Some(GameMove::EndMovement));
    let end_score = scored
        .iter()
        .find(|(i, _)| is_end(i))
        .map(|(_, s)| *s);
    let Some(end_score) = end_score else {
        return scored.to_vec();
    };
    let beaten = scored
        .iter()
        .any(|(i, s)| !is_end(i) && s.is_finite() && *s > end_score);
    scored
        .iter()
        .copied()
        .filter(|(i, _)| !(beaten && is_end(i)))
        .collect()
}

/// How many of `wanted` tokens to move into `target` on an ordinary
/// (unplanned) move: never more than the free room there, since anything over
/// the area's limit is removed as surplus at the end of the turn. Always at
/// least 1, so a chosen move is never a no-op.
fn fit_to_room(wanted: usize, target: Entity, areas: &HashMap<Entity, AreaSummary>) -> usize {
    let room = areas
        .get(&target)
        .map_or(wanted, |a| a.max_population.saturating_sub(a.my_pop + a.enemy_pop));
    wanted.min(room).max(1)
}

fn send_movement_move(
    move_tokens_writer: &mut MessageWriter<MoveTokenFromAreaToAreaCommand>,
    event: &SelectStupidMove,
    movement_move: &MovementMove,
    is_attack: bool,
) {
    if is_attack {
        // Commit the attack with at least one token: keep one back if we can, but
        // never send a 0-token move (which is a no-op the AI would re-pick forever).
        let n = movement_move.max_tokens.saturating_sub(1).max(1);
        move_tokens_writer.write(MoveTokenFromAreaToAreaCommand::new(
            movement_move.source,
            movement_move.target,
            n,
            event.player,
        ));
    } else {
        match movement_move.max_tokens {
            1 | 2 => {
                move_tokens_writer.write(MoveTokenFromAreaToAreaCommand::new(
                    movement_move.source,
                    movement_move.target,
                    1,
                    event.player,
                ));
            }
            _ => {
                move_tokens_writer.write(MoveTokenFromAreaToAreaCommand::new(
                    movement_move.source,
                    movement_move.target,
                    2,
                    event.player,
                ));
            }
        }
    }
}

#[cfg(test)]
mod end_movement_tests {
    use super::*;

    #[test]
    fn end_movement_only_competes_when_nothing_beats_it() {
        let moves = HashMap::from_iter([
            (1, GameMove::EndMovement),
            (2, GameMove::EndCityConstruction), // any non-End stand-in
        ]);
        let ids = |v: Vec<(usize, f32)>| v.into_iter().map(|(i, _)| i).collect::<Vec<_>>();
        assert_eq!(ids(without_premature_end(&[(1, 0.15), (2, 0.6)], &moves)), vec![2]);
        assert_eq!(ids(without_premature_end(&[(1, 0.15), (2, 0.1)], &moves)), vec![1, 2]);
    }
}

#[cfg(test)]
mod grain_lock_tests {
    use super::*;

    // ── Rule 30.312: locked Grain is never offered as AI payment ───────────

    #[test]
    fn no_lock_can_still_spend_all_grain() {
        let mut hand = PlayerTradeCards::default();
        for _ in 0..3 {
            hand.add_trade_card(TradeCard::Grain);
        }
        // 3 Grain -> suite_value 3*3*4 = 36, plenty to cover a cost of 20.
        let (payment, _) = compute_ai_payment(&hand, 20, 0, 0);
        assert_eq!(payment.get(&TradeCard::Grain).copied(), Some(3));
    }

    #[test]
    fn fully_locked_grain_is_never_selected() {
        let mut hand = PlayerTradeCards::default();
        for _ in 0..3 {
            hand.add_trade_card(TradeCard::Grain);
        }
        // All 3 held Grain are locked -> usable count 0 -> can't cover any cost.
        let (payment, _) = compute_ai_payment(&hand, 20, 3, 0);
        assert_eq!(payment.get(&TradeCard::Grain), None);
    }

    #[test]
    fn partially_locked_grain_only_offers_the_unlocked_remainder() {
        let mut hand = PlayerTradeCards::default();
        for _ in 0..5 {
            hand.add_trade_card(TradeCard::Grain);
        }
        // 5 held, 3 locked -> 2 usable -> suite_value 2*2*4 = 16, not enough
        // for a cost of 20, so the whole usable stack is spent (not the full 5).
        let (payment, _) = compute_ai_payment(&hand, 20, 3, 0);
        assert_eq!(payment.get(&TradeCard::Grain).copied(), Some(2));
    }

    #[test]
    fn locked_grain_does_not_affect_other_commodity_types() {
        let mut hand = PlayerTradeCards::default();
        for _ in 0..3 {
            hand.add_trade_card(TradeCard::Grain);
        }
        for _ in 0..3 {
            hand.add_trade_card(TradeCard::Salt);
        }
        // Grain fully locked; Salt is untouched and should still be offered.
        let (payment, _) = compute_ai_payment(&hand, 5, 3, 0);
        assert_eq!(payment.get(&TradeCard::Grain), None);
        assert!(payment.get(&TradeCard::Salt).copied().unwrap_or(0) > 0);
    }

    // ── Rule 31.1/31.4: treasury tokens cover what the cards can't ─────────

    #[test]
    fn treasury_tokens_only_cover_the_shortfall() {
        let mut hand = PlayerTradeCards::default();
        hand.add_trade_card(TradeCard::Ochre); // 1 card, face 1 -> value 1
        // Cost 5, cards cover 1, so 4 tokens are needed out of the 10 held.
        let (_, tokens) = compute_ai_payment(&hand, 5, 0, 10);
        assert_eq!(tokens, 4);
    }

    #[test]
    fn no_treasury_tokens_are_spent_when_the_cards_already_pay() {
        let mut hand = PlayerTradeCards::default();
        for _ in 0..3 {
            hand.add_trade_card(TradeCard::Grain);
        }
        // 3 Grain cover 36 against a cost of 20 -- rule 31.4 forbids topping
        // that up with tokens the purchase does not need.
        let (_, tokens) = compute_ai_payment(&hand, 20, 0, 10);
        assert_eq!(tokens, 0);
    }

    // ── Paying costs hand value, not face value ────────────────────────────

    #[test]
    fn pays_without_breaking_up_a_set_when_singletons_will_do() {
        let mut hand = PlayerTradeCards::default();
        for _ in 0..4 {
            hand.add_trade_card(TradeCard::Ochre); // 4-set, worth 16
        }
        hand.add_trade_card(TradeCard::Salt); // 3
        hand.add_trade_card(TradeCard::Iron); // 2
        // Cost 5: Salt + Iron pay exactly 5 and lose 5; any Ochre would cost 7+.
        let (payment, tokens) = compute_ai_payment(&hand, 5, 0, 0);
        assert_eq!(payment.get(&TradeCard::Salt).copied(), Some(1));
        assert_eq!(payment.get(&TradeCard::Iron).copied(), Some(1));
        assert_eq!(payment.get(&TradeCard::Ochre), None);
        assert_eq!(tokens, 0);
    }

    #[test]
    fn a_whole_set_is_cheaper_than_part_of_it_plus_a_bigger_set() {
        let mut hand = PlayerTradeCards::default();
        for _ in 0..3 {
            hand.add_trade_card(TradeCard::Salt); // 27
        }
        for _ in 0..3 {
            hand.add_trade_card(TradeCard::Grain); // 36
        }
        // Cost 27: all the Salt pays exactly; the old greedy started with the
        // higher face value and spent 3 Grain (36) instead.
        let (payment, _) = compute_ai_payment(&hand, 27, 0, 0);
        assert_eq!(payment.get(&TradeCard::Salt).copied(), Some(3));
        assert_eq!(payment.get(&TradeCard::Grain), None);
    }

    #[test]
    fn treasury_spending_is_capped_by_what_is_actually_held() {
        let hand = PlayerTradeCards::default();
        let (_, tokens) = compute_ai_payment(&hand, 40, 0, 6);
        assert_eq!(tokens, 6);
    }
}
