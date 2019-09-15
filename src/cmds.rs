use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Client,
};
use std::error::Error;

mod decades;
mod duplicates;
mod genres;
mod spotify_api;
mod tracks_info;
mod util;

pub struct CmdHandler {
    client: Client,
}

impl CmdHandler {
    pub fn new(token: &str) -> Result<CmdHandler, Box<dyn Error>> {
        let mut headers = HeaderMap::new();
        // Set auth header
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token))?,
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        Ok(CmdHandler { client })
    }
}
