use crate::CustomProvider;

pub struct GoogleProvider {}

impl GoogleProvider {
    pub fn new(client_id: String, client_secret: String, redirect_url: String) -> CustomProvider {
        CustomProvider::new(
            String::from("https://twitter.com/i/oauth2/authorize"),
            String::from("https://api.twitter.com/2/oauth2/token"),
            client_id,
            client_secret,
            redirect_url,
        )
    }
}
