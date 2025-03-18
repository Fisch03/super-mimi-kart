use common::{
    COUNTDOWN_DURATION, ClientId, ClientMessage, RoundInitParams, ServerMessage, TICKS_PER_SECOND,
    map::Map,
};
use rand::seq::SliceRandom;
use std::{
    fs::File,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    time::Duration,
};
use tokio::{sync::mpsc, time::interval};

use crate::client::Client;

mod client_handler;
use client_handler::{ClientManager, ClientManagerHandle, SendTo, TickResult};

mod game_state;

#[derive(Debug)]
pub struct GameServerHandle {
    next_client_id: AtomicU32,
    clients: ClientManagerHandle,
}

#[derive(Debug)]
pub struct GameServer {
    clients: ClientManagerHandle,
}

#[derive(Debug, Clone)]
pub struct SerializedServerMessage(Arc<[u8]>);

impl SerializedServerMessage {
    pub fn new(msg: ServerMessage) -> Self {
        let msg = msg.to_bytes().expect("message serialization to never fail");
        Self(msg.into())
    }

    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<ServerMessage> for SerializedServerMessage {
    fn from(msg: ServerMessage) -> Self {
        Self::new(msg)
    }
}

impl GameServer {
    pub fn new() -> GameServerHandle {
        let clients = ClientManager::new();

        let server = Self {
            clients: clients.clone(),
        };

        tokio::spawn(server.run());

        GameServerHandle {
            next_client_id: AtomicU32::new(1),
            clients,
        }
    }

    async fn run(self) {
        loop {
            self.clients.await_client().await;

            #[cfg(not(debug_assertions))]
            let wait_time = if clients.len() < 3 { 30 } else { 10 };
            #[cfg(debug_assertions)]
            let wait_time = 5;

            log::info!("waiting {} seconds for players to join", wait_time);
            tokio::time::sleep(Duration::from_secs(wait_time)).await;

            let map_path = "maps/mcircuit/mcircuit.smk";
            log::info!("waiting for players to load map '{:?}'", map_path);

            let load_map = tokio::task::spawn_blocking(move || {
                let file = File::open(map_path)?;
                Map::load(file)
            });

            let map = match load_map.await.unwrap() {
                Ok(map) => map,
                Err(e) => {
                    log::error!("failed to load map '{map_path}': {:?}", e);
                    continue;
                }
            };

            let mut starting_clients = self.clients.load_map(map_path, map).await;
            starting_clients.shuffle(&mut rand::thread_rng());

            for (i, (id, _)) in starting_clients.iter().enumerate() {
                self.clients
                    .send(
                        SendTo::InGameOnly(*id),
                        ServerMessage::StartRound {
                            params: RoundInitParams {
                                client_id: *id,
                                start_pos: i,
                                players: starting_clients.clone(),
                            },
                        },
                    )
                    .await;
            }

            log::info!("waiting for clients to load in");
            tokio::time::sleep(Duration::from_secs(3)).await;

            self.clients
                .send(SendTo::InGameAll, ServerMessage::StartCountdown)
                .await;
            log::info!("waiting for race countdown to finish");
            tokio::time::sleep(Duration::from_secs_f32(COUNTDOWN_DURATION)).await;

            log::info!("round started with players: {:?}", starting_clients);
            self.clients
                .send(SendTo::InGameAll, ServerMessage::StartRace)
                .await;

            let mut tick_interval =
                interval(Duration::from_secs_f64(1.0 / TICKS_PER_SECOND as f64));

            loop {
                tokio::select! {
                    _ = tick_interval.tick() => match self.clients.game_tick().await {
                        TickResult::RaceOver => break,
                        TickResult::NoChange => {}
                    }
                }
            }
        }
    }
}

impl GameServerHandle {
    pub fn allocate_client(&self) -> ClientId {
        // dont give out client id 0 since that is used as an invalid id
        let id = loop {
            let id = self.next_client_id.fetch_add(1, Ordering::Relaxed);
            if id != 0 {
                break id;
            }
        };
        ClientId::new(id)
    }

    pub async fn register_client(
        self: &Arc<Self>,
        client_id: ClientId,
        name: String,
    ) -> mpsc::Receiver<SerializedServerMessage> {
        let (msg_tx, msg_rx) = mpsc::channel(8);

        self.clients
            .add_client(Client::new(client_id, name, msg_tx))
            .await;

        msg_rx
    }

    pub async fn remove_client(&self, client_id: ClientId) {
        self.clients.remove_client(client_id).await;
    }

    pub async fn handle_client_message(self: &Arc<Self>, client_id: ClientId, msg: ClientMessage) {
        // if !matches!(msg, ClientMessage::PlayerUpdate(_)) {
        //     log::info!("received message from client {}: {:?}", client_id, msg);
        // }
        self.clients.handle_client_message(client_id, msg).await;
    }
}
