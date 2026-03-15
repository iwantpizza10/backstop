use std::{error::Error, ffi::OsStr, fs::{self, read_dir}, io, path::{Path, PathBuf}, time::Duration};
use audiotags::{MimeType, Tag};
use serde::{Deserialize, Serialize};
use serde_binary::binary_stream::Endian;
use uuid::Uuid;
use crate::constants;

#[derive(Debug, PartialEq)]
pub enum CacheState {
    /// cache loaded from disk just fine
    Healthy,
    /// cache files not present on disk
    Dead
}

#[derive(Debug, PartialEq)]
pub enum SortType {
    Alphabetical,
    ReverseAlphabetical
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
        clear_covers_cache()?;

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

    pub fn sort(&mut self, sort_type: SortType) {
        match sort_type {
            SortType::Alphabetical => {
                self.songs.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
            },
            SortType::ReverseAlphabetical => {
                self.songs.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
                self.songs.reverse();
            }
        }
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
    pub year: Option<i32>,
    pub cover: Option<String>
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

fn scan_info(path: PathBuf) -> Result<SongFileInfo, Box<dyn Error>> {
    let tagged_file = Tag::default().read_from_path(&path)?;
    let length;
    let album_title;
    let cover_uuid = Uuid::new_v4().to_string();
    let mut cover_exists = false;
    let mut cover_location;

    if let Ok(l) = mp3_duration::from_path(&path) {
        length = l;
    } else {
        length = Duration::ZERO;
    }

    if let Some(album) = tagged_file.album() {
        album_title = Some(album.title);
    } else {
        album_title = None;
    }

    if let Some(cover) = tagged_file.album_cover() {
        cover_exists = true;
        cover_location = constants::conf_dir();
        cover_location.push("covers");
        cover_location.push(&cover_uuid);
        cover_location.set_extension(match cover.mime_type {
            MimeType::Bmp  => "bmp",
            MimeType::Gif  => "gif",
            MimeType::Jpeg => "jpeg",
            MimeType::Png  => "png",
            MimeType::Tiff => "tiff"
        });

        let image = image::load_from_memory(cover.data)?
            .thumbnail_exact(256, 256);

        image.save(&cover_location)?;
    } else {
        cover_location = constants::conf_dir();
    }

    let cover;

    if cover_exists && let Some(location) = cover_location.to_str() {
        cover = Some(location.to_string());
    } else {
        cover = None
    }

    Ok(SongFileInfo {
        filepath: path,
        title: option_str_string_thing(tagged_file.title()).unwrap_or("?".to_string()),
        length,
        artist: option_str_string_thing(tagged_file.artist()).unwrap_or("?".to_string()),
        album_artist: option_str_string_thing(tagged_file.album_artist()),
        album: option_str_string_thing(album_title),
        track_number: option_u16_i32(tagged_file.track_number()),
        year: tagged_file.year(),
        cover
    })
}

fn option_str_string_thing(input: Option<&str>) -> Option<String> {
    if let Some(input) = input {
        Some(input.to_string())
    } else {
        None
    }
}

fn option_u16_i32(input: Option<u16>) -> Option<i32> { // couldnt think of a rhyme unfortunately
    if let Some(input) = input {
        Some(input as i32)
    } else {
        None
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

fn clear_covers_cache() -> Result<(), io::Error> {
    let mut path = constants::conf_dir();
    path.push("covers");
    let dir = fs::read_dir(path)?;

    for i in dir {
        if let Ok(file) = i {
            fs::remove_file(file.path())?;
        }
    }

    Ok(())
}
