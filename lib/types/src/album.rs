use uuid::Uuid;

use crate::artist::Artist;

pub struct Album {
    pub name: Option<String>,
    pub artist: Option<Vec<Artist>>,
    pub tracks: Vec<Uuid>,
}
