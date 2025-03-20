use common::{ClientId, ClientMessage, PickupKind, ServerMessage, map::Map, types::*};
use glow::*;
use std::collections::HashMap;
use std::sync::mpsc;
use web_sys::WebSocket;

use crate::engine::{
    Camera, CreateContext, RenderContext, Shaders, UpdateContext, cache::AssetCache, object::Object,
};

mod map;
pub use map::{Collider, Offroad};
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
    own_id: ClientId,
    player: objects::Player,
    players: HashMap<ClientId, objects::ExternalPlayer>,

    colliders: Vec<Collider>,
    offroad: Vec<Offroad>,

    coins: Vec<objects::Coin>,
    item_boxes: Vec<objects::ItemBox>,

    static_objects: Vec<Box<dyn Object>>,
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

    cache: AssetCache,
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
            cache: Default::default(),
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
        let dt = dt.min(0.1); // cap to 100ms

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

                    self.cache.clear();
                    let ctx = CreateContext {
                        gl: &self.gl,
                        assets: &self.cache,
                    };
                    let scene = map.to_scene(&ctx, self.viewport, &params);

                    self.state = State::Running { map, scene };
                }

                ServerMessage::RaceUpdate { players, .. }
                    if matches!(self.state, State::Running { .. }) =>
                {
                    let scene = match &mut self.state {
                        State::Running { scene, .. } => scene,
                        _ => unreachable!(),
                    };

                    for (id, state) in players {
                        if let Some(player) = scene.players.get_mut(&id) {
                            player.update_state(state);
                        }
                    }
                }

                ServerMessage::PickUpStateChange { kind, index, state }
                    if matches!(self.state, State::Running { .. }) =>
                {
                    let scene = match &mut self.state {
                        State::Running { scene, .. } => scene,
                        _ => unreachable!(),
                    };

                    match kind {
                        PickupKind::Coin => {
                            let coins = &mut scene.coins;
                            if let Some(coin) = coins.get_mut(index) {
                                coin.state = state;
                            }
                        }
                        PickupKind::ItemBox => {
                            let item_boxes = &mut scene.item_boxes;
                            if let Some(item_box) = item_boxes.get_mut(index) {
                                item_box.state = state;
                            }
                        }
                    }
                }

                _ => log::warn!("ignoring unexpected message: {:?}", msg),
            }
        }

        // update
        match &mut self.state {
            State::Running { scene, map } => {
                let mut ctx = UpdateContext {
                    dt,
                    tick,
                    assets: &self.cache,
                    send_msg: &mut |msg| {
                        let bytes = msg.to_bytes().unwrap();
                        match self.ws.send_with_u8_array(&bytes) {
                            Ok(_) => {}
                            Err(err) => log::error!("Error sending message: {:?}", err),
                        }
                    },

                    map: &map,

                    colliders: &scene.colliders,
                    offroad: &scene.offroad,
                };

                scene
                    .static_objects
                    .iter_mut()
                    .for_each(|o| o.update(&mut ctx));
                scene
                    .players
                    .iter_mut()
                    .for_each(|(_, p)| p.update(&mut ctx));
                scene.coins.iter_mut().for_each(|c| c.update(&mut ctx));
                scene.item_boxes.iter_mut().for_each(|i| i.update(&mut ctx));

                scene.player.update(&mut ctx);
                scene.player.late_update(
                    &mut ctx,
                    &scene.players,
                    &scene.coins,
                    &scene.item_boxes,
                    &mut scene.cam,
                );
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
                    assets: &self.cache,

                    shaders: &self.shaders,
                    cam: &scene.cam,
                };

                let mut depth_objects: Vec<(&dyn Object, f32)> = scene
                    .static_objects
                    .iter()
                    .map(|o| o.as_ref() as &dyn Object)
                    .chain(scene.players.values().map(|o| o as &dyn Object))
                    .chain(std::iter::once(&scene.player as &dyn Object))
                    .chain(scene.coins.iter().map(|o| o as &dyn Object))
                    .chain(scene.item_boxes.iter().map(|o| o as &dyn Object))
                    .map(|o| {
                        let depth = o.as_ref().camera_depth(&scene.cam);
                        (o, depth)
                    })
                    .collect();

                // sort by depth
                depth_objects.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                for (o, _) in depth_objects {
                    o.render(&ctx);
                }
            }
            _ => log::warn!("todo: render state {}", self.state),
        }
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        self.ws.close().unwrap();
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
