use crate::fake_env::{BASE_API, CLIENT_ID, CLIENT_SECRET, LOGIN_API, REVOLT_USER_ID, REVOLT_USER_TOKEN};
use crate::models::{CurrentlyPlaying, OAuthResponse};
use reqwest::{Client, StatusCode};
use std::error::Error;
use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine as _};

pub async fn get_access_token(
    client_id: &str,
    client_secret: &str,
    auth_code: &str,
    redirect_uri: &str,
) -> Result<OAuthResponse, Box<dyn Error>> {
    let client = Client::new();

    let params = [
        ("grant_type", "authorization_code"),
        ("code", auth_code),
        ("redirect_uri", redirect_uri),
        ("client_id", client_id),
        ("client_secret", client_secret),
    ];

    let response = client.post(LOGIN_API).form(&params).send().await?;

    if response.status().is_success() {
        let token_response: OAuthResponse = response.json().await?;
        Ok(token_response)
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to get access token: {}", response.text().await?),
        )))
    }
}

pub async fn refresh_access_token(client: &Client, refresh_token: &str) -> Result<OAuthResponse, Box<dyn Error>> {
    println!("{refresh_token}");
    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
    ];

    let auth_value = format!(
        "Basic {}",
        STANDARD_NO_PAD.encode(format!("{}:{}", CLIENT_ID, CLIENT_SECRET))
    );

    let response = client
        .post(LOGIN_API)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Authorization", auth_value)
        .form(&params)
        .send()
        .await?;

    if response.status().is_success() {
        let oauth_response: OAuthResponse = response.json().await?;
        Ok(oauth_response)
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to refresh token: {}", response.status()),
        )))
    }
}

pub async fn get_playing_song(client: &Client) -> Result<Option<CurrentlyPlaying>, Box<dyn Error>> {
    let response = client.get(BASE_API).send().await?;

    match response.status() {
        StatusCode::OK => {
            let currently_playing: CurrentlyPlaying = response.json().await?;
            return Ok(Some(currently_playing));
        },
        StatusCode::NO_CONTENT => return Ok(None),
        StatusCode::UNAUTHORIZED => return Err("TOKEN_EXPIRED".into()),
        _ => return Err(Box::new(std::io::Error::new(
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

    let response = client
        .patch(&url)
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