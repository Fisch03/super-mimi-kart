use common::{types::*, ClientId, PlayerUpdate, ServerMessage};
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct Client {
    id: ClientId,
    name: String,
    tx: mpsc::UnboundedSender<ServerMessage>,

    pub state: ClientState,
}

#[derive(Debug, Clone)]
pub enum ClientState {
    WaitingToJoin,
    LoadingMap,
    InGame(IngameState),
}

#[derive(Debug, Clone, Default)]
pub struct IngameState {
    pub pos: Vec2,
    pub rot: f32,
}

impl ClientState {
    pub fn is_loading_map(&self) -> bool {
        match self {
            Self::LoadingMap => true,
            _ => false,
        }
    }

    pub fn is_ingame(&self) -> bool {
        match self {
            Self::InGame(_) => true,
            _ => false,
        }
    }

    pub fn in_game() -> Self {
        Self::InGame(IngameState::default())
    }

    pub fn get_ingame(&self) -> Option<&IngameState> {
        match self {
            Self::InGame(state) => Some(state),
            _ => None,
        }
    }

    pub fn get_ingame_mut(&mut self) -> Option<&mut IngameState> {
        match self {
            Self::InGame(state) => Some(state),
            _ => None,
        }
    }
}

impl IngameState {
    pub fn update(&mut self, update: PlayerUpdate) {
        self.pos = update.pos;
        self.rot = update.rot;
    }
}

impl Client {
    pub fn new(id: ClientId, name: String, tx: mpsc::UnboundedSender<ServerMessage>) -> Self {
        Self {
            id,
            name,
            tx,
            state: ClientState::WaitingToJoin,
        }
    }

    pub fn id(&self) -> ClientId {
        self.id
    }

    pub fn send(&self, message: ServerMessage) {
        log::debug!("({}) sending message: {:?}", self.id.as_u32(), message);
        match self.tx.send(message) {
            Ok(_) => {}
            Err(e) => log::error!("error sending message to client: {}", e),
        }
    }
}
