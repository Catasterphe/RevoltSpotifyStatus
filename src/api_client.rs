use reqwest::{Client};
use crate::fake_env::{LOGIN_API, BASE_API, REVOLT_USER_TOKEN, REVOLT_USER_ID};
use crate::models::{OAuthResponse, CurrentlyPlaying};
use std::error::Error;

pub async fn get_access_token(client_id: &str, client_secret: &str, auth_code: &str, redirect_uri: &str) -> Result<String, Box<dyn Error>> {
    let client = Client::new();

    let params = [
        ("grant_type", "authorization_code"),
        ("code", auth_code),
        ("redirect_uri", redirect_uri),
        ("client_id", client_id),
        ("client_secret", client_secret),
    ];

    let response = client.post(LOGIN_API)
        .form(&params)
        .send()
        .await?;

    if response.status().is_success() {
        let token_response: OAuthResponse = response.json().await?;
        Ok(token_response.access_token)
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to get access token: {}", response.text().await?),
        )))
    }
}

pub async fn get_playing_song(client: &Client) -> Result<CurrentlyPlaying, Box<dyn Error>> {
    let response = client.get(BASE_API).send().await?;

    if response.status().is_success() {
        let currently_playing: CurrentlyPlaying = response.json().await?;
        Ok(currently_playing)
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to get current song: {}", response.text().await?),
        )))
    }
}

pub async fn update_revolt_status(status_text: &str) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let url = format!("https://api.revolt.chat/users/{}", REVOLT_USER_ID);

    let body = serde_json::json!({
        "status": {
            "text": status_text,
            "presence": "Online"
        }
    });

    let response = client.patch(&url)
        .header("x-session-token", REVOLT_USER_TOKEN)
        .json(&body)
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to update status: {}", response.text().await?),
        )))
    }
}