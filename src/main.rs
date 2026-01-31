use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::{header, Method};
use axum::serve;
use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use dotenv::dotenv;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

mod arbitrum;
mod health;
mod starknet;
mod types;

pub use health::HealthMonitor;
pub use types::{AppState, RollupEvent, RollupStatus};

/// Combined state for API handlers
#[derive(Clone)]
pub struct ApiState {
    pub app: AppState,
    pub health: HealthMonitor,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();
    println!("Starting Rollup Proof Status backend...");

    // Create shared global state
    let app_state = AppState::new();

    // Create health monitor
    let health_monitor = HealthMonitor::new();

    // Spawn the Arbitrum watcher
    let arbitrum_state = app_state.clone();
    let arbitrum_health = health_monitor.clone();
    tokio::spawn(async move {
        if let Err(e) = arbitrum::start_arbitrum_watcher(arbitrum_state, arbitrum_health).await {
            eprintln!("Arbitrum watcher failed: {:?}", e);
        }
    });

    // Spawn the Starknet watcher
    let starknet_state = app_state.clone();
    let starknet_health = health_monitor.clone();
    tokio::spawn(async move {
        if let Err(e) = starknet::start_starknet_watcher(starknet_state, starknet_health).await {
            eprintln!("Starknet watcher failed: {:?}", e);
        }
    });

    // Spawn the health monitor background task
    let monitor_clone = health_monitor.clone();
    tokio::spawn(async move {
        health::start_health_monitor(monitor_clone).await;
    });

    // Combined API state
    let api_state = ApiState {
        app: app_state,
        health: health_monitor,
    };

    // CORS configuration for cross-origin requests from frontend
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE]);

    // Build Axum routes
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(service_health))
        .route("/rollups", get(list_rollups))
        .route("/rollups/arbitrum/status", get(get_arbitrum_status))
        .route("/rollups/starknet/status", get(get_starknet_status))
        .route("/rollups/arbitrum/health", get(get_arbitrum_health))
        .route("/rollups/starknet/health", get(get_starknet_health))
        .route("/rollups/health", get(get_all_health))
        .route("/rollups/stream", get(ws_handler))
        .layer(cors)
        .with_state(api_state);

    // Run Axum HTTP server (use PORT env var for Render, default to 8080)
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr: std::net::SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    println!("API running on http://{}", addr);
    println!("Endpoints:");
    println!("  GET  /                         - Root");
    println!("  GET  /health                   - Service health check");
    println!("  GET  /rollups                  - List supported rollups");
    println!("  GET  /rollups/arbitrum/status  - Arbitrum status");
    println!("  GET  /rollups/starknet/status  - Starknet status");
    println!("  GET  /rollups/arbitrum/health  - Arbitrum health");
    println!("  GET  /rollups/starknet/health  - Starknet health");
    println!("  GET  /rollups/health           - All rollups health");
    println!("  WS   /rollups/stream           - Real-time event stream");

    let listener = TcpListener::bind(addr).await?;
    serve(listener, app).await?;
    Ok(())
}

// ------------------------------------------
// REST Endpoints
// ------------------------------------------

async fn root() -> &'static str {
    "Rollup Proof Status API - Track L2 rollup commitments on Ethereum L1"
}

async fn service_health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "rollup-proof-status"
    }))
}

async fn list_rollups() -> impl IntoResponse {
    Json(serde_json::json!({
        "rollups": [
            {
                "name": "arbitrum",
                "status_endpoint": "/rollups/arbitrum/status",
                "health_endpoint": "/rollups/arbitrum/health",
                "events": ["BatchDelivered", "ProofSubmitted", "ProofVerified"]
            },
            {
                "name": "starknet",
                "status_endpoint": "/rollups/starknet/status",
                "health_endpoint": "/rollups/starknet/health",
                "events": ["StateUpdate", "MessageLog"]
            }
        ]
    }))
}

async fn get_arbitrum_status(State(state): State<ApiState>) -> impl IntoResponse {
    Json(state.app.get_status("arbitrum"))
}

async fn get_starknet_status(State(state): State<ApiState>) -> impl IntoResponse {
    Json(state.app.get_status("starknet"))
}

async fn get_arbitrum_health(State(state): State<ApiState>) -> impl IntoResponse {
    Json(state.health.check_health("arbitrum"))
}

async fn get_starknet_health(State(state): State<ApiState>) -> impl IntoResponse {
    Json(state.health.check_health("starknet"))
}

async fn get_all_health(State(state): State<ApiState>) -> impl IntoResponse {
    Json(serde_json::json!({
        "rollups": state.health.evaluate_all()
    }))
}

// ------------------------------------------
// WebSocket Endpoint
// ------------------------------------------

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<ApiState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_ws(socket, state))
}

async fn handle_ws(mut socket: WebSocket, state: ApiState) {
    let mut rx = state.app.tx.subscribe();

    println!("New WebSocket client connected");

    // Send initial status to the client (including health)
    let statuses = state.app.statuses.read().unwrap().clone();
    let health = state.health.evaluate_all();
    let initial = serde_json::json!({
        "type": "initial",
        "statuses": statuses,
        "health": health
    });
    if let Ok(json_msg) = serde_json::to_string(&initial) {
        let _ = socket.send(Message::Text(json_msg.into())).await;
    }

    // Stream events as they arrive
    while let Ok(event) = rx.recv().await {
        if let Ok(json_msg) = serde_json::to_string(&event) {
            if socket.send(Message::Text(json_msg.into())).await.is_err() {
                break;
            }
        }
    }

    println!("WebSocket client disconnected");
}
