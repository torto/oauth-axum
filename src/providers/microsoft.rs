use crate::CustomProvider;

pub struct MicrosoftProvider {}

impl MicrosoftProvider {
    pub fn new(
        tenant_id: String,
        client_id: String,
        client_secret: String,
        redirect_url: String,
    ) -> CustomProvider {
        let base_url = String::from(
            "https://login.microsoftonline.com/".to_string() + tenant_id.as_str() + "/oauth2/v2.0",
        );
        CustomProvider::new(
            String::from(base_url.clone() + "/authorize"),
            String::from(base_url + "/token"),
            client_id,
            client_secret,
            redirect_url,
        )
    }
}
