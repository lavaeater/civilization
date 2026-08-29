//! Bridges lightyear connections to game seats, and translates between the
//! Entity-based `GameMove`s of the rules engine and the stable-id network
//! protocol. This is the seam described in docs/multiplayer.md: clients only
//! ever pick from moves the server offered.

use crate::game::{Seats, find_seat_for_join};
use adv_civ::civilization::*;
use adv_civ::player::Player;
use adv_civ::{GameActivity, GameState};
use adv_civ_protocol::*;
use bevy::prelude::*;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use lightyear::prelude::server::*;
use lightyear::prelude::*;

fn server_addr() -> SocketAddr {
    SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        crate::server_port(),
    )
}

pub struct NetBridgePlugin;

impl Plugin for NetBridgePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NeedsFullSync>();
        app.add_systems(Startup, start_server).add_systems(
            Update,
            (
                handle_joins,
                sync_joined_clients.after(handle_joins),
                receive_moves,
                send_available_moves,
                send_hands,
                broadcast_phase_changes,
                broadcast_board_state,
            ),
        );
        app.add_systems(
            Update,
            (
                receive_propose_trade_offers,
                receive_accept_trade_offers,
                receive_settle_trade_offers,
                broadcast_trade_offers,
            ),
        );
        app.add_observer(on_client_disconnected);
    }
}

fn start_server(mut commands: Commands, keys: Res<crate::http::NetcodeKeys>) {
    // Plain ws:// for now. TLS terminates at a reverse proxy in the Docker
    // deployment (docs/multiplayer.md, "Transport").
    let addr = server_addr();
    let config = ServerConfig::builder()
        .with_bind_default(crate::server_port())
        .with_no_encryption();
    let server = commands
        .spawn((
            NetcodeServer::new(
                NetcodeConfig::default()
                    .with_protocol_id(keys.protocol_id)
                    .with_key(keys.key),
            ),
            LocalAddr(addr),
            WebSocketServerIo { config },
        ))
        .id();
    commands.trigger(Start { entity: server });
    // The transport binds all interfaces on the port (verified: `*:PORT`);
    // `addr` above is just the informational LocalAddr. Report what clients
    // actually need: the port to reach and the URL we advertise to them.
    info!(
        "WebSocket game server listening on 0.0.0.0:{} (advertised to clients as {})",
        crate::server_port(),
        crate::http::public_ws_url()
    );
}

