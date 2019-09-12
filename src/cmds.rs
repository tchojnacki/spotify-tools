use command::Command;
use dialoguer::Select;
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Client,
};
use std::error::Error;

mod command;
mod spotify_api;
mod tracks_info;

pub struct CmdHandler {
    client: Client,
}

impl CmdHandler {
    pub fn new(token: &str) -> Result<CmdHandler, Box<dyn Error>> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token))?,
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        Ok(CmdHandler { client })
    }

    pub fn select_cmd(&self) -> bool {
        let commands = Command::commands();
        let select = {
            let mut select = Select::new();
            select.with_prompt("Select an action");
            select.items(&commands);
            select.default(0);
            select
        };
        let answer = commands
            .get(select.interact().unwrap_or(commands.len() - 1))
            .unwrap_or(&Command::Exit);

        match answer {
            Command::TracksInfo => self.tracks_info().unwrap(),
            _ => (),
        };

        if let Command::Exit = answer {
            true
        } else {
            false
        }
    }
}
