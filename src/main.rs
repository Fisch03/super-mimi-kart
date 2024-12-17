use axum::{
    extract::{
        connect_info::ConnectInfo,
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::{HeaderMap, Response, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};

use std::{
    collections::{HashMap, HashSet},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
};
use tower_http::{compression::CompressionLayer, services::ServeDir};
use tracing::{info, instrument};

use common::ClientId;

struct GameServer {}

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt().with_target(false).finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let app = Router::new();

    let serve_game_dir = ServeDir::new("./static/game").append_index_html_on_directories(true);
    let serve_editor_dir = ServeDir::new("./static/editor").append_index_html_on_directories(true);
    let serve_assets_dir = ServeDir::new("./static/assets").append_index_html_on_directories(false);

    let state = Arc::new(GameServer {});

    let app = app
        .route("/ws", get(ws_handler))
        .nest_service("/editor", serve_editor_dir)
        .nest_service("/assets", serve_assets_dir)
        .fallback_service(serve_game_dir)
        .with_state(state);

    let compression = CompressionLayer::new()
        .gzip(true)
        .zstd(true)
        .br(true)
        .deflate(true);
    let app = app.layer(compression);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    info!(
        "ready! listening on port {}",
        listener.local_addr().unwrap().port()
    );
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    State(state): State<Arc<GameServer>>,
) -> impl IntoResponse {
    let client_id = ClientId::new();
    ws.on_upgrade(move |socket| handle_client(socket, client_id, addr, state))
}

#[instrument(skip(socket, state))]
async fn handle_client(
    socket: WebSocket,
    client_id: ClientId,
    client_addr: SocketAddr,
    state: Arc<GameServer>,
) {
    info!("client connected!");
}
