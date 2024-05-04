//! # oauth-axum crate
//!
//! This crate is a wrapper of oauth2 lib, but it has all the provider configuration done, making it easy to implement in your Axum project.
//! The intention is to add all providers from this list: https://en.wikipedia.org/wiki/List_of_OAuth_providers that have oauth2 available.
//!
//! # Usage
//!
//! To use it, it's very simple. Just create a new instance of `OAuthClient` in your project, add the client_id, secret_id, and redirect_url from the provider, and you can connect with it.
//! This crate has two methods to deal with the authorization: Memory and DB.
//!
//! The structure of this project is separated into two steps:
//!
//! ### 1. Generate the URL
//!
//! This step will create a URL to redirect the user to the provider to execute the authorization of your app access to the user info.
//!
//! The URL has this format (Github example): https://github.com/login/oauth/authorize?response_type=code&client_id={CLIENT_ID}&state={RANDOM_STATE}&code_challenge={RANDOM_STATE}&code_challenge_method=S256&redirect_uri={REDIRECT_URL}&scope={SCOPES}
//!
//! This step is important because that will generate the VERIFIER field, we need to save it in some place (memory, db...) with the state field, the state will be your id to get the verifier in the second step.
//!
//! ### 2. Callback URL
//!
//! After the user accepts the auth from the provider, it will redirect the user to the specific URL that you added in the config of the provider, and is important to remember that the same URL should be set in the oauth-axum params, if it is not the same an error will happens.
//! This redirect will have two queries parameters, CODE and STATE, we need to generate a token from the code and verifier fields, which is the reason that in the first step, you need to save the verifier and state together.
//! After that, you will have a token to access the API in the provider.
//!
//! ## Memory Method
//!
//! This method is for a small project that will run in one unique instance of Axum. It saves the state and verifier in memory, which can be accessible in the callback URL call.
//!
//! ```rust
//! use std::sync::Arc;
//!
//! use axum::extract::Query;
//! use axum::Router;
//! use axum::{routing::get, Extension};
//! use oauth_axum::client::{OAuthClient, Provider};
//! use oauth_axum::memory_db::AxumState;
//!
//! #[derive(Clone, serde::Deserialize)]
//! pub struct QueryAxumCallback {
//!     pub code: String,
//!     pub state: String,
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     println!("Starting server...");
//!
//!     // Struct that will hadle the memory save in the axum,
//!     // you can pass it as a state or Extension,
//!     // in that example is using Extension
//!     let state = Arc::new(AxumState::new());
//!
//!     //create two router, the first one to generate the URL
//!     // the second one to generate the token
//!     let app = Router::new()
//!         .route("/", get(create_url))
//!         .route("/api/v1/github/callback", get(callback))
//!         // is important to set the state/extension using the oauth_axum::AxumState
//!         .layer(Extension(state.clone()));
//!
//!     println!("🚀 Server started successfully");
//!     let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
//!         .await
//!         .unwrap();
//!     axum::serve(listener, app).await.unwrap();
//! }
//!
//! // method to create a instace of OAuthClient all the time that it necessary
//! fn get_client() -> OAuthClient {
//!     OAuthClient::new(
//!         Provider::Github, //! Pass the provider that you want to connect
//!         "CLIENT_ID".to_string(),
//!         "CLIENT_SECRET".to_string(),
//!         "URL_CALLBACK".to_string(),
//!     )
//! }
//!
//! pub async fn create_url(Extension(state): Extension<Arc<AxumState>>) -> String {
//!     //get the client with default method memory
//!     get_client()
//!     // set to the lib the handle of memory oauth_axum::AxumState,
//!         .set_memory_state(Arc::clone(&state))
//!         // Pass all the scopes, each provider has different scopes
//!         .generate_url(Vec::from(["read:user".to_string()]))
//!         //get the url inside the struct, the variable url_generated will
//!         //be avalible only after execute the method generate_url
//!         .url_generated
//!         .unwrap_or_default()
//! }
//!
//! pub async fn callback(
//!     Extension(state): Extension<Arc<AxumState>>,
//!     Query(queries): Query<QueryAxumCallback>,
//! ) -> String {
//!     //get the client
//!     get_client()
//!     // set to the lib the handle of memory oauth_axum::AxumState,
//!         .set_memory_state(Arc::clone(&state))
//!         // generate the token passing the code and state to the lib
//!         .generate_token_memory(queries.code, queries.state)
//!         .await
//! }
//! ```
//!
//! ## DB Method
//! This method is used to a big axum project, that has more than one instance and access in a DB, in this example, I will use a Postgres example:
//!
//! ```rust
//! use std::sync::Arc;
//!
//! use axum::extract::{Query, State};
//! use axum::routing::get;
//! use axum::Router;
//! use oauth_axum::{MethodExecute, OAuthClient, Provider};
//!
//! #[derive(Clone, serde::Deserialize)]
//! pub struct QueryAxumCallback {
//!     pub code: String,
//!     pub state: String,
//! }
//!
//! use tokio_postgres::{Client, NoTls};
//!
//! #[tokio::main]
//! async fn main() {
//!     println!("Starting server...");
//!     // connection with DB
//!     let (client, connection) = tokio_postgres::connect(
//!         "postgresql://admin:password123@172.18.0.2:5432/rust_hs256",
//!         NoTls,
//!     )
//!     .await
//!     .unwrap();
//!     // Error handle of db
//!     tokio::spawn(async move {
//!         if let Err(e) = connection.await {
//!             eprintln!("connection error: {}", e);
//!         }
//!     });
//!
//!     //create two router, the first one to generate the URL
//!     // the second one to generate the token
//!     let app = Router::new()
//!         .route("/", get(create_url))
//!         .route("/api/v1/github/callback", get(callback))
//!         .layer(Extension(state.clone()))
//!         .with_state(Arc::new(client));
//!
//!     println!("🚀 Server started successfully");
//!     let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
//!         .await
//!         .unwrap();
//!     axum::serve(listener, app).await.unwrap();
//! }
//!
//! fn get_client() -> OAuthClient {
//!     OAuthClient::new(
//!         Provider::Github,
//!         "CLIENT_ID".to_string(),
//!         "CLIENT_SECRET".to_string(),
//!         "URL_CALLBACK".to_string(),
//!     )
//!     //SET THE METHOD TO DB, the default method is Memory
//!     .set_method(MethodExecute::DB)
//! }
//!
//! pub async fn create_url(State(state): State<Arc<Client>>) -> String {
//!     //generate the url
//!     let state_db = get_client()
//!         .generate_url(Vec::from(["read:user".to_string()]))
//!         .db_state
//!         .unwrap();
//!
//!     //save in the db the state and verifier
//!     state
//!         .execute(
//!             "INSERT INTO oauth (state, verifier) VALUES ($1, $2)",
//!             &[&state_db.state, &state_db.verifier],
//!         )
//!         .await
//!         .unwrap();
//!
//!     state_db.url_generated.unwrap_or_default()
//! }
//!
//! pub async fn callback(
//!     State(state): State<Arc<Client>>,
//!     Query(queries): Query<QueryAxumCallback>,
//! ) -> String {
//!     //get in db using state as ID to have access to the verifier
//!     let row = state
//!         .query_one(
//!             "SELECT verifier FROM oauth WHERE state LIKE $1",
//!             &[&queries.state],
//!         )
//!         .await
//!         .unwrap();
//!     //! Generate the token using the code from the query parameter and verifier from db
//!     get_client()
//!         .generate_token_db(queries.code, row.get(0))
//!         .await
//! }
//!
//! ```
//!
//!
//!  # Next Steps of Development
//!
//! - Add all tests
//! - Add more Providers

