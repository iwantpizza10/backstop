use std::collections::HashSet;
use std::hash::Hash;
use std::io::ErrorKind;
use std::sync::Arc;
use std::{error::Error, ffi::OsStr, fs, io, path::PathBuf};
use audiotags::{AudioTag, MimeType, Tag};
use iced::futures::StreamExt;
use iced::widget::{image as iced_image, text};
use image::imageops::FilterType;
use serde_binary::binary_stream::Endian;
use uuid::Uuid;

use crate::menu_view::SongListItem;
use crate::{EventMessage, SongsViewType, constants, softunwrap_str};
use crate::saved_state::song_file_info::SongFileInfo;

#[derive(Debug, Default, Clone)]
pub enum CacheSortType {
    TitleAlphabetical,
    #[default]
    ArtistAlphabetical,
}

impl CacheSortType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::TitleAlphabetical => "By title (alphabetical)",
            Self::ArtistAlphabetical => "By artist (alphabetical)",
        }
    }
}

pub enum CacheFilterType {
    Artist(Arc<Artist>),
    Album(Arc<Album>),
    None,
}

#[derive(Debug, Clone, Eq)]
pub struct Artist {
    name: String,
    icon: Option<PathBuf>,
}

impl Hash for Artist {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Artist {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl SongListItem for Artist {
    fn image(&self) -> Option<iced_image::Image<iced_image::Handle>> {
        if let Some(icon) = &self.icon {
            Some(iced_image(icon))
        } else {
            None
        }
    }

    fn textrow_one<'a>(&'a self) -> Option<impl text::IntoFragment<'a>> {
        Some(&self.name)
    }

    fn event(&self) -> EventMessage {
        EventMessage::ChangeViewType(SongsViewType::Artist(Arc::new(self.clone())))
    }
}

#[derive(Debug, Clone, Eq)]
pub struct Album {
    name: String,
    artist: Option<String>,
    icon: Option<PathBuf>
}

impl Hash for Album {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Album {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl SongListItem for Album {
    fn image(&self) -> Option<iced_image::Image<iced_image::Handle>> {
        if let Some(icon) = &self.icon {
            Some(iced_image(icon))
        } else {
            None
        }
    }

    fn textrow_one<'a>(&'a self) -> Option<impl text::IntoFragment<'a>> {
        Some(&self.name)
    }

    fn textrow_two<'a>(&'a self) -> Option<impl text::IntoFragment<'a>> {
        if let Some(artist) = &self.artist {
            Some(artist)
        } else {
            None
        }
    }

    fn event(&self) -> EventMessage {
        EventMessage::ChangeViewType(SongsViewType::Album(Arc::new(self.clone())))
    }
}

#[derive(Debug, Clone, Default)]
pub struct MediaCache {
    apparent_songs: Vec<Arc<SongFileInfo>>,
    songs: HashSet<Arc<SongFileInfo>>,
    artists: Vec<Arc<Artist>>,
    albums: Vec<Arc<Album>>,
}

