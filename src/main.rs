mod api_client;
mod fake_env;
mod models;

use api_client::{get_access_token, get_playing_song, update_revolt_status, refresh_access_token};
use fake_env::{CLIENT_ID, CLIENT_SECRET, REDIRECT_URI};
use reqwest::{header};
use std::{
    io::{self, Write},
    thread,
    time::Duration,
};

#[tokio::main]
async fn main() {
    println!("Go to the following URL to authorize the application:");
    let auth_url = format!(
        "https://accounts.spotify.com/authorize?client_id={}&response_type=code&redirect_uri={}&scope=user-read-currently-playing",
        CLIENT_ID, REDIRECT_URI
    );
    println!("{}", auth_url);

    let mut code = String::new();
    print!("Enter the authorization code: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut code).unwrap();
    let code = code.trim();

    let oauth_response = match get_access_token(CLIENT_ID, CLIENT_SECRET, code, REDIRECT_URI).await {
        Ok(token) => token,
        Err(e) => {
            eprintln!("Error getting access token: {}", e);
            return;
        }
    };

    let mut access_token = oauth_response.access_token.unwrap();
    let mut refresh_token = oauth_response.refresh_token.unwrap();

    let mut headers = header::HeaderMap::new();
    let mut auth_value = header::HeaderValue::from_str(format!("Bearer {}", access_token).as_str())
        .expect("Invalid header value");
    auth_value.set_sensitive(true);
    headers.insert(header::AUTHORIZATION, auth_value);

    let mut client = reqwest::Client::builder()
        .default_headers(headers.clone())
        .build()
        .expect("Could not build client!");

    loop {
        match get_playing_song(&client).await {
            Ok(Some(currently_playing)) => {
                let currently_playing = currently_playing;
                if let Some(track) = currently_playing.item {
                    let artist_names: Vec<&str> =
                        track.artists.iter().map(|a| a.name.as_str()).collect();
                    let mut status_text = format!(
                        "Listening to '{}' by '{}'",
                        track.name,
                        artist_names.join(", ")
                    );
                    if status_text.len() > 128 {
                        status_text =
                            format!("Listening to '{}' by '{}'", track.name, artist_names[0]);
                    }
                    println!("{}", status_text);

                    if let Err(e) = update_revolt_status(&status_text).await {
                        eprintln!("Error updating Revolt status: {}", e);
                    } else {
                        // wait the amount of time the song has left Plus 3 seconds just incase spotify has some weird shit happen?
                        thread::sleep(Duration::from_millis(
                            (track.duration_ms - currently_playing.progress_ms + 3000)
                                .try_into()
                                .expect("erm"),
                        ));
                    }
                } else {
                    println!("No track is currently playing.");
                }
            },
            Ok(None) => {
                println!("Spotify returned an error 204 - No content, are you sure you're listening to something?");
                thread::sleep(Duration::from_secs(10));
            },
            Err(e) => {
                if e.to_string() == "TOKEN_EXPIRED" {
                    println!("Token expired - refreshing");
                    match refresh_access_token(&client, &refresh_token).await {
                        Ok(new_tokens) => {
                            access_token = new_tokens.access_token.unwrap();
                            if let Some(new_refresh_token) = new_tokens.refresh_token {
                                refresh_token = new_refresh_token;
                            }

                            auth_value = header::HeaderValue::from_str(&format!("Bearer {}", access_token)).expect("Invalid header value");
                            auth_value.set_sensitive(true);
                            headers.insert(header::AUTHORIZATION, auth_value.clone());

                            client = reqwest::Client::builder()
                                .default_headers(headers.clone())
                                .build()
                                .expect("Could not build client!");

                            println!("Access token refreshed successfully.");
                        }
                        Err(err) => {
                            eprintln!("Error refreshing access token: {}", err);
                            thread::sleep(Duration::from_secs(10));
                        }
                    }
                } else {
                    eprintln!("Error: {}", e);
                    thread::sleep(Duration::from_secs(10));
                }
            }
        }
    }
}
