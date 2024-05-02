use crate::CustomProvider;

pub struct GoogleProvider {}

impl GoogleProvider {
    pub fn new(client_id: String, client_secret: String, redirect_url: String) -> CustomProvider {
        CustomProvider::new(
            String::from("https://accounts.google.com/o/oauth2/v2/auth"),
            String::from("https://oauth2.googleapis.com/token"),
            client_id,
            client_secret,
            redirect_url,
        )
    }
}
