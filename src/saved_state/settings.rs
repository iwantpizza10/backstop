use std::io::ErrorKind;
use std::{collections::HashSet, error::Error, fs, path::PathBuf};
use chrono::{DateTime, Utc};
use iced::time;
use serde::{Deserialize, Serialize};
use serde_binary::binary_stream::Endian;

use crate::constants;
use crate::discord_rpc::DiscordRpcMode;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BackstopSettings {
    shuffle: bool,
    repeat: bool,
    volume: f32,
    speed: f32,
    first_launch: bool,
    cache_last_updated: DateTime<Utc>,
    media_directories: HashSet<PathBuf>,
    rpc_mode: DiscordRpcMode,
    rpc_list: HashSet<String>,
}

impl BackstopSettings {
    /// loads settings from disk
    /// 
    /// returns:
    /// * `Ok(_)` if it loaded fine
    /// * `Err<serde_binary::Error>` if it cant deserialize
    /// * `Err<std::io::Error>` if io error
    pub fn load() -> Result<Self, Box<dyn Error>> {
        let mut path = constants::conf_dir();
        path.push("settings_temp.bin"); // todo: untemp

        let file = fs::read(path);

        match file {
            Ok(file) => {
                let mut settings = serde_binary::from_vec::<BackstopSettings>(file, Endian::Little)?;

                // todo: uncomment this
                // settings.first_launch = false;

                Ok(settings)
            },
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    Ok(Self::default())
                } else {
                    Err(Box::new(err))
                }
            }
        }
    }

    /// saves settings to disk
    /// 
    /// returns:
    /// * Ok(()) if it saved w/o issue
    /// * `Err<serde_binary::Error>` if it cant serialize
    /// * `Err<std::io::Error>` if io error
    pub async fn save(&mut self) -> Result<(), Box<dyn Error>> {
        let mut path = constants::conf_dir();
        path.push("settings_temp.bin"); // todo: untemp

        let serialized = serde_binary::to_vec(self, Endian::Little)?;
        fs::write(path, serialized)?;

        tokio::time::sleep(time::milliseconds(500)).await;

        Ok(())
    }

    // get/set functions

    /// gets the current shuffle state
    pub fn get_shuffle(&self) -> bool {
        self.shuffle
    }

    /// toggles the shuffle state, then returns the new state
    pub fn toggle_shuffle(&mut self) -> bool {
        self.shuffle = !self.shuffle;
        self.shuffle
    }

    /// gets the current repeat state
    pub fn get_repeat(&self) -> bool {
        self.repeat
    }

    /// toggles the repeat state, then returns the new state
    pub fn toggle_repeat(&mut self) -> bool {
        self.repeat = !self.repeat;
        self.repeat
    }

    /// returns the volume in **decibels**
    pub fn get_volume_db(&self) -> f32 {
        self.volume
    }

    /// returns the **linear** volume (e.g. multiplier typa thing)
    pub fn get_volume_linear(&self) -> f32 {
        10f32.powf(self.volume / 20.0)
    }

    /// sets the volume in **decibels**
    pub fn set_volume_db(&mut self, volume: f32) {
        self.volume = volume;
    }

    /// get the configued playback speed
    pub fn get_speed(&self) -> f32 {
        self.speed
    }

    /// set the configured playback speed (does NOT adjust
    /// the audio player, only the saved value in the
    /// settings)
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    /// gets whether this is the first time the app
    /// has been launched
    pub fn get_first_launch(&self) -> bool {
        self.first_launch
    }

    /// returns timestamp of when the media cache was last indexed
    pub fn get_cache_last_updated(&self) -> DateTime<Utc> {
        self.cache_last_updated
    }

    /// sets timestamp of when media cache was last indexed
    pub fn set_cache_last_updated(&mut self, time: DateTime<Utc>) {
        self.cache_last_updated = time;
    }

    /// returns a vector of media directories to index
    pub fn get_media_directories(&self) -> Vec<PathBuf> {
        self.media_directories.iter().map(PathBuf::clone).collect()
    }

    /// adds a media directory to the index list
    pub fn add_media_directory(&mut self, dir: PathBuf) {
        self.media_directories.insert(dir);
    }

    /// removed a media directory from the index list
    pub fn remove_media_directory(&mut self, dir: &PathBuf) {
        self.media_directories.remove(dir);
    }

    /// returns the current configured discord rich presence mode
    pub fn get_rpc_mode(&self) -> DiscordRpcMode {
        self.rpc_mode
    }

    /// sets the current discord rich presence mode
    pub fn set_rpc_mode(&mut self, mode: DiscordRpcMode) {
        self.rpc_mode = mode;
    }

    /// returns list of rpc (in|ex)clusisons
    pub fn get_rpc_list(&self) -> Vec<&String> {
        self.rpc_list.iter().collect()
    }

    /// adds to list of rpc (in|ex)clusisons
    pub fn add_rpc_list(&mut self, item: String) {
        self.rpc_list.insert(item.to_lowercase());
    }

    /// removes from list of rpc (in|ex)clusisons
    pub fn remove_rpc_list(&mut self, item: String) {
        self.rpc_list.remove(&item.to_lowercase());
    }
}

impl Default for BackstopSettings {
    fn default() -> Self {
        Self {
            shuffle: Default::default(),
            repeat: Default::default(),
            volume: -0.0,
            speed: 1.0,
            first_launch: true,
            cache_last_updated: Default::default(),
            media_directories: Default::default(),
            rpc_mode: Default::default(),
            rpc_list: Default::default(),
        }
    }
}
