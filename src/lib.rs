//! # oauth-axum
//!
//! This crate is a wrapper of oauth2 lib, but it has all the provider configuration done, making it easy to implement in your Axum project.
//! The intention is to add all providers from this list: https://en.wikipedia.org/wiki/List_of_OAuth_providers that have oauth2 available.
//!
//! # Usage
//!
//! To use it, it's very simple. Just create a new instance of some provider:
//!
//! - CustomProvider
//! - GithubProvider
//! - DiscordProvider
//! - TwitterProvider
//! - GoogleProvider
//! - FacebookProvider
//!
//! in your project, pass to the ```new``` function:
//!
//!   - **client_id:** Unique ID from the app created in your provider
//!   - **secret_id:** Secret token from your app inside the provider, this token needs to be hidden from the users
//!   - **redirect_url:** URL from your backend that will accept the return from the provider
//!
//!   If you are using **``CustomProvider``** you need to pass:
//!
//!   - **auth_url:** URL from your provider that is used to get the permission of your app access user account
//!   - **token_url:** URL that is used to generate the auth token
//!
//! The structure of this project is separated into two steps:
//!
//! ### 1. Generate the URL
//!
//! This step will create a URL to redirect the user to the provider to execute the authorization of your app access to the user info.
//!
//! The URL has this format (Github example): https://github.com/login/oauth/authorize?response_type=code&client_id={CLIENT_ID}&state={RANDOM_STATE}&code_challenge={RANDOM_STATE}&code_challenge_method=S256&redirect_uri={REDIRECT_URL}&scope={SCOPES}
//!
//! This step is important because that will generate the VERIFIER field, it is needed to save in some place (memory, db...) with the state field, the state will be your ID to get the verifier in the second step.
//!
//! ### 2. Callback URL
//!
//! After the user accepts the auth from the provider, it will redirect the user to the specific URL that you added in the config of the provider ``redirect_url``, and is important to remember that the same URL should be set in the oauth-axum params, if it is not the same an error will happen.
//! This redirect will have two query parameters, CODE and STATE, we need to generate a token from the code and verifier fields, which is the reason that in the first step, you need to save the verifier and state together.
//! After that, you will have a token to access the API in the provider.
//!
//! ## Example
//!
//! This method is for a small project that will run in one unique instance of Axum. It saves the state and verifier in memory, which can be accessible in the callback URL call.
//!
//! ```rust
//! mod utils;
//! use std::sync::Arc;
//!
//! use axum::extract::Query;
//! use axum::Router;
//! use axum::{routing::get, Extension};
//! use oauth_axum::providers::twitter::TwitterProvider;
//! use oauth_axum::{CustomProvider, OAuthClient};
//!
//! use crate::utils::memory_db_util::AxumState;
//!
//! #[derive(Clone, serde::Deserialize)]
//! pub struct QueryAxumCallback {
//!     pub code: String,
//!     pub state: String,
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     dotenv::from_filename("examples/.env").ok();
//!     println!("Starting server...");
//!
//!     let state = Arc::new(AxumState::new());
//!     let app = Router::new()
//!         .route("/", get(create_url))
//!         .route("/api/v1/twitter/callback", get(callback))
//!         .layer(Extension(state.clone()));
//!
//!     println!("🚀 Server started successfully");
//!     let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
//!         .await
//!         .unwrap();
//!     axum::serve(listener, app).await.unwrap();
//! }
//!
//! fn get_client() -> CustomProvider {
//!     TwitterProvider::new(
//!         std::env::var("TWITTER_CLIENT_ID").expect("TWITTER_CLIENT_ID must be set"),
//!         std::env::var("TWITTER_SECRET").expect("TWITTER_SECRET must be set"),
//!         "http://localhost:3000/api/v1/twitter/callback".to_string(),
//!     )
//! }
//!
//! pub async fn create_url(Extension(state): Extension<Arc<AxumState>>) -> String {
//!     let state_oauth = get_client()
//!         .generate_url(
//!             Vec::from(["users.read".to_string()]),
//!             |state_e| async move {
//!                 //SAVE THE DATA IN THE DB OR MEMORY
//!                 //state should be your ID
//!                 state.set(state_e.state, state_e.verifier);
//!             },
//!         )
//!         .await
//!         .unwrap()
//!         .state
//!         .unwrap();
//!
//!     state_oauth.url_generated.unwrap()
//! }
//!
//! pub async fn callback(
//!     Extension(state): Extension<Arc<AxumState>>,
//!     Query(queries): Query<QueryAxumCallback>,
//! ) -> String {
//!     println!("{:?}", state.clone().get_all_items());
//!     // GET DATA FROM DB OR MEMORY
//!     // get data using state as ID
//!     let item = state.get(queries.state.clone());
//!     get_client()
//!         .generate_token(queries.code, item.unwrap())
//!         .await
//! }
//! ```
//!
//! # Next Steps of Development
//!
//! - Add all tests
//! - Add more Providers
//!

pub mod providers;

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
pub struct CustomProvider {
    pub auth_url: String,
    pub token_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
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

impl CustomProvider {
    pub fn new(
        auth_url: String,
        token_url: String,
        client_id: String,
        client_secret: String,
        redirect_url: String,
    ) -> Self {
        CustomProvider {
            auth_url,
            token_url,
            client_id,
            client_secret,
            redirect_url,
            state: None,
        }
    }
}

/// OAuthClient is the main struct of the lib, it will handle all the connection with the provider
#[async_trait]
pub trait OAuthClient {
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
impl OAuthClient for CustomProvider {
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
