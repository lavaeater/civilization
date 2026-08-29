//! Minimal interactive CLI client for the multiplayer spike.
//!
//! Connects to the local spike server over plain WebSocket, claims a seat,
//! prints the board state and your available moves, and submits the move
//! index you type. Usage:
//!
//! ```sh
//! cargo run -p adv_civ_server --bin spike_client -- Tommie
//! # then type e.g. `0` (move index) or `0 2` (move index + token count)
//! ```
//!
//! Trade commands (see docs/multiplayer.md's "Trade over the network"),
//! usable any time two of these clients are connected during the Trade
//! phase, independent of `YourMoves`:
//!
//! ```text
//! offer <give_card> <give_hidden> <want_card> <want_hidden> [target_faction]
//! accept <offer_id>
//! settle <offer_id>   # commits your entire current hand (guaranteed + hidden)
//! ```

use bevy::app::ScheduleRunnerPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use core::time::Duration;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender};

use adv_civ_protocol::*;
use lightyear::netcode::Key;
use lightyear::prelude::client::*;
use lightyear::prelude::*;

const TICK_HZ: f64 = 32.0;
const SERVER_PORT: u16 = 5111;
const SERVER_ADDR: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), SERVER_PORT);

#[derive(Resource)]
struct PlayerName(String);

#[derive(Resource)]
struct StdinLines(Mutex<Receiver<String>>);

/// `--auto`: play unattended — prefer the phase-ending move when one is
/// offered, otherwise take the first. Exercises the protocol end to end.
#[derive(Resource)]
struct AutoPlay(bool);

/// Base64 ConnectToken from the HTTP join endpoint (`--token <b64>`).
/// Without it, falls back to manual zero-key dev auth + a JoinGame message.
#[derive(Resource, Default)]
struct JoinToken(Option<String>);

/// This client's own trade-card hand, kept in sync from `YourHand` so
/// `settle <offer_id>` can commit it without the user retyping their cards.
#[derive(Resource, Default)]
struct MyHand(Vec<(TradeCard, usize)>);

/// Tracks the current phase from `PhaseChanged` so `--auto` can stay hands-off
/// during Trade (auto-picking `StopTrading` the instant it's offered would
/// make manual trade-command testing impossible).
#[derive(Resource, Default)]
struct CurrentPhase(Option<NetPhase>);

fn main() {
    let name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "Anonymous".into());
    let auto = std::env::args().any(|a| a == "--auto");
    let token = {
        let args: Vec<String> = std::env::args().collect();
        args.iter()
            .position(|a| a == "--token")
            .and_then(|i| args.get(i + 1).cloned())
    };

    let (tx, rx): (Sender<String>, Receiver<String>) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut line = String::new();
        loop {
            line.clear();
            match stdin.read_line(&mut line) {
                // 0 bytes read = EOF; without this the loop spins forever
                // flooding the channel with empty strings.
                Ok(0) | Err(_) => break,
                Ok(_) => {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && tx.send(trimmed.to_string()).is_err() {
                        break;
                    }
                }
            }
        }
    });

    let mut app = App::new();
    app.add_plugins(
        MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        ))),
    )
    .add_plugins(LogPlugin {
        filter: "warn,spike_client=info".into(),
        ..default()
    });

    app.add_plugins(ClientPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / TICK_HZ),
    });
    app.add_plugins(ProtocolPlugin);

    app.insert_resource(PlayerName(name));
    app.insert_resource(StdinLines(Mutex::new(rx)));
    app.insert_resource(AutoPlay(auto));
    app.insert_resource(JoinToken(token));
    app.init_resource::<MyHand>();
    app.init_resource::<CurrentPhase>();

    app.add_systems(Startup, connect);
    app.add_systems(
        Update,
        (join_when_connected, receive_messages, submit_typed_input),
    );

    app.run();
}

fn connect(mut commands: Commands, token: Res<JoinToken>) -> Result {
    let auth = if let Some(token_b64) = &token.0 {
        // Real flow: short-lived ConnectToken from the HTTP join endpoint.
        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(token_b64)
            .map_err(|e| format!("--token is not valid base64: {e}"))?;
        let token = lightyear::netcode::ConnectToken::try_from_bytes(&bytes)
            .map_err(|e| format!("--token is not a valid ConnectToken: {e}"))?;
        Authentication::Token(token)
    } else {
        // Dev-only manual auth: zero key, random client id. Works against a
        // server running with the default dev key.
        let client_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        Authentication::Manual {
            server_addr: SERVER_ADDR,
            client_id,
            private_key: Key::default(),
            protocol_id: PROTOCOL_ID,
        }
    };
    let config = ClientConfig::builder().with_no_encryption();
    let client = commands
        .spawn((
            Client,
            PeerAddr(SERVER_ADDR),
            Link::default(),
            NetcodeClient::new(auth, NetcodeConfig::default())?,
            WebSocketClientIo::from_url(config, format!("ws://127.0.0.1:{SERVER_PORT}")),
        ))
        .id();
    commands.trigger(Connect { entity: client });
    println!("Connecting to {SERVER_ADDR} …");
    Ok(())
}

