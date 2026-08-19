use crate::civilization::*;
use crate::stupid_ai::{compute_ai_payment, AgentControlled};
use crate::GameActivity;
use bevy::ecs::system::SystemParam;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use serde_json::{json, Value};
use tiny_http::{Header, Method, Response, Server};

/// The command writers the agent move-translator emits into, bundled so
/// `poll_agent_api` stays under Bevy's system-parameter limit.
#[derive(SystemParam)]
pub struct MoveWriters<'w> {
    expand: MessageWriter<'w, ExpandPopulationManuallyCommand>,
    move_tokens: MessageWriter<'w, MoveTokenFromAreaToAreaCommand>,
    ferry: MessageWriter<'w, ShipFerryCommand>,
    end_move: MessageWriter<'w, PlayerMovementEnded>,
    build_city: MessageWriter<'w, BuildCityCommand>,
    end_city: MessageWriter<'w, EndPlayerCityConstruction>,
    eliminate_city: MessageWriter<'w, EliminateCity>,
    done_civ: MessageWriter<'w, PlayerDoneAcquiringCivilizationCards>,
    purchase: MessageWriter<'w, ConfirmCivCardPurchase>,
}

/// Holds the embedded HTTP server. `tiny_http::Server` is `Send + Sync`, so it
/// lives happily as a Bevy resource and `try_recv` is non-blocking.
#[derive(Resource)]
pub struct AgentServer {
    pub server: Server,
}

// Owned snapshot of everything the API needs for a frame — decoupled from ECS
// queries so the HTTP handlers stay free of borrow-lifetime gymnastics.
struct AreaInfo {
    area_id: i32,
    your_tokens: usize,
    total: usize,
    max: usize,
}

struct PlayerInfo {
    player: Entity,
    name: String,
    faction: GameFaction,
    moves: Vec<(usize, GameMove)>,
    areas: Vec<AreaInfo>,
    can_trade: bool,
    /// Commodity cards in hand, by card name.
    hand: Vec<(String, usize)>,
}

/// An `OpenTradeOffer`, flattened to owned data for the API.
struct OfferInfo {
    id: u64,
    creator: Entity,
    target: Option<Entity>,
    accepted_by: Option<Entity>,
    withdrawn: bool,
    settling: bool,
    offering_guaranteed: Vec<(String, usize)>,
    offering_hidden: usize,
    wanting_guaranteed: Vec<(String, usize)>,
    wanting_hidden: usize,
}

impl OfferInfo {
    /// Mirrors `OpenTradeOffer::can_accept`.
    fn can_accept(&self, player: Entity) -> bool {
        player != self.creator
            && self.accepted_by.is_none()
            && !self.withdrawn
            && self.target.is_none_or(|t| t == player)
    }
}

struct Snapshot {
    phase: String,
    /// Every agent-controlled player (each `AgentControlled` + `IsHuman`, not `StupidAi`).
    players: Vec<PlayerInfo>,
    /// Area entity → printed area id, for describing moves that reference areas.
    area_ids: HashMap<Entity, i32>,
    /// Current open trade offers.
    offers: Vec<OfferInfo>,
    /// Any player entity → faction name, for naming offer participants.
    player_factions: HashMap<Entity, String>,
}

impl Snapshot {
    /// Selects the player a request targets: by `faction` name if given, else the
    /// single player who currently has moves (the active one in a sequential phase).
    fn select(&self, faction: Option<&str>) -> Result<&PlayerInfo, Value> {
        if let Some(name) = faction {
            return self
                .players
                .iter()
                .find(|p| p.faction_str().eq_ignore_ascii_case(name))
                .ok_or_else(|| json!({ "ok": false, "error": format!("no agent player for faction '{name}'") }));
        }
        let mut with_moves = self.players.iter().filter(|p| !p.moves.is_empty());
        match (with_moves.next(), with_moves.next()) {
            (Some(p), None) => Ok(p),
            (Some(_), Some(_)) => Err(json!({ "ok": false, "error": "several players have moves; specify \"faction\"" })),
            _ => Err(json!({ "ok": false, "error": "no agent player currently has moves" })),
        }
    }
}

