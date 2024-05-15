use serde::Deserialize;

#[derive(Deserialize)]
pub struct OAuthResponse {
    pub access_token: String,
}

#[derive(Deserialize)]
pub struct CurrentlyPlaying {
    pub progress_ms: i32,
    pub item: Option<Track>,
}

#[derive(Deserialize)]
pub struct Track {
    pub name: String,
    pub artists: Vec<Artist>,
    pub duration_ms: i32,
}

#[derive(Deserialize)]
pub struct Artist {
    pub name: String,
}
