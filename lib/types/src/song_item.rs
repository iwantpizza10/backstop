use lofty::tag::items::Timestamp;
use uuid::Uuid;

pub struct SongItem {
    pub title: Option<String>,
    pub artist_names: Option<Vec<String>>,
    pub date: Option<Timestamp>,
    pub genre: Option<String>,
    pub track: Option<u32>,
    pub total_tracks: Option<u32>,
    pub album_name: Option<String>,
    pub album_artists: Option<Vec<String>>,
    pub id: Uuid,
}
