use crate::CustomProvider;

pub struct FacebookProvider {}

impl FacebookProvider {
    pub fn new(client_id: String, client_secret: String, redirect_url: String) -> CustomProvider {
        CustomProvider::new(
            String::from("https://www.facebook.com/v19.0/dialog/oauth"),
            String::from("https://graph.facebook.com/v19.0/oauth/access_token"),
            client_id,
            client_secret,
            redirect_url,
        )
    }
}