/// Assign joining clients to free seats; once every seat is claimed, leave
/// the lobby and start the game proper (Loading→Menu happened at boot).
///
/// Two join paths land here: an explicit `JoinGame` message (dev clients
/// with manual auth), and netcode connections whose client id was
/// registered through the HTTP join endpoint (`PendingJoins`).
#[allow(clippy::type_complexity)]
fn handle_joins(
    mut receivers: Query<(Entity, &RemoteId, &mut MessageReceiver<JoinGame>), With<ClientOf>>,
    connected: Query<(Entity, &RemoteId), (With<ClientOf>, Added<Connected>)>,
    mut pending: ResMut<crate::http::PendingJoins>,
    mut seats: ResMut<Seats>,
    mut player_names: Query<&mut Name, With<Player>>,
    mut sender: ServerMultiMessageSender,
    server: Single<&Server>,
    game_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut needs_sync: ResMut<NeedsFullSync>,
) -> Result {
    let server = server.into_inner();
    let mut lobby_changed = false;

    let mut joins: Vec<(Entity, PeerId, String, Option<String>)> = Vec::new();
    for (client_entity, remote_id, mut receiver) in receivers.iter_mut() {
        for join in receiver.receive() {
            joins.push((
                client_entity,
                remote_id.0,
                join.player_name,
                join.reconnect_token,
            ));
        }
    }
    for (client_entity, remote_id) in connected.iter() {
        if let PeerId::Netcode(client_id) = remote_id.0
            && let Some((name, token)) = pending.0.remove(&client_id)
        {
            joins.push((client_entity, remote_id.0, name, token));
        }
    }

    for (client_entity, peer, player_name, token) in joins {
        if seats.by_client(client_entity).is_some() {
            continue;
        }
        // Session-token hardening (docs/multiplayer.md): a name match alone
        // only claims a seat that's never had a token bound; reclaiming a
        // bound identity needs the matching token. See `find_seat_for_join`.
        let seat_index = find_seat_for_join(&seats.0, &player_name, token.as_deref());
        let Some(seat) = seat_index.map(|i| &mut seats.0[i]) else {
            info!("Rejecting {player_name}: all seats taken");
            continue;
        };
        seat.client = Some(client_entity);
        seat.peer = Some(peer);
        seat.name = Some(player_name.clone());
        // Bind a token to this identity the first time one is available —
        // whether newly minted (HTTP join) or already on file (a genuine
        // reconnect); a seat that already has one keeps it.
        if seat.reconnect_token.is_none()
            && let Some(token) = token
        {
            seat.reconnect_token = Some(token);
        }
        // Mid-game (re)join: the seat's player exists, rename it now.
        // In the lobby, bind_seats applies the name at StartGame instead.
        if let Some(player) = seat.player
            && let Ok(mut name) = player_names.get_mut(player)
        {
            *name = Name::new(player_name.clone());
        }
        info!("{player_name} claimed seat {}", seat.faction);
        sender.send::<_, ControlChannel>(
            &JoinAccepted {
                player_name,
                faction: seat.faction,
            },
            server,
            &NetworkTarget::Single(peer),
        )?;
        // Mid-game (re)joiners would otherwise see nothing until the next
        // state change; push them everything.
        needs_sync.0.push(peer);
        lobby_changed = true;
    }

    if lobby_changed {
        let lobby = LobbyState {
            players: seats
                .0
                .iter()
                .map(|s| LobbyPlayer {
                    name: s.name.clone().unwrap_or_else(|| "open".into()),
                    faction: s.faction,
                    connected: s.client.is_some(),
                })
                .collect(),
            seats_total: seats.0.len(),
        };
        sender.send::<_, ControlChannel>(&lobby, server, &NetworkTarget::All)?;

        if seats.all_claimed() && *game_state.get() == GameState::Menu {
            info!("All seats claimed — starting the game");
            next_state.set(GameState::Playing);
        }
    }
    Ok(())
}

/// Whenever the rules engine computes fresh `AvailableMoves` for a seated
/// player, translate them to stable ids and send them to that client only.
fn send_available_moves(
    changed: Query<(Entity, &AvailableMoves), Changed<AvailableMoves>>,
    seats: Res<Seats>,
    areas: Query<&GameArea>,
    factions: Query<&Faction>,
    mut sender: ServerMultiMessageSender,
    server: Single<&Server>,
) -> Result {
    let server = server.into_inner();
    for (player, available) in changed.iter() {
        let Some(seat) = seats.by_player(player) else {
            continue;
        };
        let Some(peer) = seat.peer else { continue };

        let mut moves: Vec<(usize, NetGameMove)> = available
            .moves
            .iter()
            .filter_map(|(index, game_move)| {
                to_net_move(game_move, &areas, &factions).map(|net| (*index, net))
            })
            .collect();
        moves.sort_by_key(|(index, _)| *index);

        info!("Sending {} moves to {}", moves.len(), seat.faction);
        sender.send::<_, ControlChannel>(
            &YourMoves { moves },
            server,
            &NetworkTarget::Single(peer),
        )?;
    }
    Ok(())
}