pub mod memory_db;
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

    /// Get the state and verifier from the memory
    /// # Return
    /// A tuple with the state and verifier
    /// # Example
    /// ```rust
    /// let (state, verifier) = get_client().get_memory_state();
    /// ```
    fn get_state(&self) -> Option<StateAuth>;

    /// Genrate the URL to redirect the user to the provider
    /// # Arguments
    /// * `scopes` - Vec<String> - The scopes that you want to access in the provider
    /// # Return
    /// A new instance of OAuthClient with the URL generated
    async fn generate_url<F, Fut>(
        mut self,
        scopes: Vec<String>,
        save: F,
    ) -> Result<Box<Self>, Error>
    where
        F: FnOnce(StateAuth) -> Fut + Send,
        Fut: Future<Output = ()> + Send;

    /// Generate the token from the code and verifier using db method
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

    /// Get the state and verifier from the memory
    /// # Return
    /// A tuple with the state and verifier
    /// # Example
    /// ```rust
    /// let (state, verifier) = get_client().get_memory_state();
    /// ```
    fn get_state(&self) -> Option<StateAuth> {
        self.state.clone()
    }

    /// Genrate the URL to redirect the user to the provider
    /// # Arguments
    /// * `scopes` - Vec<String> - The scopes that you want to access in the provider
    /// # Return
    /// A new instance of OAuthClient with the URL generated
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
        println!("State: ANTES");
        save(state).await;
        println!("State: FINAL");

        Ok(Box::new(self.clone()))
    }

    /// Generate the token from the code and verifier using db method
    /// # Arguments
    /// * `code` - String - The code that the provider will return after the user accept the auth
    /// * `verifier` - String - The verifier that was generated in the first step
    /// # Return
    /// The token generated
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