impl PlayerInfo {
    fn faction_str(&self) -> String {
        format!("{:?}", self.faction)
    }
}

/// A chosen move resolved to concrete parameters, ready to emit as a command.
#[derive(Debug, PartialEq, Eq)]
enum ResolvedMove {
    Expand { player: Entity, area: Entity, tokens: usize },
    MoveTokens { source: Entity, target: Entity, tokens: usize, player: Entity },
    ShipFerry { source: Entity, target: Entity, tokens: usize, player: Entity },
    EndMovement { player: Entity },
    BuildCity { player: Entity, area: Entity },
    EndCityConstruction { player: Entity },
    EliminateCity { player: Entity, city: Entity, area: Entity },
    AcquireCivCard { player: Entity, card: CivCardName },
    DoneAcquiringCivCards { player: Entity },
}

impl ResolvedMove {
    fn applied_json(&self, faction: &str) -> Value {
        let kind = match self {
            ResolvedMove::Expand { .. } => "PopulationExpansion",
            ResolvedMove::MoveTokens { .. } => "Movement",
            ResolvedMove::ShipFerry { .. } => "ShipFerry",
            ResolvedMove::EndMovement { .. } => "EndMovement",
            ResolvedMove::BuildCity { .. } => "CityConstruction",
            ResolvedMove::EndCityConstruction { .. } => "EndCityConstruction",
            ResolvedMove::EliminateCity { .. } => "EliminateCity",
            ResolvedMove::AcquireCivCard { .. } => "AcquireCard",
            ResolvedMove::DoneAcquiringCivCards { .. } => "DoneAcquiringCards",
        };
        json!({ "ok": true, "applied": kind, "faction": faction })
    }
}

fn json_header() -> Header {
    Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap()
}

type ControlledQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Name,
        &'static Faction,
        Option<&'static AvailableMoves>,
        &'static PlayerAreas,
        Has<CanTrade>,
        &'static PlayerTradeCards,
    ),
    With<AgentControlled>,