/// Claim a seat as soon as the connection handshake completes.
fn join_when_connected(
    connected: Query<&mut MessageSender<JoinGame>, Added<Connected>>,
    name: Res<PlayerName>,
) {
    for mut sender in connected {
        println!("Connected. Joining as {} …", name.0);
        sender.send::<ControlChannel>(JoinGame {
            player_name: name.0.clone(),
            reconnect_token: None,
        });
    }
}

fn receive_messages(
    mut accepted: Query<&mut MessageReceiver<JoinAccepted>>,
    mut lobby: Query<&mut MessageReceiver<LobbyState>>,
    mut phases: Query<&mut MessageReceiver<PhaseChanged>>,
    mut moves: Query<&mut MessageReceiver<YourMoves>>,
    mut rejected: Query<&mut MessageReceiver<MoveRejected>>,
    mut state: Query<&mut MessageReceiver<GameStateView>>,
    mut hands: Query<&mut MessageReceiver<YourHand>>,
    mut trade_offers: Query<&mut MessageReceiver<TradeOffersView>>,
    mut submit: Query<&mut MessageSender<SubmitMove>>,
    auto_play: Res<AutoPlay>,
    mut my_hand: ResMut<MyHand>,
    mut current_phase: ResMut<CurrentPhase>,
) {
    for mut receiver in hands.iter_mut() {
        for msg in receiver.receive() {
            let cards: Vec<String> = msg
                .cards
                .iter()
                .map(|(card, count)| format!("{card} ×{count}"))
                .collect();
            println!(
                "Your hand: {}",
                if cards.is_empty() {
                    "(empty)".into()
                } else {
                    cards.join(", ")
                }
            );
            my_hand.0 = msg.cards;
        }
    }
    for mut receiver in trade_offers.iter_mut() {
        for msg in receiver.receive() {
            if msg.offers.is_empty() {
                println!("Trade offers: (none open)");
                continue;
            }
            println!("Trade offers:");
            for offer in &msg.offers {
                let offering = describe_offer_side(&offer.offering_guaranteed, offer.offering_hidden_count);
                let wanting = describe_offer_side(&offer.wanting_guaranteed, offer.wanting_hidden_count);
                let target = offer.target.map_or("anyone".to_string(), |f| f.to_string());
                let accepted = offer
                    .accepted_by
                    .map_or(String::new(), |a| format!(" — accepted by {a}"));
                let settling = if offer.settling { " [settling]" } else { "" };
                println!(
                    "  #{} {} offers {} for {} (to {}){}{}",
                    offer.id.0, offer.creator, offering, wanting, target, accepted, settling
                );
            }
        }
    }
    for mut receiver in accepted.iter_mut() {
        for msg in receiver.receive() {
            println!("✓ Seated as {} ({})", msg.player_name, msg.faction);
        }
    }
    for mut receiver in lobby.iter_mut() {
        for msg in receiver.receive() {
            println!("Lobby ({} seats):", msg.seats_total);
            for p in &msg.players {
                let status = if p.connected { "joined" } else { "open" };
                println!("  {} — {} [{}]", p.faction, p.name, status);
            }
        }
    }
    for mut receiver in phases.iter_mut() {
        for msg in receiver.receive() {
            println!("— Phase: {:?} —", msg.phase);
            current_phase.0 = Some(msg.phase);
        }
    }
    for mut receiver in state.iter_mut() {
        for msg in receiver.receive() {
            println!("Board:");
            for area in &msg.areas {
                let pops: Vec<String> = area
                    .population
                    .iter()
                    .map(|(f, n)| format!("{f}:{n}"))
                    .collect();
                println!(
                    "  [{}] {} (cap {}) {}",
                    area.area.0,
                    area.name,
                    area.max_population,
                    pops.join(" ")
                );
            }
            for player in &msg.players {
                println!(
                    "  {} ({}) — {} tokens in stock",
                    player.name, player.faction, player.tokens_in_stock
                );
            }
        }
    }
    for mut receiver in moves.iter_mut() {
        for msg in receiver.receive() {
            if msg.moves.is_empty() {
                println!("No moves available — waiting.");
                continue;
            }
            println!("Your moves (type an index, optionally `<index> <tokens>`):");
            for (index, game_move) in &msg.moves {
                println!("  {index}: {}", describe_move(game_move));
            }
            if auto_play.0 && current_phase.0 == Some(NetPhase::Trade) {
                println!(
                    "⚙ auto-play pauses during Trade — use offer/accept/settle, \
                     then submit the StopTrading move index manually when done."
                );
            } else if auto_play.0 {
                // Build cities when possible (cities draw trade cards →
                // exercises the hidden-info path), otherwise wrap the phase
                // up, otherwise take whatever is first.
                let pick = msg
                    .moves
                    .iter()
                    .find(|(_, m)| matches!(m, NetGameMove::BuildCity { .. }))
                    .or_else(|| msg.moves.iter().find(|(_, m)| is_phase_ending(m)))
                    .or_else(|| msg.moves.first())
                    .map(|(i, _)| *i);
                if let Some(index) = pick {
                    println!("⚙ auto-picking move {index}");
                    for mut sender in submit.iter_mut() {
                        sender.send::<ControlChannel>(SubmitMove::index(index));
                    }
                }
            }
        }
    }
    for mut receiver in rejected.iter_mut() {
        for msg in receiver.receive() {
            println!("✗ Move {} rejected: {}", msg.move_index, msg.reason);
        }
    }
}

