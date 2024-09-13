mod utils;
use std::sync::Arc;

use axum::extract::Query;
use axum::Router;
use axum::{routing::get, Extension};
use oauth_axum::providers::github::GithubProvider;
use oauth_axum::{CustomProvider, OAuthClient};

use crate::utils::memory_db_util::AxumState;

#[derive(Clone, serde::Deserialize)]
pub struct QueryAxumCallback {
    pub code: String,
    pub state: String,
}

#[tokio::main]
async fn main() {
    dotenv::from_filename("examples/.env").ok();
    println!("Starting server...");

    let state = Arc::new(AxumState::new());
    let app = Router::new()
        .route("/", get(create_url))
        .route("/api/v1/github/callback", get(callback))
        .layer(Extension(state.clone()));

    println!("🚀 Server started successfully");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn get_client() -> CustomProvider {
    GithubProvider::new(
        std::env::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID must be set"),
        std::env::var("GITHUB_SECRET").expect("GITHUB_SECRET must be set"),
        "http://localhost:3000/api/v1/github/callback".to_string(),
    )
}

pub async fn create_url(Extension(state): Extension<Arc<AxumState>>) -> String {
    let state_oauth = get_client()
        .generate_url(Vec::from(["read:user".to_string()]), |state_e| async move {
            state.set(state_e.state, state_e.verifier);
        })
        .await
        .ok()
        .unwrap()
        .state
        .unwrap();

    state_oauth.url_generated.unwrap()
}

pub async fn callback(
    Extension(state): Extension<Arc<AxumState>>,
    Query(queries): Query<QueryAxumCallback>,
) -> String {
    println!("{:?}", state.clone().get_all_items());
    let item = state.get(queries.state.clone());
    get_client()
        .generate_token(queries.code, item.unwrap())
        .await
        .ok()
        .unwrap()
}
