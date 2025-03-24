use crate::engine::{
    Camera, CreateContext, RenderContext, UpdateContext,
    object::{Object, Transform},
    sprite::{Billboard, SpriteSheet},
};
use crate::game::objects::{Coin, ItemBox};
use common::{
    ActiveItemKind, ClientId, ClientMessage, ItemKind, PickupKind, PlayerState, map::TrackPosition,
    map_coord_to_world, types::*, world_coord_to_map,
};
use std::collections::HashMap;

use nalgebra::Point2;
use parry2d::math::{Isometry, Vector};
use parry2d::shape::Ball;
use parry2d::utils::point_in_poly2d;

const ROTATION_OFFSET: f32 = 184.0;

fn load_player(ctx: &CreateContext, transform: Transform) -> Billboard {
    let sprite_sheet = ctx
        .assets
        .load_sheet("player", || SpriteSheet::load_multi(ctx.gl, "player"));

    let mut billboard = Billboard::new(ctx, "player", sprite_sheet);
    billboard.pos = transform.pos;

    billboard.rot = transform.rot;
    billboard.rotation_offset = ROTATION_OFFSET;

    billboard.scale_uniform(0.5);

    billboard
}

#[derive(Debug)]
pub struct Player {
    billboard: Billboard,

    physical_pos: Vec2,
    physical_rot: f32,

    track_pos: TrackPosition,
    place: usize,

    input: Vec2,
    velocity: Vec2,

    offroad_since: Option<f64>,
    drift_state: DriftState,
    jump_progress: f32,

    coins: u32,
    use_item: bool,
    item: Option<ItemKind>,

    camera_angle: f32,
    collider: Ball,
}

#[derive(Debug, PartialEq)]
enum DriftState {
    None,
    Offroad,
    Queued(f32), // time in s before queue is cancelled
    Left,
    Right,
}

impl DriftState {
    fn update(&mut self, dt: f32) {
        match self {
            DriftState::Queued(time) => {
                *time -= dt;
                if *time <= 0.0 {
                    *self = DriftState::None;
                }
            }
            DriftState::None | DriftState::Offroad => {}
            DriftState::Left | DriftState::Right => {}
        }
    }

    fn is_drifting(&self) -> bool {
        match self {
            DriftState::Left | DriftState::Right => true,
            DriftState::None | DriftState::Queued(_) | DriftState::Offroad => false,
        }
    }

    fn as_multiplier(&self) -> f32 {
        match self {
            DriftState::Left => -1.0,
            DriftState::Right => 1.0,
            DriftState::None | DriftState::Queued(_) | DriftState::Offroad => 0.0,
        }
    }
}

impl Player {
    pub fn new(ctx: &CreateContext, place: usize, pos: Vec2) -> Self {
        let mut transform = Transform::new();
        transform.rot.y = 270.0; // TODO: load from map
        transform.pos = Vec3::new(pos.x, 0.0, pos.y);

        transform.rot.y += 360.0 * 50.0; // hack for broken rotation offset at negative values
        let physical_rot = transform.rot.y;
        let camera_angle = transform.rot.y - 270.0;
        let billboard = load_player(ctx, transform);

        Self {
            billboard,
            physical_rot,
            physical_pos: pos,

            track_pos: TrackPosition::default(),
            place,

            input: Vec2::new(0.0, 0.0),
            velocity: Vec2::new(0.0, 0.0),

            coins: 0,
            use_item: false,
            item: Some(ItemKind::RedShell),

            offroad_since: None,
            drift_state: DriftState::None,
            jump_progress: 1.0,

            camera_angle,
            collider: Ball::new(5.0),
        }
    }

