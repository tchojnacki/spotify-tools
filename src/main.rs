use std::error::Error;

mod auth;
mod cmds;

fn run() -> Result<(), Box<dyn Error>> {
    let token = auth::authorize()?;
    let client = cmds::CmdHandler::new(&token)?;

    loop {
        // Break loop on true (if should exit)
        if let true = client.select_cmd()? {
            break;
        }
    }

    Ok(())
}

fn main() {
    if cfg!(debug_assertions) {
        run().unwrap();
    } else {
        run().unwrap_or_else(|_| println!("Unexpected error occurred."));
    }
}
