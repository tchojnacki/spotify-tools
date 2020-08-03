use super::spotify_api::{
    endpoints::{ALL_PLAYLISTS, GET_USER, PLAYLIST_CREATION},
    models::{Paging, PlaylistTrack, SimplifiedPlaylist, User},
};
use super::CmdHandler;
use console::style;
use dialoguer::{Confirmation, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use serde::de::DeserializeOwned;
use serde_json::json;
use std::cmp::min;
use std::error::Error;

pub enum Command {
    TracksInfo,
    Duplicates,
    Decades,
    Genres,
    Exit,
}

impl Command {
    pub fn commands() -> Vec<Command> {
        vec![
            Command::TracksInfo,
            Command::Duplicates,
            Command::Decades,
            Command::Genres,
            Command::Exit,
        ]
    }
}

impl ToString for Command {
    fn to_string(&self) -> String {
        String::from(match &self {
            Command::TracksInfo => "Show information about top artists from your library",
            Command::Duplicates => "Remove duplicates from liked songs or from a playlist",
            Command::Decades => "Categorize your liked songs based on their release decade",
            Command::Genres => "Categorize your liked songs based on their artist's genre",
            Command::Exit => "Exit",
        })
    }
}

impl CmdHandler {
    pub fn select_cmd(&self) -> Result<bool, Box<dyn Error>> {
        let commands = Command::commands();
        let select = {
            let mut select = Select::new();
            select.with_prompt(&style("Select an action").cyan().to_string());
            select.items(&commands);
            select.default(0);
            select
        };

        println!();

        let answer = commands
            .get(select.interact().unwrap_or(commands.len() - 1))
            .unwrap();

        match answer {
            Command::TracksInfo => self.tracks_info()?,
            Command::Duplicates => self.duplicates()?,
            Command::Decades => self.decades()?,
            Command::Genres => self.genres()?,
            _ => (),
        };

        Ok(if let Command::Exit = answer {
            true
        } else {
            false
        })
    }

    pub fn paged_request<T: DeserializeOwned>(
        &self,
        endpoint: &str,
    ) -> Result<Vec<T>, Box<dyn Error>> {
        let mut next_url = Some(String::from(endpoint));
        let mut data: Vec<T> = Vec::new();
        let mut progress: Option<ProgressBar> = None;

        while next_url.is_some() {
            let resp = self
                .client
                .get(&next_url.unwrap())
                .send()?
                .error_for_status()?
                .json::<Paging<T>>()?;

            next_url = resp.next;
            data.extend(resp.items);

            if progress.is_none() {
                progress = Some(
                    ProgressBar::new(resp.total).with_style(
                        ProgressStyle::default_bar()
                            .template("[{wide_bar}] {pos}/{len}")
                            .progress_chars("=> "),
                    ),
                );
            }
            progress.as_ref().unwrap().inc(resp.limit);
        }
        progress.as_ref().unwrap().finish_and_clear();

        Ok(data)
    }

    fn open_playlist(&self, uri: &str) -> Result<(), Box<dyn Error>> {
        if Confirmation::new()
            .with_text(&style("Do you want to view it now?").cyan().to_string())
            .default(false)
            .interact()?
        {
            match open::that(uri) {
                Ok(_) => println!("Playlist opened in Spotify."),
                Err(_) => println!("See the playlist at: {}", uri),
            }
        }
        Ok(())
    }

    pub fn create_playlist(
        &self,
        tracks: Vec<&String>,
        default_name: &str,
    ) -> Result<(), Box<dyn Error>> {
        let name = Input::<String>::new()
            .with_prompt(
                &style("Select the name of your new playlist")
                    .cyan()
                    .to_string(),
            )
            .default(String::from(&default_name[..min(default_name.len(), 100)]))
            .interact()?;

        let name = &name[..min(name.len(), 100)];

        let user_id = self
            .client
            .get(GET_USER)
            .send()?
            .error_for_status()?
            .json::<User>()?
            .id;
        let playlists = self
            .paged_request::<SimplifiedPlaylist>(ALL_PLAYLISTS)?
            .into_iter()
            .filter(|playlist| playlist.owner.id == user_id && playlist.name == name)
            .collect::<Vec<_>>();

        if playlists.len() == 1 {
            let current = &playlists[0];
            println!(
                "You are going to update an existing \"{}\" playlist containing {} songs to have {} songs.",
                current.name, current.tracks.total, tracks.len()
            );
            if Confirmation::new()
                .with_text(&style("Do you want to proceed?").cyan().to_string())
                .interact()?
            {
                println!("Updating the playlist...");
                println!("Fetching current playlist information.");
                let current_tracks = self.paged_request::<PlaylistTrack>(&current.tracks.href)?;
                println!("Playlist information downloaded.");
                let current_uris = current_tracks
                    .iter()
                    .map(|t| &t.track.uri)
                    .collect::<Vec<_>>();
                let new_uris = tracks;

                let uris_to_delete = current_uris
                    .iter()
                    .filter(|&&c_uri| !new_uris.iter().any(|&n_uri| c_uri == n_uri))
                    .collect::<Vec<_>>();

                let uris_to_add = new_uris
                    .iter()
                    .filter(|&&n_uri| !current_uris.iter().any(|&c_uri| n_uri == c_uri))
                    .collect::<Vec<_>>();

                println!("Removing tracks from playlist...");
                let delete_chunks = uris_to_delete.chunks(100);
                for chunk in delete_chunks {
                    self.client
                        .delete(&current.tracks.href)
                        .json(&json!({
                            "tracks": chunk.iter().map(|c| {
                                    json!({
                                        "uri": &c
                                    })
                                }).collect::<Vec<_>>(),
                            "snapshot_id": &current.snapshot_id
                        }))
                        .send()?
                        .error_for_status()?;
                }
                println!("Tracks removed successfully.");

                println!("Adding tracks to the playlist...");
                let add_chunks = uris_to_add.chunks(100);
                for chunk in add_chunks {
                    self.client
                        .post(&current.tracks.href)
                        .json(&json!({ "uris": &chunk }))
                        .send()?
                        .error_for_status()?;
                }
                println!("Tracks added successfully.");
                println!("Playlist updated.");
                self.open_playlist(&current.uri)?;
            } else {
                println!("Didn't update the playlist.");
            }
            Ok(())
        } else {
            println!(
                "You are going to create a \"{}\" playlist containing {} songs.",
                name,
                tracks.len()
            );
            if Confirmation::new()
                .with_text(&style("Do you want to proceed?").cyan().to_string())
                .interact()?
            {
                println!("Creating the playlist...");
                let playlist = self
                    .client
                    .post(&PLAYLIST_CREATION.replace("{user_id}", &user_id))
                    .json(&json!({ "name": &name }))
                    .send()?
                    .error_for_status()?
                    .json::<SimplifiedPlaylist>()?;
                println!("Adding songs to the playlist...");
                let chunks = tracks.chunks(100);
                for chunk in chunks {
                    self.client
                        .post(&playlist.tracks.href)
                        .json(&json!({ "uris": &chunk }))
                        .send()?
                        .error_for_status()?;
                }
                println!("Playlist created.");
                self.open_playlist(&playlist.uri)?;
            } else {
                println!("Didn't create the playlist.");
            }
            Ok(())
        }
    }
}
