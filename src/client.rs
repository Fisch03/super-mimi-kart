use common::{ClientId, PlayerState};
use tokio::sync::mpsc;

use crate::server::SerializedServerMessage;

#[derive(Debug)]
pub struct Client {
    id: ClientId,
    name: String,
    tx: mpsc::Sender<SerializedServerMessage>,
    pub state: PlayerState,
}

impl Client {
    pub fn new(id: ClientId, name: String, tx: mpsc::Sender<SerializedServerMessage>) -> Self {
        Self {
            id,
            name,
            tx,
            state: PlayerState::default(),
        }
    }

    pub fn id(&self) -> ClientId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn send<M: Into<SerializedServerMessage>>(&self, message: M) {
        let message = message.into();
        match self.tx.send(message).await {
            Ok(_) => {}
            Err(e) => log::error!("error sending command to client: {}", e),
        }
    }

    pub fn disconnect(self) {
        log::error!("TODO: disconnect client");
    }
}
