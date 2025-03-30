use glow::*;
use std::collections::HashMap;
use std::{rc::Rc, sync::mpsc};
use web_sys::WebSocket;

use crate::engine::{
    Camera, CreateContext, RenderContext, Shaders, UiCamera, UpdateContext, cache::AssetCache,
    object::Object,
};
use common::{ClientId, ClientMessage, PickupKind, Placement, ServerMessage, map::Map, types::*};

mod map;
pub use map::{Collider, Offroad};
use map::{MapDownload, MapToScene};
pub mod objects;

mod assets;
use assets::SharedAssets;

#[derive(Debug)]
enum State {
    MainMenu {
        click: bool,
        show_credits: bool,
    },
    WaitingToJoin,
    Loading {
        map_download: MapDownload,
    },
    WaitingToStart {
        map: Rc<Map>,
    },
    Running {
        scene: Scene,
        map: Rc<Map>,
        race_state: RaceState,
    },
}

#[derive(Debug)]
enum RaceState {
    Waiting,
    Countdown { current: u32, next: f32 },
    Running { race_time: f32 },
    Completed { place: usize },
    RaceResults { placements: Vec<Placement> },
}

#[derive(Debug)]
struct Scene {
    own_id: ClientId,
    player: objects::Player,
    players: HashMap<ClientId, objects::ExternalPlayer>,

    colliders: Vec<Collider>,
    offroad: Vec<Offroad>,

    coins: Vec<objects::Coin>,
    item_boxes: Vec<objects::ItemBox>,
    items: Vec<objects::Item>,

    map: objects::Map,

    static_objects: Vec<Box<dyn Object>>,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::MainMenu { .. } => write!(f, "MainMenu"),
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

    mouse_pos: Vec2,
    hide_cursor: bool,

    gl: glow::Context,
    shaders: Shaders,
    viewport: Vec2,
    cam: Camera,
    ui_cam: UiCamera,

    player_count: usize,

    rng: rand::rngs::SmallRng,

    cache: AssetCache,
    shared_assets: SharedAssets,
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
        let cache = AssetCache::default();

        unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.enable(glow::BLEND);
            gl.enable(glow::CULL_FACE);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            gl.clear_color(0.3, 0.3, 0.3, 1.0);
        }

        // let ctx = CreateContext {
        //     gl: &gl,
        //     assets: &cache,
        // };
        // objects::Item::preload_assets(&ctx);

        use rand::SeedableRng;
        let time = web_sys::window().unwrap().performance().unwrap().now();
        let rng = rand::rngs::SmallRng::seed_from_u64((time * 12345.0) as u64);

        let ctx = CreateContext {
            gl: &gl,
            assets: &cache,
            viewport,
        };
        let shared_assets = SharedAssets::load(&ctx);

        Self {
            ws,
            ws_rx,
            gl,

            mouse_pos: Vec2::default(),
            hide_cursor: false,

            shaders,
            viewport,
            ui_cam: UiCamera::new(viewport),
            cam: Camera::new(60.0, viewport),

            player_count: 1,

            rng,

            state: State::MainMenu {
                click: false,
                show_credits: false,
            },

            cache,
            shared_assets,
        }
    }

    pub fn connect(&mut self) {
        self.send(ClientMessage::Register {
            name: "cool player".to_string(),
        });
        self.state = State::WaitingToJoin;
    }

    pub fn resize(&mut self, dim: Vec2) {
        self.viewport = dim;
        unsafe { self.gl.viewport(0, 0, dim.x as i32, dim.y as i32) };

        self.cam.resize(dim);
        self.ui_cam.resize(dim);
    }

    pub fn mouse_move(&mut self, pos: Vec2) {
        self.mouse_pos = pos;
        self.hide_cursor = false;
    }
    pub fn mouse_down(&mut self) {
        match &mut self.state {
            State::MainMenu { click, .. } => {
                *click = true;
            }
            _ => {}
        }
    }
    pub fn mouse_up(&mut self) {}

    pub fn key_down(&mut self, key: String) {
        match &mut self.state {
            State::Running {
                scene,
                race_state: RaceState::Running { .. },
                ..
            } => {
                scene.player.key_down(&key);
            }
            _ => {}
        }
        self.hide_cursor = true;
    }
    pub fn key_up(&mut self, key: String) {
        match &mut self.state {
            State::Running {
                scene,
                race_state: RaceState::Running { .. },
                ..
            } => {
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
            match (msg, &mut self.state) {
                (ServerMessage::PrepareRound { map }, _) => {
                    log::info!("preparing round with map: {:?}", map);
                    let map_download = MapDownload::start(map);
                    self.state = State::Loading { map_download };
                }

                (ServerMessage::LoadedTooSlow, _) => {
                    log::warn!("loaded too slow");
                    self.state = State::WaitingToJoin;
                }

                (ServerMessage::StartRound { params }, State::WaitingToStart { map, .. }) => {
                    log::info!("starting round with params: {:?}", params);

                    self.cache.clear();
                    let ctx = CreateContext {
                        gl: &self.gl,
                        assets: &self.cache,
                        viewport: self.viewport,
                    };
                    objects::Item::preload_assets(&ctx);
                    let scene = map.to_scene(&ctx, &params);

                    self.state = State::Running {
                        map: map.clone(),
                        scene,
                        race_state: RaceState::Waiting,
                    };
                }
                (ServerMessage::StartRound { .. }, _) => {
                    log::warn!("received StartRound message in invalid state");
                }

                (ServerMessage::StartCountdown, State::Running { race_state, .. }) => {
                    *race_state = RaceState::Countdown {
                        current: 3,
                        next: 0.0,
                    };
                }
                (ServerMessage::StartCountdown, _) => {
                    log::warn!("received StartCountdown message in invalid state");
                }

                (ServerMessage::StartRace, State::Running { race_state, .. }) => {
                    *race_state = RaceState::Running { race_time: 0.0 }
                }
                (ServerMessage::StartRace, _) => {
                    log::warn!("received StartRace message in invalid state");
                }

                (
                    ServerMessage::RaceUpdate {
                        players,
                        active_items,
                        race_time: new_race_time,
                    },
                    State::Running {
                        scene, race_state, ..
                    },
                ) => {
                    let ctx = CreateContext {
                        gl: &self.gl,
                        assets: &self.cache,
                        viewport: self.viewport,
                    };

                    if let RaceState::Running { race_time } = race_state {
                        *race_time = new_race_time;
                    }

                    scene.items.clear();
                    scene.items.extend(
                        active_items
                            .into_iter()
                            .map(|i| objects::Item::new(&ctx, i)),
                    );

                    for (id, state) in players {
                        if let Some(player) = scene.players.get_mut(&id) {
                            player.update_state(state);
                        }
                    }
                }

                (ServerMessage::RaceUpdate { .. }, _) => {
                    log::warn!("received RaceUpdate message in invalid state");
                }

                (
                    ServerMessage::PickUpStateChange { kind, index, state },
                    State::Running { scene, .. },
                ) => match kind {
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
                },
                (ServerMessage::PickUpStateChange { .. }, _) => {
                    log::warn!("received PickUpStateChange message in invalid state");
                }

                // (
                //     ServerMessage::PlayerCollision {
                //         normal,
                //         depth,
                //
                //         other_velocity,
                //         other_rotation,
                //     },
                //     State::Running {
                //         scene,
                //         race_state: RaceState::Running { .. },
                //         ..
                //     },
                // ) => {
                //     scene
                //         .player
                //         .apply_collision(normal, depth, other_velocity, other_rotation);
                // }
                // (ServerMessage::PlayerCollision { .. }, _) => {
                //     log::warn!("received PlayerCollision message in invalid state");
                // }
                (ServerMessage::HitByItem { player }, State::Running { scene, .. }) => {
                    if player == scene.own_id {
                        scene.player.hit();
                    }
                }
                (ServerMessage::HitByItem { .. }, _) => {
                    log::warn!("received HitByItem message in invalid state");
                }

                (ServerMessage::PlayerCountChanged { count }, _) => self.player_count = count,
                (ServerMessage::PlayerLeft(id), _) => {
                    if let State::Running { scene, .. } = &mut self.state {
                        scene.players.remove(&id);
                    }
                }

                (ServerMessage::EndRound { placements }, State::Running { race_state, .. }) => {
                    *race_state = RaceState::RaceResults { placements };
                }
                (ServerMessage::EndRound { .. }, _) => {
                    self.state = State::WaitingToJoin;
                }
            }
        }

        // update
        match &mut self.state {
            State::Running {
                scene,
                map,
                race_state,
            } => {
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

                    rng: &mut self.rng,
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
                    &mut self.cam,
                );

                match race_state {
                    RaceState::Waiting => {}
                    RaceState::Countdown { current, next } => {
                        *next += dt;
                        if *next >= 1.0 {
                            *next -= 1.0;
                            *current = current.saturating_sub(1);
                        }
                    }
                    RaceState::Running { race_time } => {
                        if scene.player.track_pos.lap > 3 {
                            let race_time = *race_time;
                            *race_state = RaceState::Completed {
                                place: scene.player.place,
                            };
                            scene.player.input = Default::default();
                            self.send(ClientMessage::FinishRound { race_time });
                        }
                    }
                    RaceState::Completed { .. } => {}
                    RaceState::RaceResults { .. } => {}
                }
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
                self.state = State::WaitingToStart { map: Rc::new(map) };
                // let objects = map.to_scene(&self.gl, params);
                // let cam = Camera::new(60.0, self.viewport);
                // self.state = State::Running { cam, objects, map };
            }
            State::MainMenu {
                click,
                show_credits,
            } => {
                if *click {
                    *click = false;

                    if self
                        .shared_assets
                        .credits_button
                        .hovered(self.viewport, self.mouse_pos)
                        || *show_credits
                    {
                        *show_credits = !*show_credits;
                    } else if self
                        .shared_assets
                        .start_button
                        .hovered(self.viewport, self.mouse_pos)
                    {
                        self.connect();
                    }
                }
            }
            _ => {}
        }

        // render
        unsafe {
            self.gl
                .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT | glow::STENCIL_BUFFER_BIT);
        };

        let ctx = RenderContext {
            gl: &self.gl,
            viewport: self.viewport,
            assets: &self.cache,

            shaders: &self.shaders,

            cam: &self.cam,
            ui_cam: &self.ui_cam,

            mouse_pos: self.mouse_pos,
        };

        match &self.state {
            State::Running {
                scene, race_state, ..
            } => {
                let mut depth_objects: Vec<(&dyn Object, f32)> = scene
                    .static_objects
                    .iter()
                    .map(|o| o.as_ref() as &dyn Object)
                    .chain(scene.players.values().map(|o| o as &dyn Object))
                    .chain(std::iter::once(&scene.player as &dyn Object))
                    .chain(scene.coins.iter().map(|o| o as &dyn Object))
                    .chain(scene.item_boxes.iter().map(|o| o as &dyn Object))
                    .chain(scene.items.iter().map(|o| o as &dyn Object))
                    .map(|o| {
                        let depth = o.as_ref().camera_depth(&self.cam);
                        (o, depth)
                    })
                    .collect();

                // sort by depth
                depth_objects.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

                // render
                self.shared_assets.skybox.render(&ctx);
                scene.map.render(&ctx); // map is always at the back
                for (o, _) in depth_objects {
                    o.render(&ctx);
                }

                unsafe { self.gl.disable(glow::DEPTH_TEST) };
                match race_state {
                    RaceState::Waiting => {}
                    RaceState::Countdown { current, .. } => {
                        self.shared_assets
                            .render_countdown(&ctx, (*current).max(1) as u32);
                    }
                    RaceState::Running { .. } => {
                        self.shared_assets.item_frame.render(&ctx);
                        self.shared_assets
                            .render_pos(&ctx, scene.player.place as u32);
                    }
                    RaceState::Completed { place } => {
                        self.shared_assets.render_pos_centered(&ctx, *place as u32);
                    }
                    RaceState::RaceResults { placements } => {
                        let place = placements
                            .iter()
                            .position(|p| p.client_id == scene.own_id)
                            .map(|p| p + 1);

                        if let Some(place) = place {
                            self.shared_assets.render_pos_centered(&ctx, place as u32);
                        }

                        log::warn!("TODO: render race results")
                    }
                }
                unsafe { self.gl.enable(glow::DEPTH_TEST) };
            }

            State::WaitingToJoin => {
                unsafe { self.gl.disable(glow::DEPTH_TEST) };

                self.shared_assets.game_logo.render(&ctx);
                self.shared_assets.join_waiting.render(&ctx);
            }

            State::WaitingToStart { .. } => {
                unsafe { self.gl.disable(glow::DEPTH_TEST) };

                self.shared_assets.game_logo.render(&ctx);
                self.shared_assets.load_waiting.render(&ctx);
            }
            State::Loading { .. } => {
                unsafe { self.gl.disable(glow::DEPTH_TEST) };

                self.shared_assets.game_logo.render(&ctx);
                self.shared_assets.download_waiting.render(&ctx);
            }

            State::MainMenu { show_credits, .. } => {
                unsafe { self.gl.disable(glow::DEPTH_TEST) };

                self.shared_assets.game_logo.render(&ctx);

                if *show_credits {
                    self.shared_assets.credits.render(&ctx);
                } else {
                    self.shared_assets.render_menu(&ctx);
                }
            }
        }

        if !self.hide_cursor {
            self.shared_assets.render_cursor(&ctx);
        }

        unsafe { self.gl.enable(glow::DEPTH_TEST) };
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
            .field("viewport", &self.viewport)
            .field("cam", &self.cam)
            .field("ui_cam", &self.ui_cam)
            .field("rng", &self.rng)
            .field("cache", &self.cache)
            .field("shared_assets", &self.shared_assets)
            .field("state", &self.state)
            .finish()
    }
}