fn to_net_move(
    game_move: &GameMove,
    areas: &Query<&GameArea>,
    factions: &Query<&Faction>,
) -> Option<NetGameMove> {
    let area_id = |entity: Entity| areas.get(entity).ok().map(|a| AreaId(a.id));
    Some(match game_move {
        GameMove::PopulationExpansion(pop_exp) => NetGameMove::PopulationExpansion {
            area: area_id(pop_exp.area)?,
            max_tokens: pop_exp.max_tokens,
        },
        GameMove::Movement(m) => NetGameMove::Movement {
            source: area_id(m.source)?,
            target: area_id(m.target)?,
            max_tokens: m.max_tokens,
        },
        GameMove::ShipFerry(m) => NetGameMove::ShipFerry {
            source: area_id(m.source)?,
            target: area_id(m.target)?,
            max_tokens: m.max_tokens,
        },
        GameMove::AttackArea(m) => NetGameMove::AttackArea {
            source: area_id(m.source)?,
            target: area_id(m.target)?,
            max_tokens: m.max_tokens,
        },
        GameMove::AttackCity(m) => NetGameMove::AttackCity {
            source: area_id(m.source)?,
            target: area_id(m.target)?,
            max_tokens: m.max_tokens,
        },
        GameMove::EndMovement => NetGameMove::EndMovement,
        GameMove::CityConstruction(build) => NetGameMove::BuildCity {
            area: area_id(build.target)?,
        },
        GameMove::EndCityConstruction => NetGameMove::EndCityConstruction,
        GameMove::EliminateCity(elim) => NetGameMove::EliminateCity {
            area: area_id(elim.area)?,
            tokens_gained: elim.tokens_gained,
            tokens_needed: elim.tokens_needed,
        },
        GameMove::Trade(trade) => NetGameMove::Trade(match trade {
            TradeMove::ProposeTrade(receiver, matching) => NetTradeMove::ProposeTrade {
                to: factions.get(*receiver).ok()?.faction,
                matching_cards: matching.iter().map(|(c, n)| (*c, *n)).collect(),
            },
            TradeMove::AcceptOrDeclineTrade(offer) => NetTradeMove::AcceptOrDeclineTrade {
                offer: NetOfferId(offer.to_bits()),
            },
            TradeMove::AutoDeclineTrade(offer) => NetTradeMove::AutoDeclineTrade {
                offer: NetOfferId(offer.to_bits()),
            },
            TradeMove::StopTrading => NetTradeMove::StopTrading,
            TradeMove::SettleTrade(offer) => NetTradeMove::SettleTrade {
                offer: NetOfferId(offer.to_bits()),
            },
        }),
        GameMove::AcquireCivilizationCards(civ) => match civ {
            AcquireCivilizationCardsMove::AcquireCard(card) => {
                NetGameMove::AcquireCivCards { cards: vec![*card] }
            }
            AcquireCivilizationCardsMove::AcquireCards(cards) => NetGameMove::AcquireCivCards {
                cards: cards.clone(),
            },
            AcquireCivilizationCardsMove::DoneAcquiringCards => NetGameMove::DoneAcquiringCivCards,
        },
    })
}

/// All the per-phase command writers the move dispatch can feed. Same
/// messages the AI writes — the rules engine can't tell humans and AI apart.
#[derive(bevy::ecs::system::SystemParam)]
struct MoveCommandWriters<'w> {
    expand: MessageWriter<'w, ExpandPopulationManuallyCommand>,
    move_tokens: MessageWriter<'w, MoveTokenFromAreaToAreaCommand>,
    ship_ferry: MessageWriter<'w, ShipFerryCommand>,
    end_movement: MessageWriter<'w, PlayerMovementEnded>,
    build_city: MessageWriter<'w, BuildCityCommand>,
    end_city_construction: MessageWriter<'w, EndPlayerCityConstruction>,
    eliminate_city: MessageWriter<'w, EliminateCity>,
    confirm_civ_purchase: MessageWriter<'w, ConfirmCivCardPurchase>,
    done_acquiring_civ: MessageWriter<'w, PlayerDoneAcquiringCivilizationCards>,
}

