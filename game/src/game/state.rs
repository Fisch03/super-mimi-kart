use super::Game;
use crate::engine::{object::Object, Camera, RenderContext, Shaders, UpdateContext};
use common::{map::Map, ClientMessage};

// mod running;
// use running::Running;

#[derive(Debug)]
pub(super) enum GameState {
    WaitingToJoin,
    Loading {
        map_download: MapDownload,
    },
    WaitingToStart {
        map: Map,
    },
    Running {
        cam: Camera,
        objects: Vec<Box<dyn Object>>,
        map: Map,
    },
}

impl GameState {
    pub fn cleanup(&mut self) {
        match self {
            Self::Running { objects, .. } => {
                objects.iter_mut().for_each(|o| o.cleanup(&self.gl));
            }
            _ => {}
        }
    }
}

impl std::fmt::Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WaitingToJoin => write!(f, "WaitingToJoin"),
            Self::Loading { .. } => write!(f, "Loading"),
            Self::WaitingToStart { .. } => write!(f, "WaitingToStart"),
            Self::Running { .. } => write!(f, "Running"),
        }
    }
}

impl Game {
    pub fn update_state(&mut self, dt: f32, tick: bool) {
        match &mut self.state {
            GameState::Running { cam, objects, .. } => {
                let mut ctx = UpdateContext {
                    dt,
                    tick,
                    cam,
                    send_msg: &mut |msg| {
                        let bytes = msg.to_bytes().unwrap();
                        match self.ws.send_with_u8_array(&bytes) {
                            Ok(_) => {}
                            Err(err) => log::error!("Error sending message: {:?}", err),
                        }
                    },
                };
                objects.iter_mut().for_each(|o| o.update(&mut ctx));
            }
            GameState::Loading { map_download } => {
                let map = match map_download.poll() {
                    Some(Ok(map)) => map,
                    Some(Err(err)) => {
                        log::error!("error loading map: {:?}", err);
                        return;
                    }
                    None => return,
                };

                log::info!("loaded map '{:?}', waiting for round start", map.metadata);
                self.send(ClientMessage::LoadedMap);
                self.state = GameState::WaitingToStart { map };
                // let objects = map.to_scene(&self.gl, params);
                // let cam = Camera::new(60.0, self.viewport);
                // self.state = State::Running { cam, objects, map };
            }
            _ => {}
        }
    }
}

impl Drop for GameState {}
