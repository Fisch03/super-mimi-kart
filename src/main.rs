use axum::{
    Router,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
    routing::get,
};
use futures::{SinkExt, StreamExt};
use std::{net::SocketAddr, sync::Arc};
use tower_http::{compression::CompressionLayer, services::ServeDir};

use common::ClientMessage;

mod server;
use server::{GameServer, GameServerHandle};

mod client;

#[tokio::main]
async fn main() {
    colog::init();

    let app = Router::new();

    let serve_game_dir = ServeDir::new("./static/game").append_index_html_on_directories(true);
    let serve_editor_dir = ServeDir::new("./static/editor").append_index_html_on_directories(true);
    let serve_assets_dir = ServeDir::new("./static/assets").append_index_html_on_directories(false);
    let serve_maps_dir = ServeDir::new("./static/maps").append_index_html_on_directories(false);

    let server = Arc::new(GameServer::new());

    let app = app
        .route("/ws", get(ws_handler))
        .nest_service("/editor", serve_editor_dir)
        .nest_service("/assets", serve_assets_dir)
        .nest_service("/maps", serve_maps_dir)
        .fallback_service(serve_game_dir)
        .with_state(server);

    let compression = CompressionLayer::new()
        .gzip(true)
        .zstd(true)
        .br(true)
        .deflate(true);
    let app = app.layer(compression);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    log::info!(
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
    State(server): State<Arc<GameServerHandle>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_client(socket, server))
}

async fn handle_client(socket: WebSocket, server: Arc<GameServerHandle>) {
    let client_id = server.allocate_client();
    log::info!("({}) client connecting", client_id);

    let (mut socket_tx, mut socket_rx) = socket.split();

    let client_name = if let Some(Ok(Message::Binary(msg))) = socket_rx.next().await {
        match ClientMessage::from_bytes(&msg) {
            Ok(ClientMessage::Register { name }) => name,
            Ok(_) => {
                log::warn!("client didnt register before sending data");
                return;
            }
            Err(e) => {
                log::warn!("client sent invalid register message: {}", e);
                return;
            }
        }
    } else {
        log::warn!("client didnt send register message");
        return;
    };
    log::info!(
        "({}) client connected with name '{}'",
        client_id,
        client_name
    );

    let mut msg_rx = server.register_client(client_id, client_name).await;

    let rx_task = {
        let server = server.clone();
        tokio::spawn(async move {
            while let Some(Ok(Message::Binary(msg))) = socket_rx.next().await {
                let msg = match ClientMessage::from_bytes(&msg) {
                    Ok(msg) => msg,
                    Err(e) => {
                        log::warn!("client sent invalid message: {}", e);
                        continue;
                    }
                };

                if matches!(msg, ClientMessage::Register { .. }) {
                    log::warn!("client tried to register again");
                    continue;
                }

                server.handle_client_message(client_id, msg).await;
            }
        })
    };

    let tx_task = tokio::spawn(async move {
        while let Some(msg) = msg_rx.recv().await {
            if let Err(e) = socket_tx.send(Message::Binary(msg.bytes().to_vec())).await {
                log::warn!("error sending message to client: {}", e);
                break;
            }
        }
    });

    tokio::select! {
        _ = rx_task => (),
        _ = tx_task => ()
    }

    server.remove_client(client_id).await;
    log::info!("({}) client disconnected", client_id);
}