/// Moves that finish the player's involvement in the current phase.
fn is_phase_ending(game_move: &NetGameMove) -> bool {
    matches!(
        game_move,
        NetGameMove::EndMovement
            | NetGameMove::EndCityConstruction
            | NetGameMove::DoneAcquiringCivCards
            | NetGameMove::Trade(NetTradeMove::StopTrading)
    )
}

fn describe_move(game_move: &NetGameMove) -> String {
    match game_move {
        NetGameMove::PopulationExpansion { area, max_tokens } => {
            format!("expand {area} with up to {max_tokens} token(s)")
        }
        NetGameMove::Movement {
            source,
            target,
            max_tokens,
        } => format!("move up to {max_tokens} token(s) {source} → {target}"),
        NetGameMove::ShipFerry {
            source,
            target,
            max_tokens,
        } => format!("ferry up to {max_tokens} token(s) {source} → {target}"),
        NetGameMove::AttackArea {
            source,
            target,
            max_tokens,
        } => format!("attack {target} from {source} (up to {max_tokens})"),
        NetGameMove::AttackCity {
            source,
            target,
            max_tokens,
        } => format!("attack city in {target} from {source} (up to {max_tokens})"),
        NetGameMove::EndMovement => "end movement".into(),
        NetGameMove::BuildCity { area } => format!("build city in {area}"),
        NetGameMove::EndCityConstruction => "end city construction".into(),
        NetGameMove::EliminateCity {
            area,
            tokens_gained,
            tokens_needed,
        } => format!("eliminate city in {area} (gain {tokens_gained}, need {tokens_needed})"),
        NetGameMove::Trade(trade) => match trade {
            NetTradeMove::ProposeTrade { to, matching_cards } => {
                format!("propose trade with {to} (matching: {matching_cards:?})")
            }
            NetTradeMove::AcceptOrDeclineTrade { .. } => "accept/decline trade offer".into(),
            NetTradeMove::AutoDeclineTrade { .. } => "auto-decline trade offer".into(),
            NetTradeMove::StopTrading => "stop trading".into(),
            NetTradeMove::SettleTrade { .. } => "settle trade".into(),
        },
        NetGameMove::AcquireCivCards { cards } => {
            let names: Vec<String> = cards.iter().map(|c| c.to_string()).collect();
            format!("buy civilization card(s): {}", names.join(", "))
        }
        NetGameMove::DoneAcquiringCivCards => "done buying civilization cards".into(),
    }
}

/// "2 Wine, 1 Grain + 3 hidden" style summary of one side of a trade offer.
fn describe_offer_side(guaranteed: &[(TradeCard, usize)], hidden_count: usize) -> String {
    let mut parts: Vec<String> = guaranteed
        .iter()
        .map(|(card, count)| format!("{count} {card}"))
        .collect();
    if hidden_count > 0 {
        parts.push(format!("{hidden_count} hidden"));
    }
    if parts.is_empty() {
        "nothing".to_string()
    } else {
        parts.join(", ")
    }
}

/// Parses a "Card" or "Card:count" token into `(TradeCard, count)`, count
/// defaulting to 1. Case-insensitive on the card name.
fn parse_card_token(token: &str) -> Option<(TradeCard, usize)> {
    let (name, count) = token.split_once(':').unwrap_or((token, "1"));
    let count: usize = count.parse().ok()?;
    let card = COMMODITY_CARDS.iter().find(|c| c.to_string().eq_ignore_ascii_case(name))?;
    Some((*card, count))
}