/// Validate a submitted move against the offering in `AvailableMoves` and
/// feed it to the rules engine through the same command the AI uses. The
/// client echoes the server's move index, so the original Entity-based
/// `GameMove` is replayed — no inbound id translation exists at all.
fn receive_moves(
    mut receivers: Query<(Entity, &mut MessageReceiver<SubmitMove>), With<ClientOf>>,
    seats: Res<Seats>,
    available: Query<&AvailableMoves>,
    mut writers: MoveCommandWriters,
    mut commands: Commands,
    mut sender: ServerMultiMessageSender,
    server: Single<&Server>,
) -> Result {
    let server = server.into_inner();
    for (client_entity, mut receiver) in receivers.iter_mut() {
        for submit in receiver.receive() {
            let Some(seat) = seats.by_client(client_entity) else {
                continue;
            };
            let Some(peer) = seat.peer else { continue };

            let reject = |reason: &str, sender: &mut ServerMultiMessageSender| {
                sender.send::<_, ControlChannel>(
                    &MoveRejected {
                        move_index: submit.move_index,
                        reason: reason.into(),
                    },
                    server,
                    &NetworkTarget::Single(peer),
                )
            };

            let Some(player) = seat.player else {
                reject("game has not started yet", &mut sender)?;
                continue;
            };
            let Ok(moves) = available.get(player) else {
                reject("no moves available", &mut sender)?;
                continue;
            };
            let Some(game_move) = moves.moves.get(&submit.move_index) else {
                reject("unknown move index", &mut sender)?;
                continue;
            };
            info!("{} submits: {:?}", seat.faction, game_move);

            // Token-count choice for moves that allow one. Default mirrors
            // the AI: move everything but leave one token behind if possible.
            let pick_tokens = |max: usize, leave_one_default: bool| {
                let default = if leave_one_default {
                    max.saturating_sub(1).max(1)
                } else {
                    max
                };
                submit.tokens.unwrap_or(default).clamp(1, max.max(1))
            };

            match game_move {
                GameMove::PopulationExpansion(pop_exp) => {
                    let tokens = pick_tokens(pop_exp.max_tokens, false);
                    writers.expand.write(ExpandPopulationManuallyCommand::new(
                        player,
                        pop_exp.area,
                        tokens,
                    ));
                }
                GameMove::Movement(m) | GameMove::AttackArea(m) | GameMove::AttackCity(m) => {
                    let tokens = pick_tokens(m.max_tokens, true);
                    writers
                        .move_tokens
                        .write(MoveTokenFromAreaToAreaCommand::new(
                            m.source, m.target, tokens, player,
                        ));
                }
                GameMove::ShipFerry(m) => {
                    let tokens = pick_tokens(m.max_tokens, true);
                    writers
                        .ship_ferry
                        .write(ShipFerryCommand::new(m.source, m.target, tokens, player));
                }
                GameMove::EndMovement => {
                    writers.end_movement.write(PlayerMovementEnded::new(player));
                }
                GameMove::CityConstruction(build) => {
                    writers
                        .build_city
                        .write(BuildCityCommand::new(player, build.target));
                }
                GameMove::EndCityConstruction => {
                    writers
                        .end_city_construction
                        .write(EndPlayerCityConstruction::new(player));
                }
                GameMove::EliminateCity(elim) => {
                    writers
                        .eliminate_city
                        .write(EliminateCity::new(player, elim.city, elim.area, false));
                }
                GameMove::Trade(TradeMove::StopTrading) => {
                    // Leaving the table: the trade gate ends the phase once
                    // no player still has CanTrade.
                    commands.entity(player).remove::<CanTrade>();
                }
                GameMove::Trade(_) => {
                    // Interactive trading (propose/accept/settle) is the
                    // acknowledged hard case — deferred to its own milestone.
                    reject("interactive trading not yet supported", &mut sender)?;
                }
                GameMove::AcquireCivilizationCards(civ) => match civ {
                    AcquireCivilizationCardsMove::AcquireCard(card) => {
                        writers.confirm_civ_purchase.write(ConfirmCivCardPurchase {
                            player,
                            cards_to_buy: vec![*card],
                            payment: submit.payment.iter().copied().collect(),
                            // Protocol doesn't expose treasury-token payment yet;
                            // networked purchases pay in commodity cards only.
                            treasury_tokens: 0,
                        });
                    }
                    AcquireCivilizationCardsMove::AcquireCards(cards) => {
                        writers.confirm_civ_purchase.write(ConfirmCivCardPurchase {
                            player,
                            cards_to_buy: cards.clone(),
                            payment: submit.payment.iter().copied().collect(),
                            treasury_tokens: 0,
                        });
                    }
                    AcquireCivilizationCardsMove::DoneAcquiringCards => {
                        writers
                            .done_acquiring_civ
                            .write(PlayerDoneAcquiringCivilizationCards(player));
                    }
                },
            }
        }
    }
    Ok(())
}