>;
type AreaQuery<'w, 's> = Query<'w, 's, (Entity, &'static GameArea, &'static Population)>;
type OfferQuery<'w, 's> = Query<'w, 's, (Entity, &'static mut OpenTradeOffer)>;
type FactionQuery<'w, 's> = Query<'w, 's, (Entity, &'static Faction)>;

/// Parses a `TradeCard` from its display name (case-insensitive).
fn parse_card(name: &str) -> Option<TradeCard> {
    TradeCard::iter().find(|c| format!("{c}").eq_ignore_ascii_case(name))
}

/// Parses a `{ "Ochre": 2, ... }` JSON object into a `TradeCard` count map.
/// Unknown card names are ignored.
fn parse_card_map(value: Option<&Value>) -> HashMap<TradeCard, usize> {
    let mut map = HashMap::default();
    if let Some(Value::Object(obj)) = value {
        for (name, count) in obj {
            if let (Some(card), Some(n)) = (parse_card(name), count.as_u64())
                && n > 0
            {
                *map.entry(card).or_insert(0) += n as usize;
            }
        }
    }
    map
}

/// Finds the player entity for a faction name (case-insensitive).
fn faction_entity(snapshot: &Snapshot, name: &str) -> Option<Entity> {
    snapshot
        .player_factions
        .iter()
        .find(|(_, f)| f.eq_ignore_ascii_case(name))
        .map(|(e, _)| *e)
}

/// Parses an offer id (accepts a JSON string or number) back into an `Entity`.
fn parse_offer_id(payload: &Value) -> Option<Entity> {
    let bits = payload
        .get("id")
        .and_then(|v| v.as_str().and_then(|s| s.parse::<u64>().ok()).or_else(|| v.as_u64()))?;
    Entity::try_from_bits(bits)
}

/// Polls the agent HTTP server once per frame and answers any pending requests.
#[allow(clippy::too_many_arguments)]
pub fn poll_agent_api(
    mut commands: Commands,
    server: Res<AgentServer>,
    activity: Option<Res<State<GameActivity>>>,
    controlled_query: ControlledQuery,
    area_query: AreaQuery,
    mut offer_query: OfferQuery,
    faction_query: FactionQuery,
    civ_data: Option<Res<AvailableCivCards>>,
    civ_cards_query: Query<(
        &PlayerTradeCards,
        &PlayerCivilizationCards,
        Option<&crate::civilization::resolve_calamities::resolve_calamities_components::GrainLockedForPurchase>,
        Option<&crate::civilization::CardsHeldBeforePurchasing>,
    )>,
    mut writers: MoveWriters,
) {
    let snapshot = build_snapshot(activity.as_ref(), &controlled_query, &area_query, &offer_query, &faction_query);

    while let Ok(Some(request)) = server.server.try_recv() {
        let mut request = request;
        let method = request.method().clone();
        let url = request.url().to_string();
        let path = url.split('?').next().unwrap_or("").to_string();
        let faction_q = query_param(&url, "faction");

        let body: Value = match (&method, path.as_str()) {
            (Method::Get, "/state") => state_json(&snapshot),
            (Method::Get, "/players") => players_json(&snapshot),
            (Method::Get, "/moves") => moves_json(&snapshot, faction_q.as_deref()),
            (Method::Get, "/trade") => trade_json(&snapshot, faction_q.as_deref()),
            (Method::Post, "/trade/stop") => {
                let payload = read_json_body(&mut request);
                let faction = payload.get("faction").and_then(|v| v.as_str()).map(str::to_string).or(faction_q);
                match snapshot.select(faction.as_deref()) {
                    Ok(p) => {
                        commands.entity(p.player).remove::<CanTrade>();
                        json!({ "ok": true, "stopped": p.faction_str() })
                    }
                    Err(e) => e,
                }
            }
            (Method::Post, "/trade/accept") => {
                let payload = read_json_body(&mut request);
                let faction = payload.get("faction").and_then(|v| v.as_str()).map(str::to_string).or(faction_q);
                match (snapshot.select(faction.as_deref()), parse_offer_id(&payload)) {
                    (Err(e), _) => e,
                    (Ok(_), None) => json!({ "ok": false, "error": "expected { \"id\": <offer id> }" }),
                    (Ok(p), Some(entity)) => {
                        let (player, name) = (p.player, p.name.clone());
                        match offer_query.get_mut(entity) {
                            Err(_) => json!({ "ok": false, "error": "no such offer" }),
                            Ok((_, mut offer)) => {
                                if offer.accept(player, name) {
                                    json!({ "ok": true, "accepted_offer": entity.to_bits().to_string(), "creator": offer.creator_name })
                                } else {
                                    json!({ "ok": false, "error": "cannot accept (own offer, already accepted/withdrawn, or not the target)" })
                                }
                            }
                        }
                    }
                }
            }
            (Method::Post, "/trade/offer") => {
                let payload = read_json_body(&mut request);
                let faction = payload.get("faction").and_then(|v| v.as_str()).map(str::to_string).or(faction_q.clone());
                match snapshot.select(faction.as_deref()) {
                    Err(e) => e,
                    Ok(p) => {
                        let target = payload.get("target").and_then(|v| v.as_str()).and_then(|n| faction_entity(&snapshot, n));
                        let target_name = target.and_then(|t| snapshot.player_factions.get(&t).cloned());
                        let mut offer = OpenTradeOffer::new(p.player, p.name.clone(), target, target_name);
                        offer.offering_guaranteed = parse_card_map(payload.get("offering_guaranteed"));
                        offer.offering_hidden_count = payload.get("offering_hidden").and_then(serde_json::Value::as_u64).unwrap_or(0) as usize;
                        offer.wanting_guaranteed = parse_card_map(payload.get("wanting_guaranteed"));
                        offer.wanting_hidden_count = payload.get("wanting_hidden").and_then(serde_json::Value::as_u64).unwrap_or(0) as usize;
                        if offer.is_valid() {
                            let id = commands.spawn(offer).id();
                            json!({ "ok": true, "created_offer": id.to_bits().to_string() })
                        } else {
                            json!({ "ok": false, "error": "invalid offer: need exactly 2 guaranteed cards and >=3 total (guaranteed+hidden) on each side" })
                        }
                    }
                }
            }
            (Method::Post, "/trade/settle") => {
                let payload = read_json_body(&mut request);
                let faction = payload.get("faction").and_then(|v| v.as_str()).map(str::to_string).or(faction_q);
                match (snapshot.select(faction.as_deref()), parse_offer_id(&payload)) {
                    (Err(e), _) => e,
                    (Ok(_), None) => json!({ "ok": false, "error": "expected { \"id\": <offer id>, \"cards\": {..} }" }),
                    (Ok(p), Some(entity)) => {
                        let player = p.player;
                        let cards = parse_card_map(payload.get("cards"));
                        match offer_query.get_mut(entity) {
                            Err(_) => json!({ "ok": false, "error": "no such offer" }),
                            Ok((_, mut offer)) => {
                                if offer.creator == player {
                                    offer.settle_creator(cards);
                                    json!({ "ok": true, "settled": "creator" })
                                } else if offer.accepted_by == Some(player) {
                                    offer.settle_acceptor(cards);
                                    json!({ "ok": true, "settled": "acceptor" })
                                } else {
                                    json!({ "ok": false, "error": "you are neither the creator nor the acceptor of this offer" })
                                }
                            }
                        }
                    }
                }
            }
            (Method::Post, "/move") => {
                let payload = read_json_body(&mut request);
                let faction = payload
                    .get("faction")
                    .and_then(|v| v.as_str())
                    .map(std::string::ToString::to_string)
                    .or(faction_q);
                match resolve_move(&snapshot, faction.as_deref(), payload) {
                    Err(e) => e,
                    Ok((resolved, faction)) => {
                        match &resolved {
                            ResolvedMove::Expand { player, area, tokens } => {
                                writers.expand.write(ExpandPopulationManuallyCommand::new(*player, *area, *tokens));
                            }
                            ResolvedMove::MoveTokens { source, target, tokens, player } => {
                                writers.move_tokens.write(MoveTokenFromAreaToAreaCommand::new(*source, *target, *tokens, *player));
                            }
                            ResolvedMove::ShipFerry { source, target, tokens, player } => {
                                writers.ferry.write(ShipFerryCommand::new(*source, *target, *tokens, *player));
                            }
                            ResolvedMove::EndMovement { player } => {
                                writers.end_move.write(PlayerMovementEnded::new(*player));
                            }
                            ResolvedMove::BuildCity { player, area } => {
                                writers.build_city.write(BuildCityCommand::new(*player, *area));
                            }
                            ResolvedMove::EndCityConstruction { player } => {
                                writers.end_city.write(EndPlayerCityConstruction::new(*player));
                            }
                            ResolvedMove::EliminateCity { player, city, area } => {
                                writers.eliminate_city.write(EliminateCity::new(*player, *city, *area, false));
                            }
                            ResolvedMove::AcquireCivCard { player, card } => {
                                if let (Some(cards_res), Ok((trade_cards, civ_cards, grain_locked, cards_held_before))) =
                                    (civ_data.as_deref(), civ_cards_query.get(*player))
                                {
                                    // Rule 31.53: see CardsHeldBeforePurchasing's doc comment.
                                    let credits = cards_res.total_credits(cards_held_before.map_or(&civ_cards.cards, |c| &c.0));
                                    if let Some(def) = cards_res.cards.iter().find(|c| c.name == *card) {
                                        let cost = def.calculate_cost(&credits) as usize;
                                        let payment = compute_ai_payment(trade_cards, cost, grain_locked.map_or(0, |l| l.0));
                                        writers.purchase.write(ConfirmCivCardPurchase {
                                            player: *player,
                                            cards_to_buy: vec![*card],
                                            payment,
                                        });
                                    }
                                }
                            }
                            ResolvedMove::DoneAcquiringCivCards { player } => {
                                writers.done_civ.write(PlayerDoneAcquiringCivilizationCards(*player));
                            }
                        }
                        resolved.applied_json(&faction)
                    }
                }
            }
            _ => json!({ "error": "unknown route", "routes": [
                "/state", "/players", "/moves?faction=", "POST /move {faction?,index,number?}",
                "/trade?faction=", "POST /trade/stop {faction?}",
                "POST /trade/accept {faction?,id}",
                "POST /trade/offer {faction?,offering_guaranteed,offering_hidden,wanting_guaranteed,wanting_hidden,target?}",
                "POST /trade/settle {faction?,id,cards}",
            ] }),
        };

        let response = Response::from_string(body.to_string()).with_header(json_header());
        let _ = request.respond(response);
    }
}

/// Extracts a query-string parameter value from a request URL.
fn query_param(url: &str, key: &str) -> Option<String> {
    let query = url.split('?').nth(1)?;
    query.split('&').find_map(|kv| {
        let (k, v) = kv.split_once('=')?;
        (k == key).then(|| v.to_string())
    })
}

fn build_snapshot(
    activity: Option<&Res<State<GameActivity>>>,
    controlled_query: &ControlledQuery,
    area_query: &AreaQuery,
    offer_query: &OfferQuery,
    faction_query: &FactionQuery,
) -> Snapshot {
    let phase = activity.map_or_else(|| "NotPlaying".to_string(), |a| format!("{:?}", a.get()));

    let area_ids: HashMap<Entity, i32> =
        area_query.iter().map(|(e, area, _)| (e, area.id)).collect();

    let player_factions: HashMap<Entity, String> = faction_query
        .iter()
        .map(|(e, f)| (e, format!("{:?}", f.faction)))
        .collect();

    let offers: Vec<OfferInfo> = offer_query
        .iter()
        .map(|(e, o)| OfferInfo {
            id: e.to_bits(),
            creator: o.creator,
            target: o.target,
            accepted_by: o.accepted_by,
            withdrawn: o.withdrawn,
            settling: o.is_settling(),
            offering_guaranteed: card_counts(&o.offering_guaranteed),
            offering_hidden: o.offering_hidden_count,
            wanting_guaranteed: card_counts(&o.wanting_guaranteed),
            wanting_hidden: o.wanting_hidden_count,
        })
        .collect();

    let players = controlled_query
        .iter()
        .map(|(player, name, faction, moves, player_areas, can_trade, trade_cards)| {
            let areas = player_areas
                .areas()
                .iter()
                .filter_map(|area| area_query.get(*area).ok())
                .map(|(_, area, pop)| AreaInfo {
                    area_id: area.id,
                    your_tokens: pop.population_for_player(player),
                    total: pop.total_population(),
                    max: pop.max_population,
                })
                .collect();
            let moves = moves
                .map(|m| m.moves.iter().map(|(i, gm)| (*i, gm.clone())).collect())
                .unwrap_or_default();
            let hand = card_counts(&trade_cards.commodity_cards());
            PlayerInfo {
                player,
                name: name.to_string(),
                faction: faction.faction,
                moves,
                areas,
                can_trade,
                hand,
            }
        })
        .collect();

    Snapshot { phase, players, area_ids, offers, player_factions }
}

/// Stable string-keyed counts from a `TradeCard` map (sorted for deterministic output).
fn card_counts(cards: &HashMap<TradeCard, usize>) -> Vec<(String, usize)> {
    let mut v: Vec<(String, usize)> = cards.iter().map(|(c, n)| (format!("{c}"), *n)).collect();
    v.sort_by(|a, b| a.0.cmp(&b.0));
    v
}

fn read_json_body(request: &mut tiny_http::Request) -> Value {
    let mut buf = String::new();
    if request.as_reader().read_to_string(&mut buf).is_ok() {
        serde_json::from_str(&buf).unwrap_or(Value::Null)
    } else {
        Value::Null
    }
}

fn player_summary(p: &PlayerInfo) -> Value {
    let areas: Vec<Value> = p
        .areas
        .iter()
        .map(|a| json!({ "area_id": a.area_id, "your_tokens": a.your_tokens, "total": a.total, "max": a.max }))
        .collect();
    json!({
        "faction": p.faction_str(),
        "name": p.name,
        "your_turn": !p.moves.is_empty(),
        "areas": areas,
    })
}

fn state_json(snapshot: &Snapshot) -> Value {
    json!({
        "phase": snapshot.phase,
        "controlled_players": snapshot.players.len(),
        "players": snapshot.players.iter().map(player_summary).collect::<Vec<_>>(),
    })
}

fn players_json(snapshot: &Snapshot) -> Value {
    json!({
        "phase": snapshot.phase,
        "players": snapshot.players.iter().map(|p| json!({
            "faction": p.faction_str(), "name": p.name, "your_turn": !p.moves.is_empty(),
        })).collect::<Vec<_>>(),
    })
}

fn moves_json(snapshot: &Snapshot, faction: Option<&str>) -> Value {
    let player = match snapshot.select(faction) {
        Ok(p) => p,
        Err(e) => return e,
    };
    let mut list: Vec<Value> = player
        .moves
        .iter()
        .map(|(index, game_move)| describe_move(*index, game_move, &snapshot.area_ids))
        .collect();
    list.sort_by_key(|v| v["index"].as_u64().unwrap_or(0));
    json!({ "faction": player.faction_str(), "your_turn": !list.is_empty(), "moves": list })
}

/// JSON object from `(card_name, count)` pairs.
fn map_json(pairs: &[(String, usize)]) -> Value {
    Value::Object(pairs.iter().map(|(k, v)| (k.clone(), json!(v))).collect())
}

fn trade_json(snapshot: &Snapshot, faction: Option<&str>) -> Value {
    let player = match snapshot.select(faction) {
        Ok(p) => p,
        Err(e) => return e,
    };
    let name = |e: Entity| snapshot.player_factions.get(&e).cloned();
    let offers: Vec<Value> = snapshot
        .offers
        .iter()
        .map(|o| {
            json!({
                "id": o.id.to_string(),
                "creator": name(o.creator),
                "target": o.target.and_then(name),
                "accepted_by": o.accepted_by.and_then(name),
                "settling": o.settling,
                "withdrawn": o.withdrawn,
                "offering_guaranteed": map_json(&o.offering_guaranteed),
                "offering_hidden": o.offering_hidden,
                "wanting_guaranteed": map_json(&o.wanting_guaranteed),
                "wanting_hidden": o.wanting_hidden,
                "can_accept": o.can_accept(player.player),
            })
        })
        .collect();
    json!({
        "faction": player.faction_str(),
        "phase": snapshot.phase,
        "can_trade": player.can_trade,
        "hand": map_json(&player.hand),
        "offers": offers,
    })
}

fn describe_move(index: usize, game_move: &GameMove, area_ids: &HashMap<Entity, i32>) -> Value {
    let aid = |e: Entity| area_ids.get(&e).copied();
    let mv = |index, kind, m: &MovementMove| {
        json!({ "index": index, "kind": kind, "from_area": aid(m.source), "to_area": aid(m.target), "max_tokens": m.max_tokens })
    };
    match game_move {
        GameMove::PopulationExpansion(m) => json!({
            "index": index, "kind": "PopulationExpansion", "area_id": aid(m.area), "max_tokens": m.max_tokens,
        }),
        GameMove::Movement(m) => mv(index, "Movement", m),
        GameMove::ShipFerry(m) => mv(index, "ShipFerry", m),
        GameMove::AttackArea(m) => mv(index, "AttackArea", m),
        GameMove::AttackCity(m) => mv(index, "AttackCity", m),
        GameMove::EndMovement => json!({ "index": index, "kind": "EndMovement" }),
        GameMove::CityConstruction(m) => json!({ "index": index, "kind": "CityConstruction", "area_id": aid(m.target) }),
        GameMove::EndCityConstruction => json!({ "index": index, "kind": "EndCityConstruction" }),
        GameMove::EliminateCity(m) => json!({
            "index": index, "kind": "EliminateCity", "area_id": aid(m.area),
            "tokens_gained": m.tokens_gained, "tokens_needed": m.tokens_needed,
        }),
        GameMove::AcquireCivilizationCards(AcquireCivilizationCardsMove::DoneAcquiringCards) =>
            json!({ "index": index, "kind": "DoneAcquiringCards" }),
        GameMove::AcquireCivilizationCards(AcquireCivilizationCardsMove::AcquireCard(card)) =>
            json!({ "index": index, "kind": "AcquireCard", "card": format!("{card:?}") }),
        other => json!({
            "index": index, "kind": "Other", "description": format!("{:?}", other),
            "note": "not yet applyable via the agent API (e.g. Trade via /trade, AcquireCards batch)",
        }),
    }
}

/// Resolves the move chosen by `index` for the selected player to a concrete
/// `ResolvedMove` + the player's faction name, or an error JSON. Pure.
fn resolve_move(
    snapshot: &Snapshot,
    faction: Option<&str>,
    payload: Value,
) -> Result<(ResolvedMove, String), Value> {
    let player_info = snapshot.select(faction)?;
    let Some(index) = payload.get("index").and_then(serde_json::Value::as_u64).map(|v| v as usize) else {
        return Err(json!({ "ok": false, "error": "expected JSON body { \"index\": <usize> }" }));
    };
    let number = payload.get("number").and_then(serde_json::Value::as_u64).map(|v| v as usize);

    let Some((_, game_move)) = player_info.moves.iter().find(|(i, _)| *i == index) else {
        return Err(json!({ "ok": false, "error": format!("no move with index {index}") }));
    };
    let player = player_info.player;
    let clamp = |m: &MovementMove| number.unwrap_or(m.max_tokens).min(m.max_tokens).max(1);

    let resolved = match game_move {
        GameMove::PopulationExpansion(m) => ResolvedMove::Expand {
            player, area: m.area, tokens: number.unwrap_or(m.max_tokens).min(m.max_tokens),
        },
        GameMove::Movement(m) | GameMove::AttackArea(m) | GameMove::AttackCity(m) =>
            ResolvedMove::MoveTokens { source: m.source, target: m.target, tokens: clamp(m), player },
        GameMove::ShipFerry(m) =>
            ResolvedMove::ShipFerry { source: m.source, target: m.target, tokens: clamp(m), player },
        GameMove::EndMovement => ResolvedMove::EndMovement { player },
        GameMove::CityConstruction(m) => ResolvedMove::BuildCity { player, area: m.target },
        GameMove::EndCityConstruction => ResolvedMove::EndCityConstruction { player },
        GameMove::EliminateCity(m) => ResolvedMove::EliminateCity { player, city: m.city, area: m.area },
        GameMove::AcquireCivilizationCards(AcquireCivilizationCardsMove::AcquireCard(card)) =>
            ResolvedMove::AcquireCivCard { player, card: *card },
        GameMove::AcquireCivilizationCards(AcquireCivilizationCardsMove::DoneAcquiringCards) =>
            ResolvedMove::DoneAcquiringCivCards { player },
        other => {
            return Err(json!({
                "ok": false,
                "error": "move kind not yet supported by the agent API",
                "kind": format!("{:?}", other),
            }));
        }
    };
    Ok((resolved, player_info.faction_str()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::Player;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::App;

    #[derive(Resource, Default)]
    struct ResolveResult(Option<ResolvedMove>);

    fn spawn_controlled(app: &mut App, faction: GameFaction, area_max: usize, move_max: usize) -> Entity {
        let area = app.world_mut().spawn((GameArea { id: 7 }, Population::new(area_max))).id();
        let mut moves = bevy::platform::collections::HashMap::default();
        moves.insert(1usize, GameMove::PopulationExpansion(PopExpMove::new(area, move_max)));
        let mut player_areas = PlayerAreas::default();
        player_areas.add_token_to_area(area, area);
        app.world_mut()
            .spawn((
                Player,
                AgentControlled,
                Name::new(format!("Ruler of {faction:?}")),
                Faction::new(faction),
                player_areas,
                AvailableMoves::new(moves),
                PlayerTradeCards::default(),
            ))
            .id()
    }

    fn run_resolve(
        activity: Option<Res<State<GameActivity>>>,
        controlled_query: ControlledQuery,
        area_query: AreaQuery,
        offer_query: OfferQuery,
        faction_query: FactionQuery,
        mut result: ResMut<ResolveResult>,
    ) {
        let snapshot = build_snapshot(activity.as_ref(), &controlled_query, &area_query, &offer_query, &faction_query);
        // Two players have moves → must select by faction.
        result.0 = resolve_move(&snapshot, Some("Egypt"), json!({ "index": 1, "number": 2 }))
            .ok()
            .map(|(rm, _)| rm);
    }

    #[test]
    fn resolves_move_for_named_faction_among_several() {
        let mut app = App::new();
        app.init_resource::<ResolveResult>();
        let _assyria = spawn_controlled(&mut app, GameFaction::Assyria, 9, 5);
        let egypt = spawn_controlled(&mut app, GameFaction::Egypt, 9, 5);

        app.world_mut().run_system_once(run_resolve).unwrap();

        let resolved = app.world().resource::<ResolveResult>().0.as_ref().unwrap();
        match resolved {
            ResolvedMove::Expand { player, tokens, .. } => {
                assert_eq!(*player, egypt, "selected the Egypt player");
                assert_eq!(*tokens, 2, "clamped requested token count");
            }
            other => panic!("expected Expand, got {other:?}"),
        }
    }

    #[test]
    fn parses_card_map_ignoring_unknown_and_zero() {
        let v = json!({ "Ochre": 2, "Bogus": 5, "Iron": 1, "Salt": 0 });
        let m = parse_card_map(Some(&v));
        assert_eq!(m.get(&TradeCard::Ochre), Some(&2));
        assert_eq!(m.get(&TradeCard::Iron), Some(&1));
        assert!(!m.contains_key(&TradeCard::Salt), "zero count skipped");
        assert_eq!(m.len(), 2, "unknown card name ignored");
    }

    #[test]
    fn offer_can_accept_follows_rules() {
        let mut world = bevy::prelude::World::new();
        let creator = world.spawn_empty().id();
        let other = world.spawn_empty().id();
        let third = world.spawn_empty().id();

        let mut offer = OfferInfo {
            id: 0,
            creator,
            target: None,
            accepted_by: None,
            withdrawn: false,
            settling: false,
            offering_guaranteed: vec![],
            offering_hidden: 0,
            wanting_guaranteed: vec![],
            wanting_hidden: 0,
        };
        assert!(offer.can_accept(other), "open offer: anyone but creator");
        assert!(!offer.can_accept(creator), "creator can't accept own offer");

        offer.target = Some(other);
        assert!(offer.can_accept(other), "directed offer: target can accept");
        assert!(!offer.can_accept(third), "directed offer: non-target cannot");

        offer.target = None;
        offer.accepted_by = Some(third);
        assert!(!offer.can_accept(other), "already accepted");
    }
}
