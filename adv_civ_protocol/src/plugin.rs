use crate::messages::*;
use bevy::prelude::*;
use lightyear::prelude::*;

/// Single ordered-reliable channel for all turn-based traffic. The game is
/// not latency-sensitive, so one channel keeps ordering trivial to reason
/// about; split channels only if a phase ever needs out-of-band traffic.
pub struct ControlChannel;

/// Registers all messages and channels. Must be added to both client and
/// server apps, after the lightyear `ClientPlugins`/`ServerPlugins`.
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // Client → Server
        app.register_message::<JoinGame>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<SubmitMove>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<ProposeTradeOffer>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<AcceptTradeOffer>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<SettleTradeOffer>()
            .add_direction(NetworkDirection::ClientToServer);

        // Server → Client
        app.register_message::<JoinAccepted>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<LobbyState>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<PhaseChanged>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<YourMoves>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<MoveRejected>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<GameStateView>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<YourHand>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<TradeOffersView>()
            .add_direction(NetworkDirection::ServerToClient);

        app.add_channel::<ControlChannel>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::Bidirectional);
    }
}