/// Tell clients about phase transitions so they can drive their UI flow.
fn broadcast_phase_changes(
    activity: Option<Res<State<GameActivity>>>,
    mut last_seen: Local<Option<GameActivity>>,
    mut sender: ServerMultiMessageSender,
    server: Single<&Server>,
) -> Result {
    let current = activity.map(|s| s.get().clone());
    if current != *last_seen {
        if let Some(phase) = current.as_ref() {
            info!("Phase changed: {phase:?}");
            sender.send::<_, ControlChannel>(
                &PhaseChanged {
                    phase: phase.into(),
                },
                server.into_inner(),
                &NetworkTarget::All,
            )?;
        }
        *last_seen = current;
    }
    Ok(())
}

type BoardAreaQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static GameArea,
        &'static Name,
        &'static Population,
        Option<&'static BuiltCity>,
    ),
>;
type BoardPlayerQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Name,
        &'static Faction,
        &'static TokenStock,
        Option<&'static PlayerCivilizationCards>,
        Option<&'static PlayerTradeCards>,
    ),
    With<Player>,
>;

/// Compose the public board state. Hidden information (hands) must never go
/// in here — it travels per-seat via [`send_hands`]/[`sync_joined_clients`].
fn compose_board_view(
    areas: &BoardAreaQuery,
    players: &BoardPlayerQuery,
    factions: &Query<&Faction>,
) -> GameStateView {
    let mut area_views: Vec<AreaView> = areas
        .iter()
        .map(|(area, name, population, built_city)| AreaView {
            area: AreaId(area.id),
            name: name.to_string(),
            max_population: population.max_population,
            population: population
                .players()
                .into_iter()
                .filter_map(|player| {
                    let faction = factions.get(player).ok()?.faction;
                    Some((faction, population.population_for_player(player)))
                })
                .collect(),
            city: built_city.and_then(|c| factions.get(c.player).ok().map(|f| f.faction)),
        })
        .collect();
    area_views.sort_by_key(|view| view.area);

    GameStateView {
        areas: area_views,
        players: players
            .iter()
            .map(
                |(name, faction, stock, civ_cards, trade_cards)| PlayerView {
                    name: name.to_string(),
                    faction: faction.faction,
                    tokens_in_stock: stock.tokens_in_stock(),
                    civ_cards: civ_cards
                        .map(|c| c.cards.iter().copied().collect())
                        .unwrap_or_default(),
                    trade_card_count: trade_cards
                        .map(|t| t.number_of_trade_cards())
                        .unwrap_or_default(),
                },
            )
            .collect(),
    }
}

