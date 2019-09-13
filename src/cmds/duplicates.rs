use super::spotify_api::{
    PlaylistTrack, SavedTrack, SimplifiedPlaylist, Track, PLAYLISTS_ENDPOINT, SAVED_TRACKS_ENDPOINT,
};
use super::CmdHandler;
use console::style;
use dialoguer::{Confirmation, Select};
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;

enum Target {
    SavedTracks,
    Playlist(SimplifiedPlaylist),
}

impl ToString for Target {
    fn to_string(&self) -> String {
        match &self {
            Target::SavedTracks => String::from("Songs saved in your library"),
            Target::Playlist(p) => format!("{} - {} tracks", p.name, p.tracks.total),
        }
    }
}

#[derive(Debug)]
struct Duplicate {
    name: String,
    id: String,
    uri: String,
    index: usize,
}

impl Duplicate {
    fn from_indexed_track(indexed_track: (usize, &Track)) -> Duplicate {
        Duplicate {
            name: indexed_track.1.name.to_string(),
            id: indexed_track.1.id.to_string(),
            uri: indexed_track.1.uri.to_string(),
            index: indexed_track.0,
        }
    }
}

fn find_duplicates(tracks: Vec<Track>) -> Vec<Duplicate> {
    let mut dup_map = HashMap::new();
    let mut duplicates = Vec::new();
    for indexed_track in tracks.iter().enumerate() {
        dup_map
            .entry(&indexed_track.1.artists[0].name)
            .and_modify(|album_entry: &mut HashMap<&str, (usize, &Track)>| {
                album_entry
                    .entry(&indexed_track.1.name)
                    .and_modify(|previous_track| {
                        // Prefer a track from an album instead of a single
                        match (
                            &previous_track.1.album.album_type[..],
                            &indexed_track.1.album.album_type[..],
                        ) {
                            (_, "album") => {
                                // Swap
                                duplicates.push(Duplicate::from_indexed_track(*previous_track));
                                *previous_track = indexed_track;
                            }
                            ("album", _) => {
                                // Don't swap
                                duplicates.push(Duplicate::from_indexed_track(indexed_track));
                            }
                            _ => {
                                // If both are from an album (are not a single) prefer lower album id
                                if indexed_track.1.album.id < previous_track.1.album.id {
                                    // Swap
                                    duplicates.push(Duplicate::from_indexed_track(*previous_track));
                                    *previous_track = indexed_track;
                                } else {
                                    // Don't swap
                                    duplicates.push(Duplicate::from_indexed_track(indexed_track));
                                }
                            }
                        }
                    })
                    .or_insert(indexed_track);
            })
            .or_insert_with(|| {
                let mut song_map = HashMap::new();
                song_map.insert(&indexed_track.1.name[..], indexed_track);
                song_map
            });
    }
    duplicates
}

impl CmdHandler {
    pub fn duplicates(&self) -> Result<(), Box<dyn Error>> {
        println!("Loading your playlists...");
        let playlists = self.paged_request::<SimplifiedPlaylist>(PLAYLISTS_ENDPOINT)?;
        println!("Playlists loaded.");

        let choices = {
            let mut choices = vec![Target::SavedTracks];
            choices.extend(playlists.into_iter().map(Target::Playlist));
            choices
        };

        let select = {
            let mut select = Select::new();
            select.with_prompt(&style("Remove duplicates from").cyan().to_string());
            select.items(&choices);
            select.default(0);
            select
        };

        println!();
        let target = choices.get(select.interact().unwrap_or(0)).unwrap();

        println!("Looking for duplicates...");
        let tracks = match &target {
            Target::SavedTracks => {
                let tracks = self.paged_request::<SavedTrack>(SAVED_TRACKS_ENDPOINT)?;
                tracks.into_iter().map(|t| t.track).collect::<Vec<_>>()
            }
            Target::Playlist(p) => {
                let tracks = self.paged_request::<PlaylistTrack>(&p.tracks.href)?;
                tracks.into_iter().map(|t| t.track).collect::<Vec<_>>()
            }
        };

        let duplicates = find_duplicates(tracks);

        if duplicates.is_empty() {
            println!("No duplicates found.");
        } else {
            println!();
            println!(
                "{}",
                style(format!("Found {} duplicates:", duplicates.len())).cyan()
            );
            for dup in &duplicates {
                println!("{}", dup.name);
            }

            let confirm = {
                let mut confirm = Confirmation::new();
                confirm.with_text(&style("Do you want to delete them?").cyan().to_string());
                confirm.default(false);
                confirm
            };

            if confirm.interact()? {
                match &target {
                    Target::SavedTracks => {
                        let chunks = duplicates.chunks(50);
                        for chunk in chunks {
                            let data = chunk.iter().map(|c| &c.id).collect::<Vec<_>>();
                            self.client
                                .delete(SAVED_TRACKS_ENDPOINT)
                                .json(&data)
                                .send()?;
                        }
                        println!("Duplicates removed successfully.");
                    }
                    Target::Playlist(p) => {
                        let chunks = duplicates.chunks(100);
                        for chunk in chunks {
                            let data = json!({
                                "tracks": chunk.iter().map(|c| {
                                        json!({
                                            "uri": &c.uri,
                                            "positions": [&c.index]
                                        })
                                    }).collect::<Vec<_>>(),
                                "snapshot_id": &p.snapshot_id
                            });
                            self.client.delete(&p.tracks.href).json(&data).send()?;
                        }
                        println!("Duplicates removed successfully.");
                    }
                }
            } else {
                println!("No duplicates removed.");
            }
        }

        Ok(())
    }
}
