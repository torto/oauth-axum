use crate::CustomProvider;

pub struct SpotifyProvider {}

impl SpotifyProvider {
    pub fn new(client_id: String, client_secret: String, redirect_url: String) -> CustomProvider {
        CustomProvider::new(
            String::from("https://accounts.spotify.com/authorize"),
            String::from("https://accounts.spotify.com/api/token"),
            client_id,
            client_secret,
            redirect_url,
        )
    }
}
