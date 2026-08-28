use crate::agent_api::agent_api_systems::{AgentServer, PendingWaits, poll_agent_api};
use bevy::prelude::*;
use tiny_http::Server;

/// Address the embedded agent HTTP server binds to.
pub const AGENT_API_ADDR: &str = "127.0.0.1:7878";

/// Embeds a tiny local HTTP server that lets an external agent query game state
/// and submit moves. See `docs/agent-api-design.md`.
pub struct AgentApiPlugin;

impl Plugin for AgentApiPlugin {
    fn build(&self, app: &mut App) {
        match Server::http(AGENT_API_ADDR) {
            Ok(server) => {
                info!("[agent-api] listening on http://{AGENT_API_ADDR}");
                // Always-on so an agent can poll /state to learn when a game starts;
                // handlers simply report no human player when not in a game.
                app.insert_resource(AgentServer { server })
                    .init_non_send::<PendingWaits>()
                    .add_systems(Update, poll_agent_api);
            }
            Err(e) => {
                warn!("[agent-api] disabled — could not bind {AGENT_API_ADDR}: {e}");
            }
        }
    }
}