    pub fn late_update(
        &mut self,
        ctx: &mut UpdateContext,
        players: &HashMap<ClientId, ExternalPlayer>,
        coins: &[Coin],
        item_boxes: &[ItemBox],
        cam: &mut Camera,
    ) {
        self.place = players
            .values()
            .filter(|player| player.track_pos > self.track_pos)
            .count()
            + 1;

        for (index, coin) in coins.iter().enumerate().filter(|(_, coin)| coin.state) {
            if coin.pos().distance(self.physical_pos) < 0.6 {
                ctx.send_msg(ClientMessage::PickUp {
                    kind: PickupKind::Coin,
                    index,
                });
            }
        }

        for (index, item_box) in item_boxes
            .iter()
            .enumerate()
            .filter(|(_, item_box)| item_box.state)
        {
            if item_box.pos().distance(self.physical_pos) < 0.6 {
                if self.item.is_none() {
                    use rand::seq::IndexedRandom;

                    let items = [
                        ItemKind::GreenShell,
                        ItemKind::RedShell,
                        ItemKind::Boost,
                        ItemKind::Banana,
                    ];

                    self.item = Some(*items.choose(ctx.rng).unwrap());
                }
                ctx.send_msg(ClientMessage::PickUp {
                    kind: PickupKind::ItemBox,
                    index,
                });
            }
        }

        let camera_forward = Vec3::new(
            self.camera_angle.to_radians().cos(),
            0.0,
            self.camera_angle.to_radians().sin(),
        );
        // let camera_right = Vec3::new(
        //     (self.camera_angle + 90.0).to_radians().cos(),
        //     0.0,
        //     (self.camera_angle + 90.0).to_radians().sin(),
        // );
        // let camera_shift = camera_right * rot_diff * 0.005;

        self.pos -= camera_forward * 0.5;

        cam.transform.pos =
            Vec3::new(self.physical_pos.x, 0.0, self.physical_pos.y) - camera_forward * 3.0 + Vec3::new(0.0, 1.0, 0.0) /* + camera_shift */;
        cam.transform.rot = Rotation::new(-5.0, self.camera_angle, self.rot.z);
        cam.set_fov(60.0 + self.velocity.y * 0.3);
    }

    pub fn key_down(&mut self, key: &str) {
        match key {
            "KeyW" => self.input.y = 1.0,
            "KeyS" => self.input.y = -0.5,
            "KeyA" => {
                self.input.x = -1.0;
                if matches!(self.drift_state, DriftState::Queued(_)) {
                    self.jump_progress = 0.0;
                    self.drift_state = DriftState::Left;
                }
            }
            "KeyD" => {
                self.input.x = 1.0;
                if matches!(self.drift_state, DriftState::Queued(_)) {
                    self.jump_progress = 0.0;
                    self.drift_state = DriftState::Right;
                }
            }

            "Space" => self.use_item = true,

            "ShiftLeft" if self.drift_state != DriftState::Offroad => {
                self.drift_state = if self.input.x < 0.0 {
                    self.jump_progress = 0.0;
                    DriftState::Left
                } else if self.input.x > 0.0 {
                    self.jump_progress = 0.0;
                    DriftState::Right
                } else {
                    DriftState::Queued(0.1)
                }
            }
            "ShiftLeft" if self.drift_state == DriftState::Offroad => {
                self.jump_progress = 0.0;
            }

            _ => {}
        }
    }
    pub fn key_up(&mut self, key: &str) {
        match key {
            "KeyW" => {
                if self.input.y > 0.0 {
                    self.input.y = 0.0
                }
            }
            "KeyS" => {
                if self.input.y < 0.0 {
                    self.input.y = 0.0
                }
            }
            "KeyA" => {
                if self.input.x < 0.0 {
                    self.input.x = 0.0
                }
            }
            "KeyD" => {
                if self.input.x > 0.0 {
                    self.input.x = 0.0
                }
            }

            "ShiftLeft" => {
                self.drift_state = DriftState::None;
            }

            _ => {}
        }
    }
}

