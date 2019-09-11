pub enum Command {
    TracksInfo,
    Exit,
}

impl Command {
    pub fn commands() -> Vec<Command> {
        vec![Command::TracksInfo, Command::Exit]
    }
}

impl ToString for Command {
    fn to_string(&self) -> String {
        String::from(match &self {
            Command::TracksInfo => "Show information about saved tracks",
            Command::Exit => "Exit",
        })
    }
}
