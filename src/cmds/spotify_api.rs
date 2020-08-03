pub mod endpoints {
    pub const GET_USER: &str = "https://api.spotify.com/v1/me";
    pub const SAVED_TRACKS: &str = "https://api.spotify.com/v1/me/tracks?limit=50";
    pub const SAVED_TRACKS_REMOVAL: &str = "https://api.spotify.com/v1/me/tracks";
    pub const ALL_PLAYLISTS: &str = "https://api.spotify.com/v1/me/playlists?limit=50";
    pub const PLAYLIST_CREATION: &str = "https://api.spotify.com/v1/users/{user_id}/playlists";
    pub const ARTISTS_INFO: &str = "https://api.spotify.com/v1/artists";
}

pub mod models {
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
        pub name: String,
        pub artists: Vec<SimplifiedArtist>,
        pub album: SimplifiedAlbum,
        pub id: String,
        pub uri: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct SimplifiedArtist {
        pub name: String,
        pub id: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct SimplifiedAlbum {
        pub album_type: String,
        pub name: String,
        pub release_date: String,
        pub id: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct SimplifiedPlaylist {
        pub name: String,
        pub owner: User,
        pub tracks: Tracks,
        pub snapshot_id: String,
        pub id: String,
        pub uri: String,
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

    #[derive(Deserialize, Debug)]
    pub struct User {
        pub id: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct ArtistsResponse {
        pub artists: Vec<FullArtist>,
    }

    #[derive(Deserialize, Debug)]
    pub struct FullArtist {
        pub id: String,
        pub genres: Vec<String>,
    }
}