impl Object for Player {
    fn update(&mut self, ctx: &mut UpdateContext) {
        const MOVE_ACCEL: f32 = 15.0;
        const STEER_ACCEL: f32 = 50.0;

        const DRIFT_ACCEL: f32 = 65.0;

        let mut move_accel = self.input.y * MOVE_ACCEL;
        let mut steer_accel = self.input.x * STEER_ACCEL;

        // offroad
        let pos_map = world_coord_to_map(Vec2::new(self.physical_pos.x, self.physical_pos.y));
        let pos_map = Point2::new(pos_map.x, pos_map.y);
        let offroad = ctx
            .offroad
            .iter()
            .any(|offroad| point_in_poly2d(&pos_map, &offroad.0));

        if offroad {
            move_accel *= 0.5;

            self.offroad_since = Some(self.offroad_since.unwrap_or(ctx.time()));
            if ctx.time() - self.offroad_since.unwrap() > 100.0 {
                self.drift_state = DriftState::Offroad;
            }
        } else {
            self.offroad_since = None;
            if self.drift_state == DriftState::Offroad {
                self.drift_state = DriftState::None;
            }
        }

        self.drift_state.update(ctx.dt);
        steer_accel += self.drift_state.as_multiplier() * DRIFT_ACCEL;

        // TODO: use smooth_step for movement but thats broken rn
        self.velocity.y = f32::lerp(self.velocity.y, move_accel, ctx.dt * 2.0);
        self.velocity.x = f32::lerp(self.velocity.x, steer_accel, ctx.dt * 4.0);

        let forward = Vec2::new(
            self.physical_rot.to_radians().cos(),
            self.physical_rot.to_radians().sin(),
        );

        let mut new_pos = self.physical_pos + forward * self.velocity.y * ctx.dt;
        let new_rot = self.physical_rot + self.velocity.x * ctx.dt;

        // collision
        let collider_pos = Isometry::new(nalgebra::zero(), 0.0);
        let new_pos_map = world_coord_to_map(new_pos);
        let own_pos = Isometry::new(Vector::new(new_pos_map.x, new_pos_map.y), 0.0);
        for collider in ctx.colliders {
            use parry2d::query;
            if let Ok(Some(contact)) = query::contact(
                &own_pos,
                &self.collider,
                &collider_pos,
                &collider.0,
                nalgebra::zero(),
            ) {
                let translation_map =
                    Vec2::new(contact.normal2.x, contact.normal2.y) * contact.dist;
                let translation = map_coord_to_world(translation_map);
                new_pos = new_pos - translation;
                self.velocity.y = 0.0;
            }
        }

        let old_pos_map = world_coord_to_map(Vec2::new(self.physical_pos.x, self.physical_pos.y));

        ctx.map
            .track
            .calc_position(old_pos_map, new_pos_map, &mut self.track_pos);

        self.physical_pos = new_pos;
        self.physical_rot = new_rot;

        self.jump_progress += ctx.dt * 5.0;
        self.jump_progress = self.jump_progress.min(1.0);

        let jump_height = f32::sin(self.jump_progress * std::f32::consts::PI) * 0.15;
        self.pos = Vec3::new(self.physical_pos.x, jump_height, self.physical_pos.y);

        let target_rot =
            self.physical_rot + self.drift_state.as_multiplier() * 75.0 + self.input.x * 15.0;
        self.rot.y = f32::lerp(self.rot.y, target_rot, ctx.dt * 5.0);

        // camera
        let target = self.physical_rot + self.drift_state.as_multiplier() * 5.0;
        self.camera_angle = f32::lerp(self.camera_angle, target, ctx.dt * 5.0);
        // self.camera_angle = self.physical_rot.y;
        // self.camera_angle += ctx.dt * 40.0;

        // net
        if ctx.tick {
            ctx.send_msg(ClientMessage::PlayerUpdate(PlayerState {
                pos: self.physical_pos,
                rot: self.physical_rot,

                jump_height,
                track_pos: self.track_pos,
            }));
        }

        if self.use_item {
            if let Some(item) = self.item.take() {
                log::info!("player {:?}", new_pos);
                log::info!("use item: {:?}", item);
                let active_item = match item {
                    ItemKind::Boost => {
                        self.velocity.y += 20.0;
                        None
                    }

                    ItemKind::Banana => Some(ActiveItemKind::Banana),
                    ItemKind::RedShell => Some(ActiveItemKind::RedShell { roll: 0.0 }),
                    ItemKind::GreenShell => Some(ActiveItemKind::GreenShell { roll: 0.0 }),
                };

                if let Some(active_item) = active_item {
                    ctx.send_msg(ClientMessage::UseItem(active_item));
                }

                self.item = Some(item);
            }
            self.use_item = false;
        }
    }

    fn render(&self, ctx: &RenderContext) {
        self.billboard.render(ctx);
    }
}

impl std::ops::Deref for Player {
    type Target = Transform;
    fn deref(&self) -> &Self::Target {
        &self.billboard.transform
    }
}

impl std::ops::DerefMut for Player {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.billboard.transform
    }
}

impl AsRef<Transform> for Player {
    fn as_ref(&self) -> &Transform {
        &self.billboard.transform
    }
}

#[derive(Debug)]
pub struct ExternalPlayer {
    billboard: Billboard,
    name: String,

    track_pos: TrackPosition,
}

impl ExternalPlayer {
    pub fn new(ctx: &CreateContext, name: String, transform: Transform) -> Self {
        let billboard = load_player(ctx, transform);

        Self {
            billboard,
            name,

            track_pos: TrackPosition::default(),
        }
    }

    pub fn update_state(&mut self, state: PlayerState) {
        self.pos = Vec3::new(state.pos.x, state.jump_height, state.pos.y);
        self.rot = Rotation::new(0.0, state.rot, 0.0);
        self.track_pos = state.track_pos;
    }
}

impl Object for ExternalPlayer {
    fn update(&mut self, _ctx: &mut UpdateContext) {}

    fn render(&self, ctx: &RenderContext) {
        self.billboard.render(ctx);
    }
}

impl std::ops::Deref for ExternalPlayer {
    type Target = Transform;
    fn deref(&self) -> &Self::Target {
        &self.billboard.transform
    }
}

impl std::ops::DerefMut for ExternalPlayer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.billboard.transform
    }
}

impl AsRef<Transform> for ExternalPlayer {
    fn as_ref(&self) -> &Transform {
        &self.billboard.transform
    }
}
