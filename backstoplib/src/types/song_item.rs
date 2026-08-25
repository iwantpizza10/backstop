use lofty::tag::items::Timestamp;
use uuid::Uuid;

#[derive(Debug)]
pub struct SongItem {
    pub title: Option<String>,
    pub track_artists: Option<Vec<String>>,
    pub date: Option<Timestamp>,
    pub genre: Option<String>,
    pub track: Option<u32>,
    pub total_tracks: Option<u32>,
    pub album_name: Option<String>,
    pub album_artists: Option<Vec<String>>,
    pub id: Uuid,
}
