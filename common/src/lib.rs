use serde::{Deserialize, Serialize};

pub mod map;
pub mod types;
pub use map::TrackPosition;
use types::*;

pub const TICKS_PER_SECOND: f32 = 60.0;
pub const COUNTDOWN_DURATION: f32 = 5.0;

pub const SHELL_SPEED: f32 = 0.2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    // server is preparing a new round
    PrepareRound {
        map: String,
    },

    // number of players has changed
    PlayerCountChanged {
        count: usize,
    },

    // map load took to long, player has been kicked to lobby
    LoadedTooSlow,

    // everyone has loaded the map, send round init params
    StartRound {
        params: RoundInitParams,
    },

    // countdown has started
    StartCountdown,

    // countdown has finished, start the race
    StartRace,

    // pickup has been picked up or respawned
    PickUpStateChange {
        kind: PickupKind,
        index: usize,
        state: bool,
    },

    // update the positions of all players
    RaceUpdate {
        players: Vec<(ClientId, PlayerState)>,

        active_items: Vec<ActiveItem>,
    },

    // player has been hit by an item
    HitByItem {
        player: ClientId,
    },

    // round has ended, show placements
    EndRound {
        placements: Vec<()>,
    },
}

impl ServerMessage {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(bytes)
    }
    pub fn to_bytes(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_allocvec(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Register { name: String }, // register a new player
    LoadedMap,                 // client has loaded the map

    PickUp { kind: PickupKind, index: usize },

    UseItem(ActiveItemKind), // player has used an item

    PlayerUpdate(PlayerState), // update the player's position
}
impl ClientMessage {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(bytes)
    }
    pub fn to_bytes(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_allocvec(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerState {
    pub pos: Vec2,
    pub rot: f32,
    pub track_pos: TrackPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundInitParams {
    pub client_id: ClientId,
    pub start_pos: usize,
    pub players: Vec<(ClientId, String)>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PickupKind {
    Coin,
    ItemBox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveItem {
    pub pos: Vec2,
    pub kind: ActiveItemKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActiveItemKind {
    GreenShell { direction: Vec2 },
    RedShell { target: ClientId },
    Banana,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ItemKind {
    GreenShell,
    RedShell,
    Banana,
    Boost,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ClientId(u32);
impl ClientId {
    pub fn new(id: u32) -> Self {
        if id == 0 {
            panic!("Client id cannot be 0");
        }
        Self(id)
    }

    pub fn invalid() -> Self {
        Self(0)
    }

    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
