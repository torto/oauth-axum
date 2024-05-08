use async_trait::async_trait;
use oauth2::http::Error;
use std::future::Future;

use oauth2::reqwest::async_http_client;
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl,
    Scope, TokenUrl,
};
use oauth2::{AuthorizationCode, PkceCodeVerifier, TokenResponse};

#[derive(Clone)]
pub struct CustomOpenIdProvider {
    pub auth_url: String,
    pub token_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
    pub user_url: String,
    pub state: Option<StateAuth>,
}

#[derive(Clone)]
pub enum MethodExecute {
    DB,
    MEMORY,
}

#[derive(Clone, Debug)]
pub struct StateAuth {
    pub url_generated: Option<String>,
    pub state: String,
    pub verifier: String,
}

impl CustomOpenIdProvider {
    pub fn new(
        auth_url: String,
        token_url: String,
        user_url: String,
        client_id: String,
        client_secret: String,
        redirect_url: String,
    ) -> Self {
        CustomOpenIdProvider {
            auth_url,
            token_url,
            client_id,
            client_secret,
            redirect_url,
            user_url,
            state: None,
        }
    }
}

/// OAuthClient is the main struct of the lib, it will handle all the connection with the provider
#[async_trait]
pub trait OpenIdClient {
    fn get_client(&self) -> BasicClient;

    /// Get fields data from generated URL
    /// # Return
    /// StateAuth - The state, verifier and url_generated
    fn get_state(&self) -> Option<StateAuth>;

    /// Genrate the URL to redirect the user to the provider
    /// # Arguments
    /// * `scopes` - Vec<String> - The scopes that you want to access in the provider
    /// * `save` - F - The function that will use to save your state in the db/memory
    async fn generate_url<F, Fut>(
        mut self,
        scopes: Vec<String>,
        save: F,
    ) -> Result<Box<Self>, Error>
    where
        F: FnOnce(StateAuth) -> Fut + Send,
        Fut: Future<Output = ()> + Send;

    /// Generate the token from the code and verifier
    /// # Arguments
    /// * `code` - String - The code that the provider will return after the user accept the auth
    /// * `verifier` - String - The verifier that was generated in the first step
    /// # Return
    /// The token generated
    async fn generate_token(&self, code: String, verifier: String) -> String;
}

#[async_trait]
impl OpenIdClient for CustomOpenIdProvider {
    fn get_client(&self) -> BasicClient {
        BasicClient::new(
            ClientId::new(self.client_id.clone()),
            Some(ClientSecret::new(self.client_secret.clone())),
            AuthUrl::new(self.auth_url.clone()).unwrap(),
            Some(TokenUrl::new(self.token_url.clone()).unwrap()),
        )
        .set_redirect_uri(RedirectUrl::new(self.redirect_url.clone()).unwrap())
    }

    fn get_state(&self) -> Option<StateAuth> {
        self.state.clone()
    }

    async fn generate_url<F, Fut>(
        mut self,
        scopes: Vec<String>,
        save: F,
    ) -> Result<Box<Self>, Error>
    where
        F: FnOnce(StateAuth) -> Fut + Send,
        Fut: Future<Output = ()> + Send,
    {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let binding = self.get_client();
        let (auth_url, csrf_token) = binding
            .authorize_url(CsrfToken::new_random)
            .add_scopes(scopes.into_iter().map(Scope::new).collect::<Vec<Scope>>())
            .set_pkce_challenge(pkce_challenge)
            .url();

        let state = StateAuth {
            url_generated: Some(auth_url.to_string()),
            state: csrf_token.secret().to_string(),
            verifier: pkce_verifier.secret().to_string(),
        };

        self.state = Some(state.clone());
        save(state).await;

        Ok(Box::new(self.clone()))
    }

    async fn generate_token(&self, code: String, verifier: String) -> String {
        let token = self
            .get_client()
            .exchange_code(AuthorizationCode::new(code.clone()))
            .set_pkce_verifier(PkceCodeVerifier::new(verifier.clone()))
            .request_async(async_http_client)
            .await
            .unwrap();
        token.access_token().secret().to_string()
    }
}
