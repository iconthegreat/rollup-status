use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::serve;
use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tokio::net::TcpListener;
use tokio::sync::broadcast;

mod arbitrum;
mod starknet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollupEvent {
    pub rollup: String,
    pub event_type: String,
    pub block_number: u64,
    pub tx_hash: String,
    pub batch_number: Option<String>,
    pub timestamp: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RollupStatus {
    pub latest_batch: Option<String>,
    pub latest_proof: Option<String>,
    pub latest_finalized: Option<String>,
    pub last_updated: Option<u64>,
}

#[derive(Clone)]
pub struct AppState {
    pub statuses: Arc<RwLock<HashMap<String, RollupStatus>>>,
    pub tx: broadcast::Sender<RollupEvent>,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();
    println!("Starting Rollup Proof Status backend...");

    // Creating shared global state
    let (tx, _rx) = broadcast::channel::<RollupEvent>(100);
    let state = AppState {
        statuses: Arc::new(RwLock::new(HashMap::new())),
        tx,
    };

    // Spawning the Arbitrum watcher
    let arbitrum_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = arbitrum::start_arbitrum_watcher(arbitrum_state).await {
            eprintln!("❌ Arbitrum watcher failed: {:?}", e);
        }
    });

    // Spawning the Starknet watcher
    let starknet_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = starknet::start_starnet_watcher(starknet_state).await {
            eprintln!("❌ Starknet watcher failed: {:?}", e);
        }
    });

    // Building Axum routes
    let app = Router::new()
        .route("/", get(root))
        .route("/rollups/arbitrum/status", get(get_arbitrum_status))
        .route("/rollups/starknet/status", get(get_starknet_status))
        .route("/rollups/stream", get(ws_handler))
        .with_state(state.clone());

    // Running Axum HTTP server
    let addr: std::net::SocketAddr = "0.0.0.0:8080".parse().unwrap();
    println!("🌐 API running on http://{}", addr);

    let listener = TcpListener::bind(addr).await?;
    serve(listener, app).await?;
    Ok(())
}

// ------------------------------------------
// Simple REST endpoint for dashboard polling
// ------------------------------------------
async fn root() -> &'static str {
    "Rollup Proof Status API is running ✅"
}

async fn get_arbitrum_status(State(state): State<AppState>) -> impl IntoResponse {
    let statuses = state.statuses.read().unwrap();
    if let Some(status) = statuses.get("arbitrum") {
        Json(status.clone())
    } else {
        Json(RollupStatus::default())
    }
}

async fn get_starknet_status(State(state): State<AppState>) -> impl IntoResponse { 
    let statuses = state.statuses.read().unwrap();
    if let Some(status) = statuses.get("starknet") {
        Json(status.clone())
    } else {
        Json(RollupStatus::default())
    }
}

// ------------------------------------------
// WebSocket endpoint for live streaming
// ------------------------------------------
async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_ws(socket, state))
}

async fn handle_ws(mut socket: WebSocket, state: AppState) {
    let mut rx = state.tx.subscribe();

    println!("🔌 New WebSocket client connected");

    while let Ok(event) = rx.recv().await {
        if let Ok(json_msg) = serde_json::to_string(&event) {
            if socket.send(Message::Text(json_msg.into())).await.is_err() {
                break;
            }
        }
    }

    println!("❌ WebSocket client disconnected");
}
