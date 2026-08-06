use chrono::{DateTime, Local};
use uuid::Uuid;

use crate::album::Album;
use crate::artist::Artist;

pub struct SongItem {
    pub title: Option<String>,
    pub artist: Option<Vec<Artist>>,
    pub date: Option<DateTime<Local>>,
    pub track: Option<i32>,
    pub album: Option<Album>,
    pub id: Uuid,
}
