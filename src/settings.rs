use std::{error::Error, fs, path::PathBuf};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_binary::binary_stream::Endian;
use crate::{cache::SortType, constants};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum RichPresenceType {
    Blacklist,
    Whitelist,
    Disabled
}

#[derive(Serialize, Deserialize)]
pub struct BackstopSettings {
    has_launched: bool,
    volume: f32,
    playback_speed: f32,
    cache_last_updated: DateTime<Utc>,
    media_directories: Vec<PathBuf>,
    sort_type: SortType,
    discord_rich_presence_type: RichPresenceType,
    rich_presence_list: Vec<String>
}

impl BackstopSettings {
    pub fn new() -> Self {
        BackstopSettings {
            has_launched: false,
            volume: -0.0,
            playback_speed: 1.0,
            cache_last_updated: Utc::now(),
            media_directories: vec![],
            sort_type: SortType::ArtistAlphabetical,
            discord_rich_presence_type: RichPresenceType::Blacklist,
            rich_presence_list: vec![]
        }
    }

    /// attempts to load backstop config, returns `BackstopConfig::new()` if it doesn't work
    pub fn load_else_new() -> Self {
        let mut path = constants::conf_dir();
        path.push("settings.bin");

        let file = fs::read(path);

        if let Ok(file) = file {
            if let Ok(settings) = serde_binary::from_vec::<BackstopSettings>(file, Endian::Little) {
                return settings;
            }
        }

        Self::new()
    }

    /// saves backstop config,
    /// 
    /// returns a:
    /// * `Err<serde_binary::Error>` if it cant serialize
    /// * `Err<std::io::Error>` if there's an error writing to the file
    pub fn save_to_disk(&self) -> Result<(), Box<dyn Error>> {
        let mut path = constants::conf_dir();
        path.push("settings.bin");

        let serialized_settings: Vec<u8> = serde_binary::to_vec(self, Endian::Little)?;
        fs::write(path, serialized_settings)?;

        Ok(())
    }

    pub fn is_first_launch(&self) -> bool {
        !self.has_launched
    }

    pub fn set_is_first_launch(&mut self, first_launch: bool) {
        self.has_launched = !first_launch;
    }

    /// returns the volume as a multiplier
    /// 
    /// `linear volume` = pow(10, `dB` / 20)
    pub fn volume_linear(&self) -> f32 {
        10f32.powf(self.volume / 20.0)
    }

    /// returns the volume in decibels
    pub fn volume(&self) -> f32 {
        self.volume
    }

    /// sets the volume (DECIBELS!)
    /// 
    /// `db` = 20 * log10(`linear volume`)
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }

    pub fn playback_speed(&self) -> f32 {
        self.playback_speed
    }

    pub fn set_playback_speed(&mut self, new_speed: f32) {
        self.playback_speed = new_speed;
    }

    pub fn cache_last_updated(&self) -> DateTime<Utc> {
        self.cache_last_updated
    }

    pub fn media_directories(&self) -> &Vec<PathBuf> {
        &self.media_directories
    }

    pub fn add_media_directory(&mut self, directory: PathBuf) {
        self.media_directories.push(directory);
    }

    pub fn remove_media_directory(&mut self, directory: PathBuf) {
        self.media_directories = self.media_directories.iter_mut()
            .filter(|x| **x != directory)
            .map(|x| x.clone())
            .collect();
    }

    pub fn sort_type(&self) -> SortType {
        self.sort_type.clone()
    }

    pub fn set_sort_type(&mut self, sort: SortType) {
        self.sort_type = sort;
    }

    pub fn set_rich_presence_type(&mut self, presence: RichPresenceType) {
        self.discord_rich_presence_type = presence;
    }

    pub fn rich_presence_type(&self) -> &RichPresenceType {
        &self.discord_rich_presence_type
    }

    pub fn add_rich_presence_list(&mut self, text: String) {
        self.rich_presence_list.push(text);
    }

    pub fn remove_rich_presence_list(&mut self, text: String) {
        self.rich_presence_list = self.rich_presence_list.iter()
            .filter(|x| **x != text)
            .map(|x| x.clone())
            .collect();
    }

    pub fn rich_presence_list(&self) -> &Vec<String> {
        &self.rich_presence_list
    }
}
