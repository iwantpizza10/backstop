use std::{fmt::Debug, fs::File, rc::Rc, sync::Arc, time::Duration};
use chrono::{DateTime, Utc};
use rodio::{Decoder, DeviceSinkBuilder, DeviceSinkError, MixerDeviceSink, Source};

use crate::{BackstopError, saved_state::song_file_info::SongFileInfo};

#[derive(Debug, Clone)]
pub struct CurrentSong {
    pub start_time: DateTime<Utc>,
    pub file_info: Arc<SongFileInfo>,
}

pub struct Player {
    pub audio_player: Rc<rodio::Player>,
    // unused because unused but kept to prevent it from
    // being dropped cause it'd stop playing sound
    _device: Rc<MixerDeviceSink>,
    current_duration: Option<Duration>,
}

impl Debug for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<player>")
    }
}

impl Player {
    /// constructs new `Player`, takes a `speed` and **LINEAR** `volume` for default values
    pub fn new(speed: f32, volume: f32) -> Result<Self, DeviceSinkError> {
        let mut device = DeviceSinkBuilder::open_default_sink()?;
        device.log_on_drop(false);

        let instance = Self {
            audio_player: Rc::new(rodio::Player::connect_new(&device.mixer())),
            _device: Rc::new(device),
            current_duration: None,
        };

        instance.set_speed(speed);
        instance.set_volume(volume);

        Ok(instance)
    }

    /// plays a song!
    pub fn play_song(&mut self, song: Arc<SongFileInfo>) -> Result<(), BackstopError> {
        let file = File::open(&song.path);

        let file = if let Ok(file) = file {
            file
        } else {
            return Err(BackstopError::PlaybackError);
        };

        let source = Decoder::try_from(file);

        let source = if let Ok(source) = source {
            source
        } else {
            return Err(BackstopError::PlaybackError);
        };

        self.current_duration = source.total_duration();
        self.audio_player.clear();
        self.audio_player.append(source);
        self.audio_player.play();

        Ok(())
    }

    /// gets position of the currently playing song
    pub fn get_pos(&self) -> Duration {
        self.audio_player.get_pos()
    }

    /// gets duration of current song
    pub fn get_duration(&self) -> Duration {
        self.current_duration.unwrap_or(Duration::ZERO)
    }

    pub fn song_done_or_empty(&self) -> bool {
        self.audio_player.empty()
    }

    /// pauses playing song
    pub fn pause(&mut self) {
        self.audio_player.pause();
    }
    
    /// resumes playing song
    pub fn resume(&mut self) {
        self.audio_player.play();
    }
    
    /// sets **LINEAR** volume of the player
    pub fn set_volume(&self, volume: f32) {
        self.audio_player.set_volume(volume);
    }
    
    /// sets speed of the player
    pub fn set_speed(&self, speed: f32) {
        self.audio_player.set_speed(speed);
    }

    pub fn clear(&self) {
        self.audio_player.clear();
    }
}
