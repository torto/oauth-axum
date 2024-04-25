use std::sync::Arc;

use oauth2::reqwest::async_http_client;
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl,
    Scope, TokenUrl,
};
use oauth2::{AuthorizationCode, PkceCodeVerifier, TokenResponse};

use crate::memory_db::AxumState;

#[derive(Clone, serde::Deserialize)]
pub struct QueryAxumCallback {
    pub code: String,
    pub state: String,
}

#[derive(Clone)]
pub enum Provider {
    Google,
    Github,
}

#[derive(Clone)]
pub enum MethodExecute {
    DB,
    MEMORY,
}

#[derive(Clone)]
struct Connector {
    auth_url: String,
    token_url: String,
    client_id: String,
    client_secret: String,
    redirect_url: String,
}

#[derive(Clone)]
pub struct OAuthClient {
    connector: Connector,
    method: MethodExecute,
    memory_state: Option<Arc<AxumState>>,
    pub url_generated: Option<String>,
}

impl OAuthClient {
    pub fn new(
        provider: Provider,
        client_id: String,
        client_secret: String,
        redirect_url: String,
    ) -> Self {
        OAuthClient {
            connector: match provider.clone() {
                Provider::Google => Connector {
                    auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
                    token_url: "https://oauth2.googleapis.com/token".to_string(),
                    client_id,
                    client_secret,
                    redirect_url,
                },

                Provider::Github => Connector {
                    auth_url: "https://github.com/login/oauth/authorize".to_string(),
                    token_url: "https://github.com/login/oauth/access_token".to_string(),
                    client_id,
                    client_secret,
                    redirect_url,
                },
            },
            memory_state: None,
            method: MethodExecute::MEMORY,
            url_generated: None,
        }
    }

    fn get_client(&self) -> BasicClient {
        BasicClient::new(
            ClientId::new(self.connector.client_id.clone()),
            Some(ClientSecret::new(self.connector.client_secret.clone())),
            AuthUrl::new(self.connector.auth_url.clone()).unwrap(),
            Some(TokenUrl::new(self.connector.token_url.clone()).unwrap()),
        )
        .set_redirect_uri(RedirectUrl::new(self.connector.redirect_url.clone()).unwrap())
        .clone()
    }

    pub fn set_memory_state(mut self, state: Arc<AxumState>) -> Self {
        self.memory_state = Some(state);
        self.clone()
    }

    pub fn set_method(mut self, method: MethodExecute) -> Self {
        self.method = method;
        self.clone()
    }

    pub fn set_redirect_url(&mut self, redirect_url: String) {
        self.connector.redirect_url = redirect_url;
    }

    pub fn set_url_generated(mut self, url: String) -> Self {
        self.url_generated = Some(url);
        self.clone()
    }

    pub fn generate_url(self, scopes: Vec<String>) -> Self {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let binding = self.get_client();
        let (auth_url, csrf_token) = binding
            .authorize_url(CsrfToken::new_random)
            .add_scopes(scopes.into_iter().map(Scope::new).collect::<Vec<Scope>>())
            .set_pkce_challenge(pkce_challenge)
            .url();

        match self.method {
            MethodExecute::MEMORY => {
                self.memory_state.as_ref().unwrap().set(
                    csrf_token.clone().secret().to_string(),
                    pkce_verifier.secret().to_string(),
                );
                println!(
                    "Generated URL: {:?}",
                    self.memory_state.as_ref().unwrap().get_all_items()
                );
            }
            MethodExecute::DB => {
                // let _session = insert_provider_session(
                //     &state.db,
                //     ProviderSession {
                //         id: uuid::Uuid::new_v4(),
                //         state: csrf_token.clone().secret().to_string(),
                //         verifier: pkce_verifier.secret().to_string(),
                //         created_at: None,
                //     },
                // )
                // .await
                // .unwrap();
            }
        }
        self.set_url_generated(auth_url.to_string())
    }

    pub async fn generate_token(&self, state: String, code: String) -> String {
        let binding = self.get_client();
        let token = binding
            .exchange_code(AuthorizationCode::new(code.clone()))
            .set_pkce_verifier(PkceCodeVerifier::new(
                self.memory_state.as_ref().unwrap().get(state).unwrap(),
            ))
            .request_async(async_http_client)
            .await
            .unwrap();
        token.access_token().secret().to_string()
    }
}
