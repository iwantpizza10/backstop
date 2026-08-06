use uuid::Uuid;

use crate::album::Album;

pub struct Artist {
    pub name: Option<String>,
    pub albums: Vec<Album>,
    pub tracks: Vec<Uuid>,
}
