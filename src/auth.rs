use std::error::Error;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use url::Url;

const CLIENT_ID: &str = "bda57df0ca3244ea96cc8f16dfe04ab7";
const SCOPES: &[&str] = &[
    "playlist-read-private",
    "playlist-read-collaborative",
    "playlist-modify-private",
    "playlist-modify-public",
    "user-library-read",
    "user-library-modify",
];
const LOCALHOST: &str = "http://localhost";
const PORT: u32 = 8000;
const CALLBACK: &str = "/callback";
const REDIRECT: &str = "/redirect";

enum AuthStage {
    /// User will now visit CALLBACK, which should redirect him to REDIRECT
    ShouldCallback,
    /// User will now visit REDIRECT, which should return a token
    ShouldRedirect,
}

enum Response {
    /// URL contains an access_token in the hash part, redirect the user to REDIRECT with access_token as a query param
    Callback,
    /// URL contains an access_token in the query params
    Redirect,
    BadRequest,
}

impl Response {
    fn html_response(body: &str, status: &str) -> Vec<u8> {
        // Basic CSS styling
        let style = [
            "body {",
            "margin: 2.5em;",
            "font-family: sans-serif;",
            "text-align: center;",
            "}",
        ]
        .join("\r\n");

        // HTML boilerplate
        let content = [
            "<!DOCTYPE html>",
            "<html>",
            "<head>",
            "<title>spotify-tools</title>",
            &format!("<style>{}</style>", style),
            "</head>",
            &format!("<body>{}</body>", body),
            "</html>",
        ]
        .join("\r\n");

        // HTTP response boilerplate
        [
            &format!("HTTP/1.1 {}", status),
            "Server: spotify-tools",
            "Content-Type: text/html; charset=utf-8",
            &format!("Content-Length: {}", content.as_bytes().len()),
            "",
            &content,
        ]
        .join("\r\n")
        .into_bytes()
    }

    fn content(&self) -> Vec<u8> {
        match self {
            Response::Callback => Response::html_response(&format!("<script>window.location = window.location.origin + '{}?' + window.location.hash.substring(1);</script>", REDIRECT), "200 OK"),
            Response::Redirect => Response::html_response("Authorization complete. Please return to your terminal. This tab can be closed.", "200 OK"),
            Response::BadRequest => Response::html_response("Bad Request", "400 Bad Request")
        }
    }
}

pub fn authorize() -> Result<String, Box<dyn Error>> {
    let auth_url = format!("https://accounts.spotify.com/authorize?response_type=token&client_id={}&redirect_uri=http://localhost:{}{}&scope={}", CLIENT_ID, PORT, CALLBACK, SCOPES.join(" "));

    match open::that(&auth_url) {
        // Link opened in user's browser
        Ok(_) => println!("Please check the tab opened in your browser."),
        // Could not open the link in user's browser
        Err(_) => println!("Open the following link in your browser:\r\n{}", &auth_url),
    }

    let listener = TcpListener::bind(format!("127.0.0.1:{}", PORT))?;

    let mut stage = AuthStage::ShouldCallback;

    Ok(loop {
        // Await a connection
        let (mut socket, _addr) = listener.accept()?;

        let req = {
            let mut req = String::new();
            BufReader::new(&socket).read_line(&mut req)?;
            req
        };

        let req_url = Url::parse(&format!(
            "{}:{}{}",
            LOCALHOST,
            PORT,
            req.split_whitespace().nth(1).unwrap_or("") // Path from the request
        ))?;

        match (&stage, req_url.path()) {
            (AuthStage::ShouldCallback, CALLBACK) => {
                // Respond with HTML content that should redirect the user to REDIRECT with a query param containing access_token
                socket.write_all(&Response::Callback.content())?;
                stage = AuthStage::ShouldRedirect;
            }
            (AuthStage::ShouldRedirect, REDIRECT) => {
                // Got redirected from CALLBACK, query params should contain an access_token
                match req_url.query_pairs().find(|(k, _v)| k == "access_token") {
                    Some(token) => {
                        println!("Authorization successful.");
                        socket.write_all(&Response::Redirect.content())?;
                        break String::from(token.1); // Return token from loop
                    }
                    None => {
                        println!("Token not found. Please try again.");
                        socket.write_all(&Response::BadRequest.content())?;
                        stage = AuthStage::ShouldCallback;
                    }
                };
            }
            _ => {
                socket.write_all(&Response::BadRequest.content())?;
            }
        };
    })
}
