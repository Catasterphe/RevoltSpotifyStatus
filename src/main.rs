mod fake_env;
mod api_client;
mod models;

use api_client::{get_access_token, get_playing_song, update_revolt_status};
use fake_env::{CLIENT_ID, CLIENT_SECRET, REDIRECT_URI};
use reqwest::header;
use std::{io::{self, Write}, thread, time::Duration};

#[tokio::main]
async fn main() {
    println!("Go to the following URL to authorize the application:");
    let auth_url = format!(
        "https://accounts.spotify.com/authorize?client_id={}&response_type=code&redirect_uri={}&scope=user-read-playback-state",
        CLIENT_ID, REDIRECT_URI
    );
    println!("{}", auth_url);

    let mut code = String::new();
    print!("Enter the authorization code: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut code).unwrap();
    let code = code.trim();

    let access_token = match get_access_token(CLIENT_ID, CLIENT_SECRET, &code, REDIRECT_URI).await {
        Ok(token) => token,
        Err(e) => {
            eprintln!("Error getting access token: {}", e);
            return;
        }
    };

    let mut headers = header::HeaderMap::new();
    let mut auth_value = header::HeaderValue::from_str(format!("Bearer {}", access_token).as_str()).expect("Invalid header value");
    auth_value.set_sensitive(true);
    headers.insert(header::AUTHORIZATION, auth_value);

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .expect("Could not build client!");

        loop {
            match get_playing_song(&client).await {
                Ok(currently_playing) => {
                    if let Some(track) = currently_playing.item {
                        let artist_names: Vec<&str> = track.artists.iter().map(|a| a.name.as_str()).collect();
                        let status_text = format!("Currently playing: {} by {}", track.name, artist_names.join(", "));
                        println!("{}", status_text);
    
                        if let Err(e) = update_revolt_status(&status_text).await {
                            eprintln!("Error updating Revolt status: {}", e);
                        }
                    } else {
                        println!("No track is currently playing.");
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
            thread::sleep(Duration::from_secs(45));
        }
}
