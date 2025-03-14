use common::{ActiveItem, ClientId, ClientMessage, PickupKind, ServerMessage, map::Map};
use std::collections::HashMap;
use tokio::{
    sync::{mpsc, oneshot},
    task::{self, JoinHandle},
    time::{self, Duration},
};

use super::{SerializedServerMessage, game_state::GameState};
use crate::client::Client;

#[derive(Debug)]
pub struct ClientManager {
    rx: mpsc::Receiver<ClientManagerCommand>,
    tx: mpsc::Sender<ClientManagerCommand>,

    waiting_clients: Vec<Client>,
    loading_clients: Vec<Client>,
    clients: HashMap<ClientId, Client>,

    waiting_for_clients: Option<oneshot::Sender<()>>,
    loading_task: Option<LoadingTask>,

    game_state: GameState,
}

#[derive(Debug)]
struct LoadingTask {
    join_handle: JoinHandle<()>,
    result_tx: oneshot::Sender<Vec<(ClientId, String)>>,
}

#[derive(Debug, Clone)]
pub struct ClientManagerHandle {
    tx: mpsc::Sender<ClientManagerCommand>,
}

enum ClientManagerCommand {
    AwaitClient(oneshot::Sender<()>),
    AddClient(Client),
    RemoveClient(ClientId),

    HandleClientMessage(ClientId, ClientMessage),
    SendServerMessage(SendTo, ServerMessage),

    LoadMap {
        map_path: String,
        map: Map,
        result_tx: oneshot::Sender<Vec<(ClientId, String)>>,
    },
    GameTick(oneshot::Sender<TickResult>),

    // internal
    LoadTimeout,
    PickupRespawn {
        kind: PickupKind,
        index: usize,
    },
}

#[derive(Debug)]
pub enum SendTo {
    All,
    LoadingAll,
    InGameAll,
    InGameExcept(ClientId),
    InGameOnly(ClientId),
}

pub enum TickResult {
    NoChange,
    RaceOver,
}

impl ClientManagerHandle {
    pub async fn await_client(&self) {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(ClientManagerCommand::AwaitClient(tx))
            .await
            .unwrap();
        rx.await.unwrap();
    }

    pub async fn add_client(&self, client: Client) {
        self.tx
            .send(ClientManagerCommand::AddClient(client))
            .await
            .unwrap();
    }

    pub async fn remove_client(&self, id: ClientId) {
        self.tx
            .send(ClientManagerCommand::RemoveClient(id))
            .await
            .unwrap();
    }

    pub async fn send(&self, to: SendTo, msg: ServerMessage) {
        self.tx
            .send(ClientManagerCommand::SendServerMessage(to, msg))
            .await
            .unwrap();
    }

    pub async fn handle_client_message(&self, id: ClientId, msg: ClientMessage) {
        self.tx
            .send(ClientManagerCommand::HandleClientMessage(id, msg))
            .await
            .unwrap();
    }

    pub async fn load_map(&self, map_name: &str, map: Map) -> Vec<(ClientId, String)> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(ClientManagerCommand::LoadMap {
                map_path: map_name.into(),
                map,
                result_tx: tx,
            })
            .await
            .unwrap();

        rx.await.unwrap()
    }

    pub async fn game_tick(&self) -> TickResult {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(ClientManagerCommand::GameTick(tx))
            .await
            .unwrap();

        rx.await.unwrap()
    }

    async fn pickup_respawn(&self, kind: PickupKind, index: usize) {
        self.tx
            .send(ClientManagerCommand::PickupRespawn { kind, index })
            .await
            .unwrap();
    }
}

impl ClientManager {
    pub fn new() -> ClientManagerHandle {
        let (tx, rx) = mpsc::channel(128);

        let manager = Self {
            rx,
            tx: tx.clone(),

            waiting_clients: Vec::new(),
            loading_clients: Vec::new(),
            clients: HashMap::new(),

            loading_task: None,
            waiting_for_clients: None,

            game_state: GameState::default(),
        };

        tokio::spawn(manager.run());

        ClientManagerHandle { tx }
    }

    pub fn make_handle(&self) -> ClientManagerHandle {
        ClientManagerHandle {
            tx: self.tx.clone(),
        }
    }

    async fn run(mut self) {
        while let Some(cmd) = self.rx.recv().await {
            match cmd {
                ClientManagerCommand::AwaitClient(tx) => {
                    if self.waiting_clients.is_empty() && self.loading_clients.is_empty() {
                        self.waiting_for_clients = Some(tx);
                    } else {
                        tx.send(()).unwrap();
                    }
                }
                ClientManagerCommand::AddClient(client) => self.add_client(client).await,
                ClientManagerCommand::RemoveClient(id) => self.remove_client(id).await,

                ClientManagerCommand::HandleClientMessage(id, msg) => {
                    self.handle_client_message(id, msg).await
                }
                ClientManagerCommand::SendServerMessage(to, msg) => self.send(to, msg).await,

                ClientManagerCommand::LoadMap {
                    map_path,
                    map,
                    result_tx,
                } => {
                    self.load_map(map_path, map, result_tx).await;
                }
                ClientManagerCommand::GameTick(result_tx) => {
                    let result = self.game_tick().await;
                    let _ = result_tx.send(result);
                }

                ClientManagerCommand::LoadTimeout => {
                    for client in &self.loading_clients {
                        client
                            .send(SerializedServerMessage::new(ServerMessage::LoadedTooSlow))
                            .await;
                    }
                    self.waiting_clients.extend(self.loading_clients.drain(..));
                }

                ClientManagerCommand::PickupRespawn { kind, index } => {
                    self.game_state.respawn_pickup(kind, index);
                }
            }
        }
    }

