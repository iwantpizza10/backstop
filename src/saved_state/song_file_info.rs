use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::softunwrap_str;

#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub struct SongFileInfo {
    pub path: PathBuf,
    title: Option<String>,
    artist: Option<String>,
    guessed_title: Option<String>,
    guessed_artist: Option<String>,
    pub album_artist: Option<String>,
    pub album: Option<String>,
    pub track_number: Option<u16>,
    pub year: Option<i32>,
    pub cover: Option<PathBuf>,
}

impl SongFileInfo {
    pub fn new(path: PathBuf) -> Self {
        let mut instance = Self {
            path,
            title: None,
            artist: None,
            guessed_title: None,
            guessed_artist: None,
            album_artist: None,
            album: None,
            track_number: None,
            year: None,
            cover: None,
        };

        instance.guess_fields();

        instance
    }

    fn guess_fields(&mut self) {
        let name = self.path.file_name().map_or(None, |x| x.to_str());
        let extension = self.path.extension().map_or(None, |x| x.to_str());
        let extension = extension.map_or(None, |x| Some(format!(".{}", x)));
        let extension = extension.unwrap_or_default();

        if let Some(name) = name {
            let mut iter = name.split(" - ");
            self.guessed_artist = softunwrap_str!(iter.next());
            self.guessed_title = softunwrap_str!(iter.next()).map_or(None, |x| Some(x.replace(&extension, "")));
        }
    }

    // setter fields

    pub fn set_title(mut self, title: Option<String>) -> Self {
        self.title = title;

        self
    }

    pub fn set_artist(mut self, artist: Option<String>) -> Self {
        self.artist = artist;

        self
    }

    pub fn set_album_artist(mut self, album_artist: Option<String>) -> Self {
        self.album_artist = album_artist;

        self
    }

    pub fn set_album(mut self, album: Option<String>) -> Self {
        self.album = album;

        self
    }

    pub fn set_track_number(mut self, track_number: Option<u16>) -> Self {
        self.track_number = track_number;

        self
    }

    pub fn set_year(mut self, year: Option<i32>) -> Self {
        self.year = year;

        self
    }

    pub fn set_cover(mut self, cover: Option<PathBuf>) -> Self {
        self.cover = cover;

        self
    }

    // getter fields
    // all the rest of them can just be read directly

    pub fn artist(&self) -> String {
        if let Some(artist) = &self.artist {
            return artist.clone();
        } else if let Some(guessed_artist) = &self.guessed_artist {
            return guessed_artist.clone();
        } else {
            return "Unknown Artist".to_string();
        }
    }

    pub fn title(&self) -> String {
        if let Some(title) = &self.title {
            return title.clone();
        } else if let Some(guessed_title) = &self.guessed_title {
            return guessed_title.clone();
        } else {
            return "Unknown Title".to_string();
        }
    }
}
