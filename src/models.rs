use serde::Deserialize;

#[derive(Deserialize)]
pub struct OAuthResponse {
    pub access_token: String,
}

#[derive(Deserialize)]
pub struct CurrentlyPlaying {
    pub item: Option<Track>,
}

#[derive(Deserialize)]
pub struct Track {
    pub name: String,
    pub artists: Vec<Artist>,
}

#[derive(Deserialize)]
pub struct Artist {
    pub name: String,
}