    async fn add_client(&mut self, client: Client) {
        self.waiting_clients.push(client);
        self.send(
            SendTo::All,
            ServerMessage::PlayerCountChanged {
                count: self.waiting_clients.len() + self.clients.len(),
            },
        )
        .await;

        if let Some(tx) = self.waiting_for_clients.take() {
            tx.send(()).unwrap();
        }
    }

    async fn remove_client(&mut self, id: ClientId) {
        if let Some(client) = self.clients.remove(&id) {
            client.disconnect();
        } else if let Some(pos) = self.waiting_clients.iter().position(|c| c.id() == id) {
            self.waiting_clients.remove(pos).disconnect();
        }

        self.send(
            SendTo::All,
            ServerMessage::PlayerCountChanged {
                count: self.waiting_clients.len() + self.clients.len(),
            },
        )
        .await;
    }

    async fn load_map(
        &mut self,
        map_path: String,
        map: Map,
        result_tx: oneshot::Sender<Vec<(ClientId, String)>>,
    ) {
        self.game_state = GameState::from_map(map);

        self.loading_clients.extend(self.waiting_clients.drain(..));

        self.send(
            SendTo::LoadingAll,
            ServerMessage::PrepareRound {
                map: map_path.to_string(),
            },
        )
        .await;

        let handle = self.make_handle();
        let handle = task::spawn(async move {
            time::sleep(Duration::from_secs(10)).await;
            handle
                .tx
                .send(ClientManagerCommand::LoadTimeout)
                .await
                .unwrap();
        });

        self.loading_task = Some(LoadingTask {
            join_handle: handle,
            result_tx,
        });
    }

    async fn game_tick(&mut self) -> TickResult {
        let handle = self.make_handle();
        self.game_state.tick(&mut self.clients, handle).await;

        let race_update = ServerMessage::RaceUpdate {
            players: self
                .clients
                .iter()
                .map(|(&id, client)| (id, client.state.clone()))
                .collect(),
            active_items: self.game_state.active_items().to_vec(),
        };

        self.send(SendTo::InGameAll, race_update).await;

        TickResult::NoChange
    }

    async fn handle_client_message(&mut self, id: ClientId, message: ClientMessage) {
        match message {
            ClientMessage::LoadedMap => {
                if let Some(client) = self.loading_clients.iter().position(|c| c.id() == id) {
                    self.clients
                        .insert(id, self.loading_clients.swap_remove(client));

                    if self.loading_clients.is_empty() {
                        if let Some(task) = self.loading_task.take() {
                            task.join_handle.abort();
                            task.result_tx
                                .send(
                                    self.clients
                                        .values()
                                        .map(|c| (c.id(), c.name().to_string()))
                                        .collect(),
                                )
                                .unwrap();
                        }
                    }
                }
            }

            ClientMessage::PlayerUpdate(state) => {
                if let Some(client) = self.clients.get_mut(&id) {
                    client.state = state;
                }
            }

            ClientMessage::UseItem(kind) => {
                if let Some(client) = self.clients.get(&id) {
                    self.game_state.add_item(ActiveItem {
                        pos: client.state.pos,
                        kind,
                    })
                }
            }

            ClientMessage::PickUp { kind, index } => {
                let success = self.game_state.pickup(kind, index);

                if success {
                    self.send(
                        SendTo::InGameExcept(id),
                        ServerMessage::PickUpStateChange {
                            kind,
                            index,
                            state: true,
                        },
                    )
                    .await;

                    let handle = self.make_handle();
                    task::spawn(async move {
                        time::sleep(Duration::from_secs(5)).await;
                        handle.pickup_respawn(kind, index).await;
                    });
                }
            }

            ClientMessage::Register { .. } => {
                log::warn!("client {id} tried to register again");
            }
        }
    }

    async fn send(&self, to: SendTo, msg: ServerMessage) {
        let msg = SerializedServerMessage::new(msg);
        match to {
            SendTo::All => {
                for client in self
                    .waiting_clients
                    .iter()
                    .chain(self.loading_clients.iter())
                    .chain(self.clients.values())
                {
                    client.send(msg.clone()).await;
                }
            }
            SendTo::LoadingAll => {
                for client in self.loading_clients.iter() {
                    client.send(msg.clone()).await;
                }
            }
            SendTo::InGameAll => {
                for client in self.clients.values() {
                    client.send(msg.clone()).await;
                }
            }
            SendTo::InGameExcept(id) => {
                for client in self.clients.values() {
                    if client.id() != id {
                        client.send(msg.clone()).await;
                    }
                }
            }
            SendTo::InGameOnly(id) => {
                if let Some(client) = self.clients.get(&id) {
                    client.send(msg).await;
                }
            }
        }
    }
}
