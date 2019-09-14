use serde::Deserialize;

pub const SAVED_TRACKS_ENDPOINT: &str = "https://api.spotify.com/v1/me/tracks?limit=50";
pub const PLAYLISTS_ENDPOINT: &str = "https://api.spotify.com/v1/me/playlists?limit=50";

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
    pub name: String,
    pub artists: Vec<Artist>,
    pub album: SimplifiedAlbum,
    pub id: String,
    pub uri: String,
}

#[derive(Deserialize, Debug)]
pub struct Artist {
    pub name: String,
    pub id: String,
}

#[derive(Deserialize, Debug)]
pub struct SimplifiedAlbum {
    pub album_type: String,
    pub name: String,
    pub id: String,
}

#[derive(Deserialize, Debug)]
pub struct SimplifiedPlaylist {
    pub name: String,
    pub tracks: Tracks,
    pub snapshot_id: String,
    pub id: String,
}

#[derive(Deserialize, Debug)]
pub struct PlaylistTrack {
    pub track: Track,
}

#[derive(Deserialize, Debug)]
pub struct Tracks {
    pub href: String,
    pub total: u64,
}
