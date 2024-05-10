use crate::CustomProvider;

pub struct MicrosoftProvider {}

impl MicrosoftProvider {
    /// Create a new MicrosoftProvider
    ///
    /// # Arguments
    /// * `tenant_id` - The tenant id - Check Microsfot docmentation for more information: https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-auth-code-flow#request-an-authorization-code
    /// * `client_id` - The client id
    /// * `client_secret` - The client secret
    /// * `redirect_url` - The redirect url
    ///
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
