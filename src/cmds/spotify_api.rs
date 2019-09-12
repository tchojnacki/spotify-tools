use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Paging<T> {
    pub items: Vec<T>,
    pub limit: u64,
    pub next: Option<String>,
    pub total: u64,
}

#[derive(Deserialize, Debug)]
pub struct SavedTrack {
    pub track: Track,
}

#[derive(Deserialize, Debug)]
pub struct Track {
    pub artists: Vec<Artist>,
    pub album: SimplifiedAlbum,
}

#[derive(Deserialize, Debug)]
pub struct Artist {
    pub name: String,
    pub id: String,
}

#[derive(Deserialize, Debug)]
pub struct SimplifiedAlbum {
    pub name: String,
    pub id: String,
}
