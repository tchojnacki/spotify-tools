use command::Command;
use dialoguer::Select;

mod command;
mod tracks_info;

pub struct Client {
    token: String,
}

impl Client {
    pub fn new(token: String) -> Client {
        Client { token }
    }

    pub fn select_cmd(&self) -> bool {
        let commands = Command::commands();
        let mut select = Select::new();
        select.with_prompt("Select an action");
        select.items(&commands);
        select.default(0);
        let answer = commands
            .get(select.interact().unwrap_or(commands.len() - 1))
            .unwrap_or(&Command::Exit);

        match answer {
            Command::TracksInfo => self.tracks_info(),
            _ => (),
        };

        if let Command::Exit = answer {
            true
        } else {
            false
        }
    }
}
