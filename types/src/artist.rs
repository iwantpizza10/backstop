use uuid::Uuid;

#[derive(Debug)]
pub struct Artist {
    pub name: Option<String>,
    pub album_names: Vec<Option<String>>,
    pub tracks: Vec<Uuid>,
}
