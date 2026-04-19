#[derive(Default, Clone)]
pub enum SongsViewType {
    #[default]
    All,
    ArtistSelect,
    Artist(String),
    AlbumSelect,
    Album(String),
}
