use serde::{Deserialize, Serialize};

pub mod map;
pub mod types;
pub use map::TrackPosition;
use types::*;

pub const TICKS_PER_SECOND: f32 = 60.0;
pub const COUNTDOWN_DURATION: f32 = 3.0;

pub const MAP_SCALE: f32 = 20.0;

pub fn map_coord_to_world(pos: Vec2) -> Vec2 {
    (pos / MAP_SCALE) * 2.0
}

pub fn world_coord_to_map(pos: Vec2) -> Vec2 {
    (pos / 2.0) * MAP_SCALE
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    DuplicateLogin,

    // server is preparing a new round
    PrepareRound {
        map: String,
    },

    // number of players has changed
    PlayerCountChanged {
        count: usize,
    },
    PlayerLeft(ClientId),

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
        race_time: f32,
        players: Vec<(ClientId, PlayerState)>,

        active_items: Vec<ActiveItem>,
    },

    // player has been hit by an item
    HitByItem {
        player: ClientId,
    },

    // PlayerCollision {
    //     depth: f32,
    //     other_velocity: f32,
    //     other_rotation: f32,
    //     normal: Vec2,
    // },

    // round has ended, show placements
    EndRound {
        placements: Vec<Placement>,
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

    FinishRound { race_time: f32 }, // player has finished the round
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
    pub vel: f32,
    pub rot: f32,
    pub visual_rot: f32,
    pub track_pos: TrackPosition,
    pub jump_height: f32,
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
    pub rot: f32,
    pub kind: ActiveItemKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActiveItemKind {
    GreenShell { roll: f32 },
    RedShell { roll: f32 },
    Banana,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ItemKind {
    GreenShell,
    RedShell,
    Banana,
    Boost,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Placement {
    pub client_id: ClientId,
    pub finish_time: Option<f32>,
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
