use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Album {
    pub name: Option<String>,
    pub artist_names: Option<Vec<String>>,
    pub tracks: Vec<Uuid>,
}
