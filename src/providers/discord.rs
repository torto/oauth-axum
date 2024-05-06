use crate::CustomProvider;

pub struct DiscordProvider {}

impl DiscordProvider {
    pub fn new(client_id: String, client_secret: String, redirect_url: String) -> CustomProvider {
        CustomProvider::new(
            String::from("https://discord.com/oauth2/authorize"),
            String::from("https://discord.com/api/oauth2/token"),
            client_id,
            client_secret,
            redirect_url,
        )
    }
}