/// Broadcast the public board state whenever populations or stocks change.
#[allow(clippy::type_complexity)]
fn broadcast_board_state(
    changed: Query<(), Or<(Changed<Population>, Changed<TokenStock>, Changed<BuiltCity>)>>,
    areas: BoardAreaQuery,
    players: BoardPlayerQuery,
    factions: Query<&Faction>,
    seats: Res<Seats>,
    mut sender: ServerMultiMessageSender,
    server: Single<&Server>,
) -> Result {
    if changed.is_empty() || seats.0.iter().all(|s| s.client.is_none()) {
        return Ok(());
    }
    let view = compose_board_view(&areas, &players, &factions);
    sender.send::<_, ControlChannel>(&view, server.into_inner(), &NetworkTarget::All)?;
    Ok(())
}

/// Peers that just claimed a seat and need the complete current state.
#[derive(Resource, Default)]
struct NeedsFullSync(Vec<PeerId>);

/// Push phase + board + private hand + pending moves to fresh (re)joiners,
/// so reconnecting mid-game resumes instantly instead of waiting for the
/// next state change.
fn sync_joined_clients(
    mut needs_sync: ResMut<NeedsFullSync>,
    seats: Res<Seats>,
    activity: Option<Res<State<GameActivity>>>,
    areas: BoardAreaQuery,
    players: BoardPlayerQuery,
    factions: Query<&Faction>,
    area_ids: Query<&GameArea>,
    hands: Query<&PlayerTradeCards>,
    available: Query<&AvailableMoves>,
    open_offers: Query<(Entity, &OpenTradeOffer)>,
    mut sender: ServerMultiMessageSender,
    server: Single<&Server>,
) -> Result {
    if needs_sync.0.is_empty() {
        return Ok(());
    }
    let server = server.into_inner();
    let board = compose_board_view(&areas, &players, &factions);
    let trade_offers = compose_trade_offers_view(&open_offers, &seats);

    for peer in needs_sync.0.drain(..) {
        let target = NetworkTarget::Single(peer);
        sender.send::<_, ControlChannel>(&trade_offers, server, &target)?;
        if let Some(activity) = &activity {
            sender.send::<_, ControlChannel>(
                &PhaseChanged {
                    phase: activity.get().into(),
                },
                server,
                &target,
            )?;
        }
        sender.send::<_, ControlChannel>(&board, server, &target)?;

        let Some(seat) = seats.0.iter().find(|s| s.peer == Some(peer)) else {
            continue;
        };
        let Some(player) = seat.player else { continue };
        if let Ok(hand) = hands.get(player) {
            sender.send::<_, ControlChannel>(
                &YourHand {
                    cards: hand.cards_with_counts(),
                },
                server,
                &target,
            )?;
        }
        if let Ok(moves) = available.get(player) {
            let mut net_moves: Vec<(usize, NetGameMove)> = moves
                .moves
                .iter()
                .filter_map(|(index, game_move)| {
                    to_net_move(game_move, &area_ids, &factions).map(|net| (*index, net))
                })
                .collect();
            net_moves.sort_by_key(|(index, _)| *index);
            sender.send::<_, ControlChannel>(&YourMoves { moves: net_moves }, server, &target)?;
        }
        info!("Synced full state to rejoined seat {}", seat.faction);
    }
    Ok(())
}

/// Hidden information: each seat gets its own trade-card hand, nobody
/// else's. Never part of any broadcast message.
fn send_hands(
    changed: Query<(Entity, &PlayerTradeCards), Changed<PlayerTradeCards>>,
    seats: Res<Seats>,
    mut sender: ServerMultiMessageSender,
    server: Single<&Server>,
) -> Result {
    let server = server.into_inner();
    for (player, hand) in changed.iter() {
        let Some(seat) = seats.by_player(player) else {
            continue;
        };
        let Some(peer) = seat.peer else { continue };
        sender.send::<_, ControlChannel>(
            &YourHand {
                cards: hand.cards_with_counts(),
            },
            server,
            &NetworkTarget::Single(peer),
        )?;
    }
    Ok(())
}

