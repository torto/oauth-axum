use std::sync::Arc;

use axum::extract::{Query, State};
use axum::routing::get;
use axum::Router;
use oauth_axum::providers::github::GithubProvider;
use oauth_axum::{CustomProvider, OAuthClient};

#[derive(Clone, serde::Deserialize)]
pub struct QueryAxumCallback {
    pub code: String,
    pub state: String,
}

use tokio_postgres::{Client, NoTls};

#[tokio::main]
async fn main() {
    dotenv::from_filename("examples/.env").ok();
    println!("Starting server...");

    let (client, connection) = tokio_postgres::connect(
        "postgresql://admin:password123@172.18.0.2:5432/rust_hs256",
        NoTls,
    )
    .await
    .unwrap();

    //     let _rows = client
    //         .query_opt(
    //             r#"CREATE TABLE IF NOT EXISTS
    //     "oauth" (
    //     id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
    //     state VARCHAR(255) NOT NULL,
    //     verifier VARCHAR(255) NOT NULL,
    // );"#,
    //             &[],
    //         )
    //         .await
    //         .unwrap();

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let app = Router::new()
        .route("/", get(create_url))
        .route("/api/v1/github/callback", get(callback))
        .with_state(Arc::new(client));

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

pub async fn create_url(State(state): State<Arc<Client>>) -> String {
    let state_oauth = get_client()
        .generate_url(Vec::from(["read:user".to_string()]), |state_e| async move {
            state
                .execute(
                    "INSERT INTO oauth (state, verifier) VALUES ($1, $2)",
                    &[&state_e.state, &state_e.verifier],
                )
                .await
                .unwrap();
        })
        .await
        .ok()
        .unwrap()
        .state
        .unwrap();

    state_oauth.url_generated.unwrap()
}

pub async fn callback(
    State(state): State<Arc<Client>>,
    Query(queries): Query<QueryAxumCallback>,
) -> String {
    let row = state
        .query_one(
            "SELECT verifier FROM oauth WHERE state LIKE $1",
            &[&queries.state],
        )
        .await
        .unwrap();

    get_client()
        .generate_token(queries.code, row.get(0))
        .await
        .ok()
        .unwrap()
}
