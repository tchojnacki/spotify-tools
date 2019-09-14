mod auth;
mod cmds;

fn main() {
    let token = auth::authorize().unwrap();
    let client = cmds::CmdHandler::new(&token).unwrap();

    loop {
        // Break loop on true (if should exit)
        if let true = client.select_cmd() {
            break;
        }
    }
}