/// Free the seat when its client disconnects so a reconnecting client can
/// claim it again — `name`/`reconnect_token` stay bound, so only the same
/// client (or one presenting the right token) can reclaim it; see
/// `find_seat_for_join` and docs/multiplayer.md's session-token section.
fn on_client_disconnected(trigger: On<Remove, Connected>, mut seats: ResMut<Seats>) {
    if let Some(seat) = seats
        .0
        .iter_mut()
        .find(|s| s.client == Some(trigger.entity))
    {
        info!("Client for seat {} disconnected", seat.faction);
        seat.client = None;
        seat.peer = None;
    }
}

// ── Trade over the network ───────────────────────────────────────────────
//
// The one real trade model is `OpenTradeOffer` (see agent-api-design.md — the
// `NetTradeMove`/`TradeMove` pair is dead). `ProposeTradeOffer`/
// `AcceptTradeOffer`/`SettleTradeOffer` are dedicated messages, not
// `SubmitMove` picks, because the card selections are free-form input rather
// than a choice from an enumerated move list — same reasoning as the agent
// API's dedicated `/trade/*` endpoints, which this mirrors closely.

fn card_map(cards: &[(TradeCard, usize)]) -> bevy::platform::collections::HashMap<TradeCard, usize> {
    cards.iter().copied().collect()
}

fn card_vec(cards: &bevy::platform::collections::HashMap<TradeCard, usize>) -> Vec<(TradeCard, usize)> {
    let mut v: Vec<(TradeCard, usize)> = cards.iter().map(|(c, n)| (*c, *n)).collect();
    v.sort_by_key(|(c, _)| format!("{c}"));
    v
}

