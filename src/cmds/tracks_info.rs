use crate::cmds::spotify_api::{Paging, SavedTrack};
use crate::cmds::CmdHandler;
use indicatif::ProgressBar;
use std::collections::HashMap;
use std::error::Error;

impl CmdHandler {
    pub fn tracks_info(&self) -> Result<(), Box<dyn Error>> {
        let mut next_url = Some(String::from(
            "https://api.spotify.com/v1/me/tracks?limit=50",
        ));
        let mut saved_tracks: Vec<SavedTrack> = Vec::new();
        let mut progress: Option<ProgressBar> = None;

        println!("Loading your library information...");
        while next_url != None {
            let resp: Paging<SavedTrack> = self.client.get(&next_url.unwrap()).send()?.json()?;

            next_url = resp.next;
            saved_tracks.extend(resp.items);

            if progress.is_none() {
                progress = Some(
                    ProgressBar::new(resp.total).with_style(
                        indicatif::ProgressStyle::default_bar()
                            .template("[{wide_bar}] {pos}/{len}")
                            .progress_chars("=> "),
                    ),
                );
            } else {
                progress.as_mut().unwrap().inc(resp.limit);
            }
        }

        if progress.is_some() {
            progress.unwrap().finish_and_clear();
        }

        let mut artist_counter = HashMap::new();
        let mut album_counter = HashMap::new();

        for track in &saved_tracks {
            for artist in &track.track.artists {
                artist_counter
                    .entry(artist.name.to_string())
                    .and_modify(|c| *c += 1)
                    .or_insert(1);
            }

            album_counter
                .entry((track.track.album.id.to_string(), track.track.album.name.to_string()))
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }

        let mut top_artists: Vec<(String, u32)> = artist_counter.into_iter().collect();
        top_artists.sort_by(|(_k1, v1), (_k2, v2)| v2.cmp(v1));

        let mut top_albums: Vec<((String, String), u32)> = album_counter.into_iter().collect();
        top_albums.sort_by(|(_k1, v1), (_k2, v2)| v2.cmp(v1));

        println!(
            "Your library contains {} songs from {} albums by {} artists.",
            saved_tracks.len(),
            top_albums.len(),
            top_artists.len()
        );

        println!("Most liked artists:");
        for artist in &top_artists[..(if 10 <= top_artists.len() {
            10
        } else {
            top_artists.len()
        })] {
            println!("{} - {} songs", artist.0, artist.1);
        }

        println!("Most liked albums:");
        for album in &top_albums[..(if 10 <= top_albums.len() {
            10
        } else {
            top_albums.len()
        })] {
            println!("{} - {} songs", (album.0).1, album.1);
        }

        Ok(())
    }
}
