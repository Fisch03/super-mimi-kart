use common::{
    ClientId, ClientMessage, PlayerState, RoundInitParams, ServerMessage, COUNTDOWN_DURATION,
    TICKS_PER_SECOND,
};
use rand::seq::SliceRandom;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Mutex, MutexGuard,
    },
    time::Duration,
};
use tokio::{sync::mpsc, time::interval};

mod client;
use client::{Client, ClientState};

#[derive(Debug, Default)]
pub struct GameServer {
    next_client_id: AtomicU32,
    inner: Mutex<GameServerInner>,
}

#[derive(Debug, Default)]
struct GameServerInner {
    clients: HashMap<ClientId, Client>,
    state: State,
    runner: Option<RunnerHandle>,
}

#[derive(Debug)]
struct RunnerHandle {
    kill_tx: mpsc::Sender<()>,
    notify_tx: mpsc::Sender<()>,
}
impl RunnerHandle {
    fn notify(&self) {
        use mpsc::error::TrySendError;
        match self.notify_tx.try_send(()) {
            Ok(_) | Err(TrySendError::Full(_)) => {}
            Err(e) => log::error!("error notifying runner: {}", e),
        }
    }

    async fn kill(self) {
        match self.kill_tx.send(()).await {
            Ok(_) => {}
            Err(e) => log::error!("error killing runner: {}", e),
        }
    }
}

enum SendTo {
    All,
    AllExcept(ClientId),
    Only(ClientId),
}

#[derive(Debug, Clone, Copy)]
enum State {
    Idle,
    WaitingForLoad,
    InGame,
}

impl Default for State {
    fn default() -> Self {
        Self::Idle
    }
}

impl GameServer {
    fn inner(&self) -> MutexGuard<GameServerInner> {
        self.inner.lock().unwrap()
    }

