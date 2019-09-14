use super::spotify_api::{endpoints::SAVED_TRACKS, models::SavedTrack};
use super::CmdHandler;
use console::style;
use itertools::Itertools;
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

struct NamedCounter<'a> {
    name: &'a str,
    counter: u32,
}

impl CmdHandler {
    pub fn tracks_info(&self) -> Result<(), Box<dyn Error>> {
        println!("Loading your library information...");
        let saved_tracks = self.paged_request::<SavedTrack>(SAVED_TRACKS)?;
        println!("Library loaded.");

        let mut artist_counter = HashMap::new();
        let mut album_counter = HashMap::new();

        for track in &saved_tracks {
            for artist in &track.track.artists {
                artist_counter
                    .entry(&artist.id)
                    .and_modify(|c: &mut NamedCounter| c.counter += 1)
                    .or_insert(NamedCounter {
                        name: &artist.name,
                        counter: 1,
                    });
            }

            album_counter
                .entry(&track.track.album.id)
                .and_modify(|c: &mut NamedCounter| c.counter += 1)
                .or_insert(NamedCounter {
                    name: &track.track.album.name,
                    counter: 1,
                });
        }

        let top_artists = artist_counter
            .into_iter()
            .sorted_by(|(_, v1), (_, v2)| v2.counter.cmp(&v1.counter))
            .collect::<Vec<_>>();

        let top_albums = album_counter
            .into_iter()
            .sorted_by(|(_, v1), (_, v2)| v2.counter.cmp(&v1.counter))
            .collect::<Vec<_>>();

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