fn parse_faction(name: &str) -> Option<GameFaction> {
    [
        GameFaction::Egypt,
        GameFaction::Crete,
        GameFaction::Africa,
        GameFaction::Asia,
        GameFaction::Assyria,
        GameFaction::Babylon,
        GameFaction::Illyria,
        GameFaction::Iberia,
        GameFaction::Thrace,
    ]
    .into_iter()
    .find(|f| f.to_string().eq_ignore_ascii_case(name))
}

const COMMODITY_CARDS: [TradeCard; 18] = [
    TradeCard::Ochre,
    TradeCard::Hides,
    TradeCard::Iron,
    TradeCard::Papyrus,
    TradeCard::Salt,
    TradeCard::Timber,
    TradeCard::Grain,
    TradeCard::Oil,
    TradeCard::Cloth,
    TradeCard::Wine,
    TradeCard::Bronze,
    TradeCard::Silver,
    TradeCard::Spices,
    TradeCard::Resin,
    TradeCard::Gems,
    TradeCard::Dye,
    TradeCard::Gold,
    TradeCard::Ivory,
];

/// Parses stdin lines into either a `SubmitMove` (`<index>` or
/// `<index> <tokens>`) or a trade command (`offer`/`accept`/`settle` — see
/// the module doc comment for exact syntax).
#[allow(clippy::too_many_arguments)]
fn submit_typed_input(
    stdin: Res<StdinLines>,
    mut move_senders: Query<&mut MessageSender<SubmitMove>>,
    mut propose_senders: Query<&mut MessageSender<ProposeTradeOffer>>,
    mut accept_senders: Query<&mut MessageSender<AcceptTradeOffer>>,
    mut settle_senders: Query<&mut MessageSender<SettleTradeOffer>>,
    my_hand: Res<MyHand>,
) {
    let Ok(lines) = stdin.0.lock() else { return };
    while let Ok(line) = lines.try_recv() {
        let mut parts = line.split_whitespace();
        let Some(first) = parts.next() else { continue };

        match first {
            "offer" => {
                let (Some(give), Some(give_hidden), Some(want), Some(want_hidden)) = (
                    parts.next().and_then(parse_card_token),
                    parts.next().and_then(|t| t.parse::<usize>().ok()),
                    parts.next().and_then(parse_card_token),
                    parts.next().and_then(|t| t.parse::<usize>().ok()),
                ) else {
                    println!(
                        "Usage: offer <give_card[:count]> <give_hidden> <want_card[:count]> <want_hidden> [target_faction]"
                    );
                    continue;
                };
                let target = parts.next().and_then(parse_faction);
                for mut sender in propose_senders.iter_mut() {
                    sender.send::<ControlChannel>(ProposeTradeOffer {
                        offering_guaranteed: vec![give],
                        offering_hidden_count: give_hidden,
                        wanting_guaranteed: vec![want],
                        wanting_hidden_count: want_hidden,
                        target,
                    });
                }
                println!("→ proposed offer");
            }
            "accept" => {
                let Some(Ok(id)) = parts.next().map(str::parse::<u64>) else {
                    println!("Usage: accept <offer_id>");
                    continue;
                };
                for mut sender in accept_senders.iter_mut() {
                    sender.send::<ControlChannel>(AcceptTradeOffer {
                        offer: NetOfferId(id),
                    });
                }
                println!("→ accepted offer #{id}");
            }
            "settle" => {
                let Some(Ok(id)) = parts.next().map(str::parse::<u64>) else {
                    println!("Usage: settle <offer_id>  (commits your whole current hand)");
                    continue;
                };
                for mut sender in settle_senders.iter_mut() {
                    sender.send::<ControlChannel>(SettleTradeOffer {
                        offer: NetOfferId(id),
                        cards: my_hand.0.clone(),
                    });
                }
                println!("→ settled offer #{id} with your current hand");
            }
            _ => {
                let Ok(move_index) = first.parse::<usize>() else {
                    println!(
                        "Could not parse {line:?} — type a move index (e.g. `0` or `0 2`), \
                         or a trade command (offer/accept/settle)."
                    );
                    continue;
                };
                let tokens = parts.next().and_then(|t| t.parse::<usize>().ok());
                for mut sender in move_senders.iter_mut() {
                    sender.send::<ControlChannel>(SubmitMove {
                        tokens,
                        ..SubmitMove::index(move_index)
                    });
                    println!("→ submitted move {move_index}");
                }
            }
        }
    }
}