    fn with_inner<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut GameServerInner) -> R,
    {
        f(&mut self.inner.lock().unwrap())
    }

    fn ensure_runner(self: &Arc<Self>) {
        let mut inner = self.inner();
        if inner.runner.is_some() {
            return;
        }

        let (kill_tx, mut kill_rx) = mpsc::channel(1);
        let (notify_tx, notify_rx) = mpsc::channel(1);
        {
            let server = self.clone();
            tokio::spawn(async move {
                log::info!("starting server runner");
                tokio::select! {
                    _ = kill_rx.recv() => {}
                    _ = server.run(notify_rx) => {}
                }

                server.with_inner(|inner| {
                    inner.runner = None;
                    inner.state = State::Idle;
                });
                log::info!("server runner stopped");
            });
        }

        inner.runner = Some(RunnerHandle { kill_tx, notify_tx });
    }

    fn runner_notify(&self) {
        if let Some(runner) = self.inner().runner.as_ref() {
            runner.notify();
        }
    }

    async fn run(&self, mut notify_rx: mpsc::Receiver<()>) {
        let mut initial_round = true;

        loop {
            #[cfg(not(debug_assertions))]
            let wait_time = if initial_round {
                initial_round = false;
                30
            } else {
                10
            };
            #[cfg(debug_assertions)]
            let wait_time = 5;

            log::info!("waiting {} seconds for players to join", wait_time);
            tokio::time::sleep(Duration::from_secs(wait_time)).await;

            self.with_inner(|inner| {
                inner.state = State::WaitingForLoad;
                inner.clients.values_mut().for_each(|client| {
                    client.state = ClientState::LoadingMap;
                });
            });

            let map = "maps/mcircuit/mcircuit.smk";
            self.send(
                SendTo::All,
                ServerMessage::PrepareRound {
                    map: map.to_string(),
                },
            );
            log::info!("waiting for players to load map '{:?}'", map);

            let wait_for_all_loaded = async {
                loop {
                    let _ = notify_rx.recv().await;
                    let loaded = self.with_inner(|inner| {
                        let mut all_loaded = true;
                        let loaded: Vec<_> = inner
                            .clients
                            .values()
                            .map(|client| {
                                all_loaded &= !client.state.is_loading_map();
                                client.id()
                            })
                            .collect();

                        if !all_loaded {
                            return None;
                        }

                        Some(loaded)
                    });

                    if let Some(loaded) = loaded {
                        break loaded;
                    }
                }
            };
            let loaded = tokio::select! {
                loaded = wait_for_all_loaded => {
                    log::info!("all players loaded the map, starting round");
                    loaded
                }
                _ = tokio::time::sleep(Duration::from_secs(20)) => {
                    log::info!("not all players loaded the map in time, proceeding without them");
                    let mut inner = self.inner();
                    inner.clients.retain(|_, client| {
                        let retain = client.state.is_ingame();
                        if !retain {
                            log::info!("kicking client {:?} for not loading the map in time", client.id());
                            client.send(ServerMessage::LoadedTooSlow);
                        }
                        retain
                    });
                    inner.clients.values_mut().filter(|client| {
                        client.state.is_ingame()
                    }).map(|client| client.id()).collect()
                }
            };

            let mut players_in_round: Vec<ClientId> = loaded
                .choose_multiple(&mut rand::thread_rng(), loaded.len())
                .map(|id| *id)
                .collect();

            let start_positions: Vec<_> = self.with_inner(|inner| {
                players_in_round
                    .iter()
                    .filter_map(|id| {
                        if let Some(client) = inner.clients.get_mut(id) {
                            Some((client.id(), client.name().to_string()))
                        } else {
                            None
                        }
                    })
                    .collect()
            });
            (0..start_positions.len()).for_each(|i| {
                let (id, _) = start_positions[i];
                self.send(
                    SendTo::Only(id),
                    ServerMessage::StartRound {
                        params: RoundInitParams {
                            start_pos: i,
                            players: start_positions.clone(),
                        },
                    },
                );
            });

            log::info!("waiting for race countdown to finish");
            tokio::time::sleep(Duration::from_secs_f32(COUNTDOWN_DURATION)).await;

            log::info!("round started with players: {:?}", players_in_round);
            self.with_inner(|inner| {
                inner.state = State::InGame;
            });
            self.send(SendTo::All, ServerMessage::StartRace);

            let mut tick_interval =
                interval(Duration::from_secs_f64(1.0 / TICKS_PER_SECOND as f64));
            loop {
                tick_interval.tick().await;
                let mut player_updates = Vec::new();
                self.with_inner(|inner| {
                    players_in_round.retain(|&id| {
                        let ingame_client = match inner.clients.get(&id) {
                            Some(client) if matches!(client.state, ClientState::InGame(_)) => {
                                client.state.get_ingame().unwrap()
                            }
                            _ => {
                                log::info!("player {:?} left the game", id);
                                return false;
                            }
                        };

                        player_updates.push((
                            id,
                            PlayerState {
                                pos: ingame_client.pos,
                                rot: ingame_client.rot,
                            },
                        ));

                        true
                    });
                });

                players_in_round.iter().for_each(|&id| {
                    self.send(
                        SendTo::Only(id),
                        ServerMessage::RaceUpdate {
                            players: player_updates.clone(), // this kinda sucks but its fine for
                                                             // now i guess
                        },
                    );
                });
            }
        }
    }

    fn send(&self, to: SendTo, msg: ServerMessage) {
        let inner = self.inner();
        match to {
            SendTo::All => {
                for client in inner.clients.values() {
                    let _ = client.send(msg.clone());
                }
            }
            SendTo::AllExcept(id) => {
                for (client_id, client) in inner.clients.iter() {
                    if *client_id != id {
                        let _ = client.send(msg.clone());
                    }
                }
            }
            SendTo::Only(id) => {
                if let Some(client) = inner.clients.get(&id) {
                    let _ = client.send(msg);
                }
            }
        }
    }

    pub fn allocate_client(&self) -> ClientId {
        ClientId::new(self.next_client_id.fetch_add(1, Ordering::Relaxed))
    }

    pub async fn register_client(
        self: &Arc<Self>,
        client_id: ClientId,
        name: String,
    ) -> mpsc::UnboundedReceiver<ServerMessage> {
        let (msg_tx, msg_rx) = mpsc::unbounded_channel();

        let client = Client::new(client_id, name, msg_tx);
        let player_count = self.with_inner(|inner| {
            if let Some(old) = inner.clients.insert(client_id, client) {
                log::error!("client id collision! removed old client: {:?}", old);
            }

            inner.clients.len()
        });

        self.ensure_runner();
        self.runner_notify();

        self.send(
            SendTo::All,
            ServerMessage::PlayerCountChanged {
                count: player_count,
            },
        );

        msg_rx
    }

    pub async fn handle_client_message(&self, client_id: ClientId, msg: ClientMessage) {
        self.with_inner(|inner| {
            let state = inner.state;
            let client = match inner.clients.get_mut(&client_id) {
                Some(client) => client,
                None => {
                    log::warn!("received message from unknown client: {:?}", client_id);
                    return;
                }
            };

            log::debug!("({}) received message: {:?}", client_id, msg);

            match msg {
                ClientMessage::LoadedMap if matches!(state, State::WaitingForLoad) => {
                    if client.state.is_loading_map() {
                        client.state = ClientState::in_game();
                    }
                }

                ClientMessage::PlayerUpdate(update) if matches!(state, State::InGame) => {
                    if let Some(ingame) = client.state.get_ingame_mut() {
                        ingame.update(update);
                    } else {
                        log::warn!(
                            "received player update from client that is not in game: {:?}",
                            client_id
                        );
                    }
                }
                ClientMessage::PlayerUpdate(_) => log::trace!(
                    "ignoring player update from client {:?} in state {:?}",
                    client_id,
                    state
                ),
                _ => log::warn!(
                    "ignoring unexpected message from client {}: {:?}",
                    client_id,
                    msg
                ),
            }
        });

        self.runner_notify();
    }

    pub async fn remove_client(&self, client_id: ClientId) {
        let killed_runner = self.with_inner(|inner| {
            inner.clients.remove(&client_id);

            self.send(
                SendTo::All,
                ServerMessage::PlayerCountChanged {
                    count: inner.clients.len(),
                },
            );

            self.runner_notify();

            if inner.clients.is_empty() {
                Some(inner.runner.take().unwrap())
            } else {
                None
            }
        });

        if let Some(runner) = killed_runner {
            runner.kill().await;
        }
    }
}
