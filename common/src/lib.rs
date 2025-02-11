use serde::{Deserialize, Serialize};

pub mod map;
pub mod types;
use types::*;

pub const TICKS_PER_SECOND: f32 = 60.0;
pub const COUNTDOWN_DURATION: f32 = 5.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    // server is preparing a new round
    PrepareRound {
        map: String,
    },
    //
    // number of players has changed
    PlayerCountChanged {
        count: usize,
    },

    // map load took to long, player has been kicked to lobby
    LoadedTooSlow,

    // everyone has loaded the map, start the countdown
    StartRound {
        params: RoundInitParams,
    },

    // countdown has finished, start the race
    StartRace,

    // update the positions of all players
    RaceUpdate {
        players: Vec<(ClientId, PlayerState)>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub pos: Vec2,
    pub rot: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundInitParams {
    pub start_pos: usize,
    pub players: Vec<(ClientId, String)>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ClientId(u32);
impl ClientId {
    pub fn new(id: u32) -> Self {
        Self(id)
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
