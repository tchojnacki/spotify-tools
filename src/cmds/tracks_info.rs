use crate::cmds::Client;

impl Client {
    pub fn tracks_info(&self) {
        println!("{}", self.token);
    }
}
