use std::{error::Error, ffi::OsStr, fs::{self, read_dir}, io, path::{Path, PathBuf}, time::Duration};
use lofty::{error::LoftyError, prelude::*, tag::Tag};
use lofty::probe;
use serde::{Deserialize, Serialize};
use serde_binary::binary_stream::Endian;

use crate::constants;

#[derive(Debug, PartialEq)]
pub enum CacheState {
    /// cache loaded from disk just fine
    Healthy,
    /// cache files not present on disk
    Dead
}

#[derive(Debug)]
pub struct MediaCache {
    songs: Vec<SongFileInfo>,
    state: CacheState
}

impl MediaCache {
    pub fn from_vec(vec: Vec<SongFileInfo>) -> Self {
        MediaCache {
            songs: vec,
            state: CacheState::Healthy
        }
    }

    pub fn dead() -> Self {
        MediaCache {
            songs: vec![],
            state: CacheState::Dead
        }
    }

    /// attempts to load media cache from disk
    /// 
    /// if the file is locked/busy, it returns `Err<io::Error>` with kind `io::ErrorKind::ResourceBusy`
    /// 
    /// if the file read operation fails for any other reason, it simply returns `None`
    pub fn load_from_disk() -> Result<Option<Self>, io::Error> {
        let mut path = constants::conf_dir();
        path.push("media_cache.bin");

        let file = fs::read(path);

        match file {
            Err(err) => {
                if err.kind() == io::ErrorKind::ResourceBusy {
                    return Err(err);
                } else {
                    return Ok(None);
                }
            },
            Ok(file) => {
                if let Ok(cache) = serde_binary::from_vec::<Vec<SongFileInfo>>(file, Endian::Little) {
                    return Ok(Some(MediaCache::from_vec(cache)));
                }

                return Ok(None);
            }
        }
    }

    /// saves media cache to disk
    /// 
    /// returns a:
    /// * `Err<serde_binary::Error>` if it cant serialize
    /// * `Err<std::io::Error>` if there's an error writing to the file
    pub fn save_to_disk(&self) -> Result<(), Box<dyn Error>> {
        let mut path = constants::conf_dir();
        path.push("media_cache.bin");

        let serialized_songs: Vec<u8> = serde_binary::to_vec(self.songs(), Endian::Little)?;
        fs::write(path, serialized_songs)?;

        Ok(())
    }

    pub fn rescan_library(&mut self, dirs: &[PathBuf]) -> Result<(), io::Error> {
        let mut songs = vec![];

        for dir in dirs {
            let mut dir_scan = scan_dir(dir)?;

            songs.append(&mut dir_scan);
        }

        self.songs = songs;

        Ok(())
    }

    pub fn songs(&self) -> &Vec<SongFileInfo> {
        &self.songs
    }

    pub fn state(&self) -> &CacheState {
        &self.state
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SongFileInfo {
    pub filepath: PathBuf,
    pub title: String,
    pub length: Duration,
    pub artist: String,
    pub album_artist: Option<String>,
    pub album: Option<String>,
    pub track_number: Option<i32>,
    pub year: Option<i32>
}

pub fn scan_dir(dir: &Path) -> Result<Vec<SongFileInfo>, io::Error> {
    let x = read_dir(dir)?;
    let mut songs_vec: Vec<SongFileInfo> = vec![];

    for i in x {
        let i = i?;
        let file_type = i.file_type()?;

        if file_type.is_dir() {
            let mut scanned_dir = scan_dir(&i.path())?;

            songs_vec.append(&mut scanned_dir);
        } else {
            let path = i.path();

            if is_fileext_ok(path.extension()) {
                let info = scan_info(i.path());

                if let Ok(info) = info {
                    songs_vec.push(info);
                }
            }
        }
    }

    Ok(songs_vec)
}

fn is_fileext_ok(ext: Option<&OsStr>) -> bool {
    if let Some(ext) = ext {
        for ok_ext in constants::MUSIC_EXTS {
            if ext == ok_ext {
                return true;
            }
        }
    }

    return false;
}

fn scan_info(path: PathBuf) -> Result<SongFileInfo, LoftyError> {
    let tagged_file = probe::Probe::open(&path)?.read()?;

	let tag = match tagged_file.primary_tag() {
		Some(primary_tag) => primary_tag,
		None => tagged_file.first_tag().expect("ERROR: No tags found!"),
	};

    let year;
    let length;
    let track_number: Option<i32>;

    if let Some(tagitem) = tag.date() {
        year = Some(tagitem.year as i32);
    } else {
        year = None;
    }

    if let Ok(l) = mp3_duration::from_path(&path) {
        length = l;
    } else {
        length = Duration::ZERO;
    }

    if let Some(tagitem) = tag.get(ItemKey::TrackNumber) {
        if let Some(trackstr) = tagitem.clone().into_value().into_string() {
            if let Ok(track) = trackstr.parse() {
                track_number = Some(track);
            } else {
                track_number = None;
            }
        } else {
            track_number = None;
        }
    } else {
        track_number = None;
    }

    Ok(SongFileInfo {
        filepath: path,
        title: get_string(&tag, ItemKey::TrackTitle).unwrap_or("?".to_string()),
        length,
        artist: get_string(&tag, ItemKey::TrackArtist).unwrap_or("?".to_string()),
        album_artist: get_string(&tag, ItemKey::AlbumArtist),
        album: get_string(&tag, ItemKey::AlbumTitle),
        track_number,
        year
    })
}

fn get_string(tag: &Tag, item: ItemKey) -> Option<String> {
     if let Some(tagitem) = tag.get(item) {
        if let Some(item_string) = tagitem.clone().into_value().into_string() {
            return Some(item_string);
        } else {
            return None;
        }
    } else {
        return None;
    }
}

pub fn load_else_dead() -> Result<MediaCache, io::Error> {
    let cache = MediaCache::load_from_disk()?;

    if let Some(cache) = cache {
        Ok(cache)
    } else {
        Ok(MediaCache::dead())
    }
}
