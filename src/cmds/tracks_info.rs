use crate::cmds::spotify_api::{Paging, SavedTrack};
use crate::cmds::CmdHandler;
use console::style;
use indicatif::ProgressBar;
use std::collections::HashMap;
use std::error::Error;

fn first_10(len: usize) -> usize {
    if 10 <= len {
        10
    } else {
        len
    }
}

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
        println!("Library loaded.");

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
                .entry(track.track.album.id.to_string())
                .and_modify(|c: &mut (String, u32)| c.1 += 1)
                .or_insert((track.track.album.name.to_string(), 1));
        }

        let mut top_artists: Vec<(String, u32)> = artist_counter.into_iter().collect();
        top_artists.sort_by(|(_k1, v1), (_k2, v2)| v2.cmp(v1));

        let mut top_albums: Vec<(String, (String, u32))> = album_counter.into_iter().collect();
        top_albums.sort_by(|(_k1, v1), (_k2, v2)| v2.1.cmp(&v1.1));

        println!();
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

        println!();
        println!("{}", style("Most liked artists:").cyan());
        for artist in &top_artists[..first_10(top_artists.len())] {
            println!("{} - {} songs", artist.0, artist.1);
        }

        println!();
        println!("{}", style("Most liked albums:").cyan());
        for album in &top_albums[..first_10(top_albums.len())] {
            println!("{} - {} songs", (album.1).0, (album.1).1);
        }

        Ok(())
    }
}
