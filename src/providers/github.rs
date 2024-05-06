use crate::CustomProvider;

pub struct GithubProvider {}

impl GithubProvider {
    pub fn new(client_id: String, client_secret: String, redirect_url: String) -> CustomProvider {
        CustomProvider::new(
            String::from("https://github.com/login/oauth/authorize"),
            String::from("https://github.com/login/oauth/access_token"),
            client_id,
            client_secret,
            redirect_url,
        )
    }
}
