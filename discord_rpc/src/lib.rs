mod types;

use chrono::{DateTime, Utc};
use discord_rich_presence::DiscordIpcClient;
use std::collections::HashSet;

pub use types::*;

pub struct DiscordRpc {
    mode: RpcMode,
    rpc_list: HashSet<String>,
    current_title: Option<String>,
    current_artist: Option<Vec<String>>,
    start_time: Option<DateTime<Utc>>,
    player_state: (), // todo
    playback_speed: f32,
    is_clear: bool,
    rpc_client: Option<DiscordIpcClient>,
}
