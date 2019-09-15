use super::spotify_api::{
    endpoints::{ARTISTS_INFO, SAVED_TRACKS},
    models::{ArtistsResponse, SavedTrack, Track},
};
use super::CmdHandler;
use console::style;
use dialoguer::Checkboxes;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use std::collections::HashMap;
use std::convert::TryInto;
use std::error::Error;

impl CmdHandler {
    pub fn genres(&self) -> Result<(), Box<dyn Error>> {
        println!("Loading your library information...");
        let saved_tracks = self.paged_request::<SavedTrack>(SAVED_TRACKS)?;
        println!("Library loaded.");

        println!("Getting genre information...");
        let mut artist_map = HashMap::new();
        for saved_track in &saved_tracks {
            let track = &saved_track.track;
            artist_map
                .entry(&track.artists[0].id)
                .and_modify(|artist_tracks: &mut Vec<&Track>| {
                    artist_tracks.push(track);
                })
                .or_insert_with(|| vec![track]);
        }

        let artists = artist_map.keys().collect::<Vec<_>>();
        let chunks = artists.chunks(50);
        let mut genre_map = HashMap::new();

        let progress = ProgressBar::new(artists.len().try_into().unwrap()).with_style(
            ProgressStyle::default_bar()
                .template("[{wide_bar}] {pos}/{len}")
                .progress_chars("=> "),
        );
        for chunk in chunks {
            let data = self
                .client
                .get(&format!("{}?ids={}", ARTISTS_INFO, chunk.iter().join(",")))
                .send()?
                .json::<ArtistsResponse>()?;
            progress.inc(50);
            for artist in &data.artists {
                for genre in &artist.genres {
                    genre_map
                        .entry(String::from(genre))
                        .and_modify(|tracks: &mut Vec<String>| {
                            tracks.push(String::from(&artist.id));
                        })
                        .or_insert_with(|| vec![String::from(&artist.id)]);
                }
            }
        }
        progress.finish_and_clear();
        println!("Genre information loaded.");

        let genres = genre_map
            .into_iter()
            .map(|(genre, artists)| {
                (
                    genre,
                    artists
                        .into_iter()
                        .map(|artist| artist_map.get(&artist).unwrap())
                        .flatten()
                        .collect::<Vec<_>>(),
                )
            })
            .sorted_by(|(_, v1), (_, v2)| v2.len().cmp(&v1.len()))
            .collect::<Vec<_>>();
        let checkboxes = {
            let mut checkboxes = Checkboxes::new();
            checkboxes.with_prompt(
                &style("Select genres to create your playlist from")
                    .cyan()
                    .to_string(),
            );
            checkboxes.items(
                &genres
                    .iter()
                    .map(|(k, v)| format!("{} - {} songs", &k, v.len()))
                    .collect::<Vec<String>>()
                    .iter()
                    .map(|s| s.as_ref())
                    .collect::<Vec<&str>>()[..],
            );
            checkboxes.paged(true);
            checkboxes
        };

        let selection = checkboxes.interact()?;
        if selection.is_empty() {
            println!("No genres selected.");
        } else {
            let default_name = selection
                .iter()
                .map(|i| &genres.get(*i).unwrap().0)
                .join("/");
            let tracks = selection
                .into_iter()
                .map(|i| &genres.get(i).unwrap().1)
                .flatten()
                .map(|track| &track.uri)
                .unique()
                .collect::<Vec<_>>();

            self.create_playlist(tracks, &default_name)?;
        }

        Ok(())
    }
}
