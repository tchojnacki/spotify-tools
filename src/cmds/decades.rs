use super::spotify_api::{
    endpoints::{GET_USER, PLAYLIST_CREATION, SAVED_TRACKS},
    models::{SavedTrack, SimplifiedPlaylist, Track, User},
};
use super::CmdHandler;
use console::style;
use dialoguer::{Checkboxes, Confirmation, Input};
use itertools::Itertools;
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;

impl CmdHandler {
    pub fn decades(&self) -> Result<(), Box<dyn Error>> {
        println!("Loading your library information...");
        let saved_tracks = self.paged_request::<SavedTrack>(SAVED_TRACKS)?;
        println!("Library loaded.");

        let mut decade_map = HashMap::new();

        for saved_track in &saved_tracks {
            let track = &saved_track.track;
            let decade = format!("{}0s", &track.album.release_date[0..3]);
            decade_map
                .entry(decade)
                .and_modify(|v: &mut Vec<&Track>| v.push(track))
                .or_insert_with(|| vec![track]);
        }

        let tracks_by_decades = decade_map
            .into_iter()
            .sorted_by(|(k1, _), (k2, _)| k1.cmp(&k2))
            .collect::<Vec<_>>();

        let checkboxes = {
            let mut checkboxes = Checkboxes::new();
            checkboxes.with_prompt(
                &style("Select decades to create your playlist from")
                    .cyan()
                    .to_string(),
            );
            checkboxes.items(
                &tracks_by_decades
                    .iter()
                    .map(|(k, v)| format!("{} - {} songs", &k[2..], v.len()))
                    .collect::<Vec<String>>()
                    .iter()
                    .map(|s| s.as_ref())
                    .collect::<Vec<&str>>()[..],
            );
            checkboxes
        };

        let selection = checkboxes.interact()?;

        if selection.is_empty() {
            println!("No decades selected.");
        } else {
            let default_name = selection
                .iter()
                .map(|i| &tracks_by_decades.get(*i).unwrap().0[2..])
                .join("/");
            let tracks = selection
                .into_iter()
                .map(|i| &tracks_by_decades.get(i).unwrap().1)
                .flatten()
                .map(|track| &track.uri)
                .collect::<Vec<_>>();

            let input = {
                let mut input = Input::<String>::new();
                input.with_prompt(
                    &style("Select the name of your new playlist")
                        .cyan()
                        .to_string(),
                );
                input.default(default_name);
                input
            };

            let name = input.interact()?;

            println!(
                "You are going to create a \"{}\" playlist containing {} songs.",
                name,
                tracks.len()
            );

            let confirm = {
                let mut confirm = Confirmation::new();
                confirm.with_text(&style("Do you want to proceed?").cyan().to_string());
                confirm
            };

            if confirm.interact()? {
                println!("Creating the playlist...");
                let user_id = self.client.get(GET_USER).send()?.json::<User>()?.id;
                let playlist = self
                    .client
                    .post(&PLAYLIST_CREATION.replace("{user_id}", &user_id))
                    .json(&json!({ "name": &name }))
                    .send()?
                    .json::<SimplifiedPlaylist>()?;

                println!("Adding songs to the playlist...");
                let chunks = tracks.chunks(100);
                for chunk in chunks {
                    self.client
                        .post(&playlist.tracks.href)
                        .json(&json!({ "uris": &chunk }))
                        .send()?;
                }
                println!("Playlist created.");
            } else {
                println!("Didn't create the playlist.");
            }
        }

        Ok(())
    }
}
