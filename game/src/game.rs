use common::{map::Map, types::*, ClientMessage, ServerMessage};
use glow::*;
use std::sync::mpsc;
use web_sys::WebSocket;

use crate::engine::{object::Object, Camera, RenderContext, Shaders, UpdateContext};

mod map;
pub use map::Collider;
use map::{MapDownload, MapToScene};
pub mod objects;

#[derive(Debug)]
enum State {
    WaitingToJoin,
    Loading { map_download: MapDownload },
    WaitingToStart { map: Map },
    Running { scene: Scene, map: Map },
}

#[derive(Debug)]
struct Scene {
    cam: Camera,
    player: objects::Player,
    colliders: Vec<Collider>,
    objects: Vec<Box<dyn Object>>,
    map_dimensions: Vec2,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::WaitingToJoin => write!(f, "WaitingToJoin"),
            State::Loading { .. } => write!(f, "Loading"),
            State::WaitingToStart { .. } => write!(f, "WaitingToStart"),
            State::Running { .. } => write!(f, "Running"),
        }
    }
}

pub struct Game {
    ws: WebSocket,
    ws_rx: mpsc::Receiver<ServerMessage>,
    gl: glow::Context,
    shaders: Shaders,
    viewport: Vec2,

    state: State,
}

impl Game {
    pub fn new(
        ws: WebSocket,
        ws_rx: mpsc::Receiver<ServerMessage>,
        gl: glow::Context,
        viewport: Vec2,
    ) -> Self {
        let shaders = Shaders::new(&gl);

        unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            gl.clear_color(0.0, 0.0, 0.0, 1.0);
        }

        // let map_download = MapDownload::start("maps/mcircuit/mcircuit.smk".to_string());

        Self {
            ws,
            ws_rx,
            gl,
            shaders,
            viewport,

            state: State::WaitingToJoin,
        }
    }

    pub fn connect(&mut self) {
        self.send(ClientMessage::Register {
            name: "cool player".to_string(),
        });
    }

    pub fn resize(&mut self, dim: Vec2) {
        self.viewport = dim;
        unsafe { self.gl.viewport(0, 0, dim.x as i32, dim.y as i32) };

        match &mut self.state {
            State::Running { scene, .. } => {
                scene.cam.resize(dim);
            }
            _ => {}
        }
    }

    pub fn key_down(&mut self, key: String) {
        match &mut self.state {
            State::Running { scene, .. } => {
                scene.player.key_down(&key);
            }
            _ => {}
        }
    }
    pub fn key_up(&mut self, key: String) {
        match &mut self.state {
            State::Running { scene, .. } => {
                scene.player.key_up(&key);
            }
            _ => {}
        }
    }

    fn send(&self, msg: ClientMessage) {
        let bytes = msg.to_bytes().unwrap();
        match self.ws.send_with_u8_array(&bytes) {
            Ok(_) => {}
            Err(err) => log::error!("Error sending message: {:?}", err),
        }
    }

    pub fn update(&mut self, dt: f32, tick: bool) {
        // handle messages
        while let Ok(msg) = self.ws_rx.try_recv() {
            match msg {
                ServerMessage::PrepareRound { map }
                    if matches!(self.state, State::WaitingToJoin) =>
                {
                    log::info!("preparing round with map: {:?}", map);
                    let map_download = MapDownload::start(map);
                    self.state = State::Loading { map_download };
                }
                ServerMessage::StartRound { params }
                    if matches!(self.state, State::WaitingToStart { .. }) =>
                {
                    log::info!("starting round with params: {:?}", params);
                    let map = match &self.state {
                        State::WaitingToStart { map } => map.clone(),
                        _ => unreachable!(),
                    };
                    let scene = map.to_scene(&self.gl, self.viewport, &params);
                    self.state = State::Running { map, scene };
                }

                ServerMessage::RaceUpdate { .. } => {}
                _ => log::warn!("ignoring unexpected message: {:?}", msg),
            }
        }

        // update
        match &mut self.state {
            State::Running { scene, .. } => {
                let mut ctx = UpdateContext {
                    dt,
                    tick,
                    colliders: &scene.colliders,
                    send_msg: &mut |msg| {
                        let bytes = msg.to_bytes().unwrap();
                        match self.ws.send_with_u8_array(&bytes) {
                            Ok(_) => {}
                            Err(err) => log::error!("Error sending message: {:?}", err),
                        }
                    },
                };

                scene.objects.iter_mut().for_each(|o| o.update(&mut ctx));

                scene.player.update(&mut ctx);
                scene.player.update_cam(&mut scene.cam);
            }
            State::Loading { map_download } => {
                let map = match map_download.poll() {
                    Some(Ok(map)) => map,
                    Some(Err(err)) => {
                        log::error!("error loading map: {:?}", err);
                        return;
                    }
                    None => return,
                };

                log::info!(
                    "loaded map '{:?}', waiting for round start",
                    map.metadata.name
                );
                self.send(ClientMessage::LoadedMap);
                self.state = State::WaitingToStart { map };
                // let objects = map.to_scene(&self.gl, params);
                // let cam = Camera::new(60.0, self.viewport);
                // self.state = State::Running { cam, objects, map };
            }
            _ => {}
        }

        // render
        unsafe {
            self.gl
                .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT | glow::STENCIL_BUFFER_BIT);
        };

        match &self.state {
            State::Running { scene, .. } => {
                let ctx = RenderContext {
                    gl: &self.gl,
                    shaders: &self.shaders,
                    cam: &scene.cam,
                };

                // why does player transparency not work when the player is rendered first??
                scene.objects.iter().for_each(|o| o.render(&ctx));
                scene.player.render(&ctx);
            }
            _ => log::warn!("todo: render state {}", self.state),
        }
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        self.ws.close().unwrap();
        self.shaders.cleanup(&self.gl);

        match &mut self.state {
            State::Running { scene, .. } => {
                scene.player.cleanup(&self.gl);
                scene.objects.iter_mut().for_each(|o| o.cleanup(&self.gl));
            }
            _ => {}
        }
    }
}

impl std::fmt::Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Game")
            .field("shaders", &self.shaders)
            .field("state", &self.state)
            .finish()
    }
}