impl MediaCache {
    /// loads media cache from disk
    ///
    /// returns:
    /// * `Ok(_)` if it loaded fine
    /// * `Err<serde_binary::Error>` if it cant deserialize
    /// * `Err<std::io::Error>` if io error
    pub fn load() -> Result<Self, Box<dyn Error>> {
        let mut path = constants::conf_dir();
        path.push("media_cache_temp.bin"); // todo: untemp

        let file = fs::read(path);

        match file {
            Ok(file) => {
                let cache = serde_binary::from_vec::<HashSet<Arc<SongFileInfo>>>(file, Endian::Little)?;

                Ok(MediaCache::from(cache))
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

    /// saves media cache to disk
    ///
    /// returns:
    /// * Ok(()) if it saved w/o issue
    /// * `Err<serde_binary::Error>` if it cant serialize
    /// * `Err<std::io::Error>` if io error
    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let mut path = constants::conf_dir();
        path.push("media_cache_temp.bin"); // todo: untemp

        let serialized = serde_binary::to_vec(&self.songs, Endian::Little)?;
        fs::write(path, serialized)?;

        Ok(())
    }

    /// creates a new, empty media cache
    pub fn new() -> Self {
        Self::default()
    }

    /// creates a media cache from a hashset
    fn from_apparent(set: HashSet<Arc<SongFileInfo>>) -> Self {
        let mut instance = Self {
            songs: set.clone(),
            apparent_songs: set.iter().map(|x| x.clone()).collect(),
            albums: vec![],
            artists: vec![],
        };

        instance.index_filterables();

        instance
    }

    /// provides the list of songs found in the media cache
    pub fn songs(&self) -> &Vec<Arc<SongFileInfo>> {
        &self.apparent_songs
    }

    /// provides the list of albums found in the media cache
    pub fn albums(&self) -> &Vec<Arc<Album>> {
        &self.albums
    }

    /// provides the list of artists found in the media cache
    pub fn artists(&self) -> &Vec<Arc<Artist>> {
        &self.artists
    }

    /// sorts the list of songs
    pub fn sort(&mut self, sort_type: CacheSortType) {
        match sort_type {
            CacheSortType::ArtistAlphabetical => {
                self.apparent_songs.sort_by(|a, b| a.artist().to_lowercase().cmp(&b.artist().to_lowercase()));
            },
            CacheSortType::TitleAlphabetical => {
                self.apparent_songs.sort_by(|a, b| a.title().to_lowercase().cmp(&b.title().to_lowercase()));
            },
        }
    }

    pub fn filter(&mut self, filter_type: CacheFilterType) {
        match filter_type {
            CacheFilterType::Album(album) => {
                let mut songs = self.songs.iter()
                    .filter(|song| song.album == Some(album.name.clone()))
                    .map(|x| x.clone())
                    .collect::<Vec<_>>();

                songs.sort_by_key(|x| x.track_number);

                self.apparent_songs = songs;
            },
            CacheFilterType::Artist(artist) => {
                let mut songs = self.songs.iter()
                    .filter(|song| song.artist() == artist.name)
                    .map(|x| x.clone())
                    .collect::<Vec<_>>();

                songs.sort_by_key(|x| x.title());

                self.apparent_songs = songs;
            },
            CacheFilterType::None => {
                self.apparent_songs = self.songs.iter()
                    .map(|x| x.clone())
                    .collect()
            },
        }
    }

    /// creates a new MediaCache by scanning the privided directories
    pub async fn from_scan(dirs: Vec<PathBuf>) -> Result<Self, io::Error> {
        let mut files_list: Vec<PathBuf> = vec![];
        let mut song_metadata: HashSet<Arc<SongFileInfo>> = HashSet::new();
        let mut instance = Self::from(HashSet::new());

        // todo: uncomment
        // clear_covers_cache()?;

        for dir in dirs {
            let mut files = scan_dir(&dir).await?;

            files_list.append(&mut files);
        }

        for path in files_list {
            let metadata = scan_metadata(&path);

            if let Ok(metadata) = metadata {
                song_metadata.insert(Arc::new(metadata));
            }
        }

        instance.songs = song_metadata.clone();
        instance.apparent_songs = song_metadata.iter().map(|x| x.clone()).collect();
        instance.index_filterables();
        let _ = instance.save();

        Ok(instance)
    }

    /// reindex artists and albums for filtering
    fn index_filterables(&mut self) {
        self.albums.clear();
        self.artists.clear();
        let mut albums_set = HashSet::new();
        let mut artists_set = HashSet::new();

        for i in &self.songs {
            artists_set.insert(Arc::new(Artist {
                name: i.artist(),
                icon: i.cover.clone()
            }));

            if let Some(album) = &i.album {
                albums_set.insert(Arc::new(Album {
                    name: album.clone(),
                    artist: i.album_artist.clone(),
                    icon: i.cover.clone()
                }));
            }
        }

        self.albums = albums_set.iter()
            .map(|x| Arc::clone(x))
            .collect::<Vec<_>>();

        self.artists = artists_set.iter()
            .map(|x| Arc::clone(x))
            .collect::<Vec<_>>();

        self.albums.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        self.artists.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    }
}

impl From<HashSet<Arc<SongFileInfo>>> for MediaCache {
    fn from(value: HashSet<Arc<SongFileInfo>>) -> Self {
        MediaCache::from_apparent(value)
    }
}

async fn scan_dir(dir: &PathBuf) -> Result<Vec<PathBuf>, io::Error> {
    let dir = async_fs::read_dir(dir).await?;
    let mut paths: Vec<PathBuf> = vec![];

    for file in dir.collect::<Vec<_>>().await {
        let file = file?;
        let file_type = file.file_type().await?;

        if file_type.is_dir() {
            let mut subdir = Box::pin(scan_dir(&file.path())).await?;

            paths.append(&mut subdir);
        } else {
            let path = file.path();

            if is_ext_audio(path.extension()) {
                paths.push(path);
            }
        }
    }

    Ok(paths)
}

fn scan_metadata(path: &PathBuf) -> Result<SongFileInfo, audiotags::Error> {
    let tagged_file = Tag::default().read_from_path(&path)?;

    let album = tagged_file.album()
        .map_or(None, |x| Some(x.title.to_string()));

    let info = SongFileInfo::new(path.clone())
        .set_title(softunwrap_str!(tagged_file.title()))
        .set_artist(softunwrap_str!(tagged_file.artist()))
        .set_album_artist(softunwrap_str!(tagged_file.album_artist()))
        .set_album(album)
        .set_track_number(tagged_file.track_number())
        .set_year(tagged_file.year())
        .set_cover(process_cover(tagged_file));

    Ok(info)
}

fn process_cover(tagged_file: Box<dyn AudioTag + Send + Sync>) -> Option<PathBuf> {
    let cover;

    if let Some(c) = tagged_file.album_cover() {
        cover = c;
    } else {
        return None;
    }

    let cover_uuid = Uuid::new_v4().to_string();
    let mut cover_location = constants::conf_dir();
    cover_location.push("covers");
    cover_location.push(&cover_uuid);
    cover_location.set_extension(match cover.mime_type {
        MimeType::Bmp  => "bmp",
        MimeType::Gif  => "gif",
        MimeType::Jpeg => "jpg",
        MimeType::Png  => "png",
        MimeType::Tiff => "tiff",
    });

    let mut image = image::load_from_memory(cover.data).ok()?;

    let smaller_dimension = if image.height() < image.width() {
        image.height()
    } else {
        image.width()
    };

    let x_offset = (image.width() - smaller_dimension) / 2;

    image = image.crop(x_offset, 0, smaller_dimension, smaller_dimension);
    // switch to catmull-rom if this takes forever
    image = image.resize_exact(256, 256, FilterType::Lanczos3);
    let _ = image.save(&cover_location);

    Some(cover_location)
}

fn is_ext_audio(ext: Option<&OsStr>) -> bool {
    if let Some(ext) = ext {
        for allowed_ext in constants::MUSIC_EXTS {
            if ext == allowed_ext {
                return true;
            }
        }
    }

    false
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
