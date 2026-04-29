use std::collections::HashSet;
use std::fmt::Display;

use chrono::{DateTime, Utc};
use discord_rich_presence::activity::{Activity, ActivityType, Assets, Button, StatusDisplayType, Timestamps};
use discord_rich_presence::{DiscordIpc, DiscordIpcClient};
use discord_rich_presence::error::Error;
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::PlayingState;
use crate::constants::{BACKSTOP_LOGO_URL, BACKSTOP_PAUSE_ICON_URL, BACKSTOP_REPO_URL, DISCORD_APP_ID};
use crate::player::CurrentSong;
use crate::saved_state::settings::BackstopSettings;

#[derive(Clone, Default, Debug, Copy, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum DiscordRpcMode {
    #[default]
    Blacklist,
    Whitelist,
    Disabled,
}

impl Display for DiscordRpcMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            DiscordRpcMode::Blacklist => "Blacklist",
            DiscordRpcMode::Whitelist => "Whitelist",
            DiscordRpcMode::Disabled => "Disabled",
        })
    }
}

impl DiscordRpcMode {
    pub fn list_all() -> Vec<DiscordRpcMode> {
        vec![
            Self::Blacklist,
            Self::Whitelist,
            Self::Disabled,
        ]
    }
}

#[derive(Debug)]
pub struct DiscordRpc {
    rpc_mode_mirror: DiscordRpcMode,
    rpc_list_mirror: HashSet<String>,
    current_song_title: Option<String>,
    current_song_artist: Option<String>,
    song_start_time: Option<DateTime<Utc>>,
    playing_state: PlayingState,
    rpc_client: Option<DiscordIpcClient>,
}

impl DiscordRpc {
    /// creates a new DiscordRpc instance
    pub fn new(settings: &BackstopSettings, playing_state: PlayingState) -> Result<Self, Error> {
        Ok(Self {
            rpc_mode_mirror: settings.get_rpc_mode(),
            rpc_list_mirror: settings.get_rpc_list().iter().map(|x| (*x).clone()).collect(),
            current_song_title: None,
            current_song_artist: None,
            song_start_time: None,
            playing_state,
            rpc_client: if settings.get_rpc_mode() != DiscordRpcMode::Disabled {
                    let mut client = DiscordIpcClient::new(DISCORD_APP_ID);
                    client.connect()?;

                    Some(client)
                } else { None },
        })
    }

    /// updates the rpc_mode, handles connecting/disconnecting as needed
    pub fn update_rpc_mode(&mut self, new_mode: DiscordRpcMode) {
        if self.rpc_mode_mirror == DiscordRpcMode::Disabled && new_mode != DiscordRpcMode::Disabled {
            self.rpc_client = Some(DiscordIpcClient::new(DISCORD_APP_ID));
        } else if self.rpc_mode_mirror != DiscordRpcMode::Disabled && new_mode == DiscordRpcMode::Disabled {
            let _ = self.clear_rpc();
            self.rpc_client = None;
        } 

        self.rpc_mode_mirror = new_mode;
    }

    /// updates the rpc_list. does not append/remove, just overwrites
    pub fn update_rpc_list(&mut self, new_list: &HashSet<String>) {
        self.rpc_list_mirror = new_list.clone();
    }

    /// updates the playing state. will update the rpc data
    pub fn update_playing_state(&mut self, state: PlayingState) {
        self.playing_state = state;

        let _ = self.rpc();
    }

    /// sends new song info to the discord rpc
    pub fn play_song(&mut self, song: CurrentSong) {
        self.current_song_artist = Some(song.file_info.artist());
        self.current_song_title = Some(song.file_info.title());
        self.song_start_time = Some(song.start_time);
        self.playing_state = PlayingState::Playing;

        let _ = self.rpc();
    }

    /// clears discord rpc
    pub fn clear_rpc(&mut self) -> Result<(), Error> {
        self.current_song_artist = None;
        self.current_song_title = None;
        self.song_start_time = None;

        if let Some(client) = &mut self.rpc_client {
            client.clear_activity()?;
        }

        Ok(())
    }

    /// sets rpc based on instance data
    fn rpc(&mut self) -> Result<Option<()>, Error> {
        let allowed = self.can_show();

        if let Some(client) = &mut self.rpc_client && allowed {
            let mut image = Assets::new().large_image(BACKSTOP_LOGO_URL);
            let timestamp = Timestamps::new().start(self.song_start_time.unwrap_or_default().timestamp());
            let button = Button::new("Get Backstop", BACKSTOP_REPO_URL);

            if self.playing_state == PlayingState::Paused {
                image = image.small_image(BACKSTOP_PAUSE_ICON_URL);
                image = image.small_text("Paused");
            }

            let activity = Activity::new()
                .activity_type(ActivityType::Listening)
                .name("Backstop")
                .details(self.current_song_title.clone().unwrap_or_default())
                .state(self.current_song_artist.clone().unwrap_or_default())
                .status_display_type(StatusDisplayType::Details)
                .timestamps(timestamp)
                .buttons(vec![ button ])
                .assets(image);

            client.set_activity(activity)?;

            return Ok(Some(()))
        }

        Ok(None)
    }

    /// determines whether or not the current song is allowed to
    /// be shown according to rpc_list and rpc_mode rules
    fn can_show(&self) -> bool {
        let artist = self.current_song_artist.clone().unwrap_or_default().to_lowercase();
        let title = self.current_song_title.clone().unwrap_or_default().to_lowercase();
        let mut allowed;

        match self.rpc_mode_mirror {
            DiscordRpcMode::Blacklist => {
                allowed = true;

                for item in &self.rpc_list_mirror {
                    if artist.contains(item) || title.contains(item) {
                        allowed = false;
                    }
                }
            },
            DiscordRpcMode::Whitelist => {
                allowed = false;

                for item in &self.rpc_list_mirror {
                    if artist.contains(item) || title.contains(item) {
                        allowed = true;
                    }
                }
            },
            DiscordRpcMode::Disabled => {
                allowed = false;
            },
        }

        allowed
    }
}
