use super::spotify_api::Paging;
use super::CmdHandler;
use console::style;
use dialoguer::Select;
use indicatif::{ProgressBar, ProgressStyle};
use serde::de::DeserializeOwned;
use std::error::Error;

pub enum Command {
    TracksInfo,
    Duplicates,
    Exit,
}

impl Command {
    pub fn commands() -> Vec<Command> {
        vec![Command::TracksInfo, Command::Duplicates, Command::Exit]
    }
}

impl ToString for Command {
    fn to_string(&self) -> String {
        String::from(match &self {
            Command::TracksInfo => "Show information about tracks saved to your library",
            Command::Duplicates => "Remove duplicates from saved tracks or from a playlist",
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
            let resp: Paging<T> = self.client.get(&next_url.unwrap()).send()?.json()?;

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
            } else {
                progress.as_mut().unwrap().inc(resp.limit);
            }
        }

        Ok(data)
    }
}