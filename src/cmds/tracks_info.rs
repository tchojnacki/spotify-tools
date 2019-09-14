use super::spotify_api::{SavedTrack, SAVED_TRACKS_ENDPOINT};
use super::CmdHandler;
use console::style;
use std::collections::HashMap;
use std::error::Error;

/// Cap a number to a ten maximum
fn first_10(len: usize) -> usize {
    if 10 <= len {
        10
    } else {
        len
    }
}

struct NamedCounter {
    name: String,
    counter: u32,
}

impl CmdHandler {
    pub fn tracks_info(&self) -> Result<(), Box<dyn Error>> {
        println!("Loading your library information...");
        let saved_tracks = self.paged_request::<SavedTrack>(SAVED_TRACKS_ENDPOINT)?;
        println!("Library loaded.");

        let mut artist_counter = HashMap::new();
        let mut album_counter = HashMap::new();

        for track in &saved_tracks {
            for artist in &track.track.artists {
                artist_counter
                    .entry(artist.id.to_string())
                    .and_modify(|c: &mut NamedCounter| c.counter += 1)
                    .or_insert(NamedCounter {
                        name: artist.name.to_string(),
                        counter: 1,
                    });
            }

            album_counter
                .entry(track.track.album.id.to_string())
                .and_modify(|c: &mut NamedCounter| c.counter += 1)
                .or_insert(NamedCounter {
                    name: track.track.album.name.to_string(),
                    counter: 1,
                });
        }

        let mut top_artists = artist_counter.into_iter().collect::<Vec<_>>();
        top_artists.sort_by(|(_k1, v1), (_k2, v2)| v2.counter.cmp(&v1.counter)); // Sort by count

        let mut top_albums = album_counter.into_iter().collect::<Vec<_>>();
        top_albums.sort_by(|(_k1, v1), (_k2, v2)| v2.counter.cmp(&v1.counter)); // Sort by count

        println!(
            "{}",
            style(format!(
                "Your library contains {} songs from {} albums by {} artists.",
                saved_tracks.len(),
                top_albums.len(),
                top_artists.len()
            ))
            .cyan()
        );

        println!("{}", style("Most liked artists:").cyan());
        for artist in &top_artists[..first_10(top_artists.len())] {
            println!("{} - {} songs", artist.1.name, artist.1.counter);
        }

        println!("{}", style("Most liked albums:").cyan());
        for album in &top_albums[..first_10(top_albums.len())] {
            println!("{} - {} songs", album.1.name, album.1.counter);
        }

        Ok(())
    }
}
