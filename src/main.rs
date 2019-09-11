mod auth;
mod cmds;

fn main() {
    let token = auth::authorize().unwrap();
    let client = cmds::Client::new(token);

    loop {
        if let true = client.select_cmd() {
            break;
        }
    }
}
