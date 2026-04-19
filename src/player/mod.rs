use std::{fs::File, rc::Rc, time::Duration};
use chrono::{DateTime, Utc};
use rodio::{Decoder, DeviceSinkBuilder, DeviceSinkError};

use crate::{BackstopError, saved_state::song_file_info::SongFileInfo};

pub struct CurrentSong {
    pub duration: Duration,
    pub start_time: DateTime<Utc>,
    pub file_info: Rc<SongFileInfo>,
}

pub struct Player {
    pub audio_player: Rc<rodio::Player>,
}

impl Player {
    /// constructs new `Player`, takes a `speed` and **LINEAR** `volume` for default values
    pub fn new(speed: f32, volume: f32) -> Result<Self, DeviceSinkError> {
        let mut device = DeviceSinkBuilder::open_default_sink()?;
        device.log_on_drop(false);

        let instance = Self {
            audio_player: Rc::new(rodio::Player::connect_new(&device.mixer())),
        };

        instance.set_speed(speed);
        instance.set_volume(volume);

        Ok(instance)
    }

    /// plays a song!
    pub fn play_song(&mut self, song: Rc<SongFileInfo>) -> Result<(), BackstopError> {
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

        self.audio_player.clear();
        self.audio_player.append(source);
        self.audio_player.play();

        Ok(())
    }

    /// gets position of the currently playing song
    pub fn get_pos(&self) -> Duration {
        self.audio_player.get_pos()
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
}