#[allow(clippy::type_complexity)]
fn receive_propose_trade_offers(
    mut receivers: Query<
        (Entity, &mut MessageReceiver<ProposeTradeOffer>),
        With<ClientOf>,
    >,
    seats: Res<Seats>,
    names: Query<&Name, With<Player>>,
    mut commands: Commands,
) {
    for (client_entity, mut receiver) in receivers.iter_mut() {
        for propose in receiver.receive() {
            let Some(seat) = seats.by_client(client_entity) else {
                continue;
            };
            let Some(player) = seat.player else { continue };
            let creator_name = names
                .get(player)
                .map_or_else(|_| seat.faction.to_string(), ToString::to_string);
            let target = propose
                .target
                .and_then(|f| seats.0.iter().find(|s| s.faction == f))
                .and_then(|s| s.player);
            let target_name = target.and_then(|p| names.get(p).ok()).map(ToString::to_string);

            let mut offer = OpenTradeOffer::new(player, creator_name, target, target_name);
            offer.offering_guaranteed = card_map(&propose.offering_guaranteed);
            offer.offering_hidden_count = propose.offering_hidden_count;
            offer.wanting_guaranteed = card_map(&propose.wanting_guaranteed);
            offer.wanting_hidden_count = propose.wanting_hidden_count;

            if offer.is_valid() {
                info!("{} proposed a trade offer", seat.faction);
                commands.spawn(offer);
            } else {
                warn!(
                    "{} proposed an invalid trade offer (needs exactly 2 guaranteed cards \
                     and >=3 total on each side) — dropped",
                    seat.faction
                );
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn receive_accept_trade_offers(
    mut receivers: Query<(Entity, &mut MessageReceiver<AcceptTradeOffer>), With<ClientOf>>,
    seats: Res<Seats>,
    names: Query<&Name, With<Player>>,
    mut offers: Query<&mut OpenTradeOffer>,
) {
    for (client_entity, mut receiver) in receivers.iter_mut() {
        for accept in receiver.receive() {
            let Some(seat) = seats.by_client(client_entity) else {
                continue;
            };
            let Some(player) = seat.player else { continue };
            let Some(offer_entity) = Entity::try_from_bits(accept.offer.0) else {
                continue;
            };
            let Ok(mut offer) = offers.get_mut(offer_entity) else {
                warn!("{} tried to accept a nonexistent trade offer", seat.faction);
                continue;
            };
            let player_name = names
                .get(player)
                .map_or_else(|_| seat.faction.to_string(), ToString::to_string);
            if offer.accept(player, player_name) {
                info!("{} accepted a trade offer", seat.faction);
            } else {
                warn!(
                    "{} could not accept that trade offer (own offer, already \
                     accepted/withdrawn, or not the target)",
                    seat.faction
                );
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn receive_settle_trade_offers(
    mut receivers: Query<(Entity, &mut MessageReceiver<SettleTradeOffer>), With<ClientOf>>,
    seats: Res<Seats>,
    mut offers: Query<&mut OpenTradeOffer>,
) {
    for (client_entity, mut receiver) in receivers.iter_mut() {
        for settle in receiver.receive() {
            let Some(seat) = seats.by_client(client_entity) else {
                continue;
            };
            let Some(player) = seat.player else { continue };
            let Some(offer_entity) = Entity::try_from_bits(settle.offer.0) else {
                continue;
            };
            let Ok(mut offer) = offers.get_mut(offer_entity) else {
                warn!("{} tried to settle a nonexistent trade offer", seat.faction);
                continue;
            };
            let cards = card_map(&settle.cards);
            if offer.creator == player {
                offer.settle_creator(cards);
                info!("{} settled their side of a trade offer", seat.faction);
            } else if offer.accepted_by == Some(player) {
                offer.settle_acceptor(cards);
                info!("{} settled their side of a trade offer", seat.faction);
            } else {
                warn!(
                    "{} tried to settle a trade offer they're not party to",
                    seat.faction
                );
            }
        }
    }
}

fn to_net_trade_offer(entity: Entity, offer: &OpenTradeOffer, seats: &Seats) -> Option<NetTradeOffer> {
    let creator = seats.by_player(offer.creator)?.faction;
    let target = offer.target.and_then(|p| seats.by_player(p)).map(|s| s.faction);
    let accepted_by = offer.accepted_by.and_then(|p| seats.by_player(p)).map(|s| s.faction);
    Some(NetTradeOffer {
        id: NetOfferId(entity.to_bits()),
        creator,
        target,
        accepted_by,
        settling: offer.is_settling(),
        offering_guaranteed: card_vec(&offer.offering_guaranteed),
        offering_hidden_count: offer.offering_hidden_count,
        wanting_guaranteed: card_vec(&offer.wanting_guaranteed),
        wanting_hidden_count: offer.wanting_hidden_count,
    })
}

fn compose_trade_offers_view(
    all_offers: &Query<(Entity, &OpenTradeOffer)>,
    seats: &Seats,
) -> TradeOffersView {
    TradeOffersView {
        offers: all_offers
            .iter()
            .filter(|(_, o)| !o.withdrawn)
            .filter_map(|(e, o)| to_net_trade_offer(e, o, seats))
            .collect(),
    }
}

/// Broadcast the full open-offers list whenever any offer changes or is
/// removed (settled offers get despawned elsewhere once both sides commit).
fn broadcast_trade_offers(
    changed: Query<(), Changed<OpenTradeOffer>>,
    mut removed: RemovedComponents<OpenTradeOffer>,
    all_offers: Query<(Entity, &OpenTradeOffer)>,
    seats: Res<Seats>,
    mut sender: ServerMultiMessageSender,
    server: Single<&Server>,
) -> Result {
    let any_removed = !removed.is_empty();
    removed.clear();
    if (changed.is_empty() && !any_removed) || seats.0.iter().all(|s| s.client.is_none()) {
        return Ok(());
    }
    let view = compose_trade_offers_view(&all_offers, &seats);
    sender.send::<_, ControlChannel>(&view, server.into_inner(), &NetworkTarget::All)?;
    Ok(())
}
