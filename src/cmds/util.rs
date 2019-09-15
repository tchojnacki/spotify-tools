use super::spotify_api::{
    endpoints::{GET_USER, PLAYLIST_CREATION},
    models::{Paging, SimplifiedPlaylist, User},
};
use super::CmdHandler;
use console::style;
use dialoguer::{Confirmation, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use serde::de::DeserializeOwned;
use serde_json::json;
use std::error::Error;
use std::cmp::min;

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
            Command::TracksInfo => "Show information about liked songs from your library",
            Command::Duplicates => "Remove duplicates from liked songs or from a playlist",
            Command::Decades => "Categorize your liked songs based on their release decade",
            Command::Genres => "Categorize your liked songs based on their artist's genre",
            Command::Exit => "Exit",
        })
    }
}

impl CmdHandler {
    pub fn select_cmd(&self) -> bool {
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
            Command::TracksInfo => self.tracks_info().unwrap(),
            Command::Duplicates => self.duplicates().unwrap(),
            Command::Decades => self.decades().unwrap(),
            Command::Genres => self.genres().unwrap(),
            _ => (),
        };

        if let Command::Exit = answer {
            true
        } else {
            false
        }
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
            .default(String::from(default_name))
            .interact()?;
        
        let name = &name[..min(name.len(), 100)];

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

            if Confirmation::new()
                .with_text(&style("Do you want to view it now?").cyan().to_string())
                .default(false)
                .interact()?
            {
                match open::that(&playlist.uri) {
                    Ok(_) => println!("Playlist opened in Spotify."),
                    Err(_) => println!("See the playlist at: {}", playlist.uri),
                }
            }
        } else {
            println!("Didn't create the playlist.");
        }
        Ok(())
    }
}
