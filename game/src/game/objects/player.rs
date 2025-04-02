use crate::engine::{
    Camera, CreateContext, RenderContext, UpdateContext,
    object::{Object, Transform},
    sprite::{Billboard, BillboardMode, SpriteSheet},
};
use crate::game::objects::{Coin, ItemBox};
use common::{
    ActiveItemKind, ClientId, ClientMessage, ItemKind, MAP_SCALE, PickupKind, PlayerState,
    map::TrackPosition, map_coord_to_world, types::*, world_coord_to_map,
};
use std::collections::HashMap;

use nalgebra::Point2;
use parry2d::math::{Isometry, Vector};
use parry2d::shape::Ball;
use parry2d::utils::point_in_poly2d;

const ROTATION_OFFSET: f32 = 186.0;
const COLLIDER_RADIUS: f32 = 5.0;

fn load_player(ctx: &CreateContext, transform: Transform) -> Billboard {
    let sprite_sheet = ctx
        .assets
        .load_sheet("player", || SpriteSheet::load_multi(ctx.gl, "player"));

    let mut billboard = Billboard::new(ctx, "player", sprite_sheet);
    billboard.pos = transform.pos;

    billboard.rot = transform.rot;
    billboard.mode = BillboardMode::Rotate {
        offset: ROTATION_OFFSET,
    };

    billboard.scale_uniform(0.5);

    billboard
}

#[derive(Debug)]
pub struct Player {
    billboard: Billboard,

    physical_pos: Vec2,
    physical_rot: f32,

    pub track_pos: TrackPosition,
    pub place: usize,

    pub input: Vec2,
    space_pressed: bool,
    velocity: Vec2,

    offroad_since: Option<f64>,
    pub drift_state: DriftState,
    jump_progress: f32,

    hit_time: f32,
    hit_rotation: f32,
    hit_rotation_target: f32,

    boost_time: f32,

    use_item: bool,
    pub item: Option<ItemKind>,
    pub coins: u32,

    pub camera_angle: f32,

    collider: Ball,
    collision_timeout: f32,
}

#[derive(Debug, PartialEq)]
pub enum DriftState {
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

    fn as_multiplier(&self) -> f32 {
        match self {
            DriftState::Left => -1.0,
            DriftState::Right => 1.0,
            DriftState::None | DriftState::Queued(_) | DriftState::Offroad => 0.0,
        }
    }
}

impl Default for DriftState {
    fn default() -> Self {
        DriftState::None
    }
}

impl Player {
    pub fn new(ctx: &CreateContext, place: usize, pos: Vec2, rot: f32) -> Self {
        let mut transform = Transform::new();
        transform.rot.y = rot;
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
            place: place + 1,

            input: Vec2::new(0.0, 0.0),
            space_pressed: false,
            velocity: Vec2::new(0.0, 0.0),

            boost_time: 0.0,

            coins: 0,
            use_item: false,
            item: None,

            offroad_since: None,
            drift_state: DriftState::None,
            jump_progress: 1.0,
            hit_time: 0.0,
            hit_rotation: 0.0,
            hit_rotation_target: 0.0,

            camera_angle,

            collider: Ball::new(COLLIDER_RADIUS),
            collision_timeout: 0.0,
        }
    }

    pub fn late_update(
        &mut self,
        ctx: &mut UpdateContext,
        players: &HashMap<ClientId, ExternalPlayer>,
        coins: &mut [Coin],
        item_boxes: &mut [ItemBox],
        cam: &mut Camera,
    ) {
        if players.len() > 0 {
            self.place = players
                .values()
                .filter(|player| player.track_pos > self.track_pos)
                .count()
                + 1;
        }

        self.collision_timeout -= ctx.dt;
        let player_collider = Ball::new((4.0 / MAP_SCALE) * 2.0);
        let own_pos = Isometry::new(Vector::new(self.physical_pos.x, self.physical_pos.y), 0.0);
        for other in players.values() {
            let other_pos = Isometry::new(Vector::new(other.pos.x, other.pos.z), 0.0);
            if let Ok(Some(contact)) = parry2d::query::contact(
                &own_pos,
                &player_collider,
                &other_pos,
                &player_collider,
                nalgebra::zero(),
            ) {
                let normal = Vec2::new(contact.normal2.x, contact.normal2.y);
                let depth = -contact.dist / 2.0;
                let other_velocity = other.velocity;
                let other_rotation = other.physical_rot;
                self.apply_collision(normal, depth, other_velocity, other_rotation);
            }
        }

        for (index, coin) in coins.iter_mut().enumerate().filter(|(_, coin)| coin.state) {
            if coin.pos().distance(self.physical_pos) < 0.6 {
                ctx.send_msg(ClientMessage::PickUp {
                    kind: PickupKind::Coin,
                    index,
                });
                coin.state = false;
                self.coins = (self.coins + 1).min(10);
            }
        }

        for (index, item_box) in item_boxes
            .iter_mut()
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
                item_box.state = false;
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

        // self.pos -= camera_forward * 0.5;
        self.pos.y -= 0.18;

        cam.transform.pos =
            Vec3::new(self.physical_pos.x, 0.0, self.physical_pos.y) - camera_forward * 2.5 + Vec3::new(0.0, 1.0, 0.0) /* + camera_shift */;
        cam.transform.rot = Rotation::new(-5.0, self.camera_angle, self.rot.z);

        cam.set_fov(f32::lerp(
            cam.fov(),
            60.0 + self.velocity.y * 0.3
                - if self.offroad_since.is_some() {
                    3.0
                } else {
                    0.0
                },
            ctx.dt * 3.0,
        ));
    }

    pub fn apply_collision(
        &mut self,
        normal: Vec2,
        depth: f32,
        other_velocity: f32,
        other_rotation: f32,
    ) {
        if self.collision_timeout > 0.0 {
            return;
        }

        log::info!(
            "collision: {:?} {:?} {:?} {:?}",
            normal,
            depth,
            other_velocity,
            other_rotation
        );

        self.physical_pos += normal * depth;

        let own_forward = Vec2::new(
            self.physical_rot.to_radians().cos(),
            self.physical_rot.to_radians().sin(),
        );
        let other_forward = Vec2::new(
            other_rotation.to_radians().cos(),
            other_rotation.to_radians().sin(),
        );

        let amt = own_forward.dot(other_forward);
        // let other_new = self.velocity.y * amt;
        self.velocity.y = other_velocity * amt;

        // // add some extra bounce
        // let diff = other_new - self.velocity.y;
        // let factor = if diff > 0.0 { 0.75 } else { 0.25 };
        // self.velocity.y += diff * factor;

        self.collision_timeout = 0.5;
    }

    pub fn hit(&mut self) {
        if self.hit_time > 0.0 {
            return;
        }

        self.hit_time = 1.5;
        self.hit_rotation_target = self.hit_rotation + 360.0 * 2.0;
        self.coins = self.coins.saturating_sub(self.coins / 2 - 1);
    }

    pub fn key_down(&mut self, key: &str, swap: bool) {
        let mut item = false;
        let mut drift = false;

        match key {
            "KeyW" | "ArrowUp" => self.input.y = 1.0,
            "KeyS" | "ArrowDown" => self.input.y = -0.5,
            "KeyA" | "ArrowLeft" => {
                self.input.x = -1.0;
                if matches!(self.drift_state, DriftState::Queued(_)) {
                    self.drift_state = DriftState::Left;
                }
            }
            "KeyD" | "ArrowRight" => {
                self.input.x = 1.0;
                if matches!(self.drift_state, DriftState::Queued(_)) {
                    self.drift_state = DriftState::Right;
                }
            }

            "Space" if !self.space_pressed => {
                self.space_pressed = true;
                if !swap {
                    item = true;
                } else {
                    drift = true;
                }
            }

            "ShiftLeft" | "ShiftRight" => {
                if !swap {
                    drift = true;
                } else {
                    item = true;
                }
            }

            _ => {}
        }

        if item {
            self.use_item = true;
        }

        if drift {
            log::info!("drift");
            self.jump_progress = 0.0;

            if self.drift_state != DriftState::Offroad {
                self.drift_state = if self.input.x < 0.0 {
                    DriftState::Left
                } else if self.input.x > 0.0 {
                    DriftState::Right
                } else {
                    DriftState::Queued(0.1)
                }
            }
        }
    }
    pub fn key_up(&mut self, key: &str, swap: bool) {
        let mut drift = false;

        match key {
            "KeyW" | "ArrowUp" => {
                if self.input.y > 0.0 {
                    self.input.y = 0.0
                }
            }
            "KeyS" | "ArrowDown" => {
                if self.input.y < 0.0 {
                    self.input.y = 0.0
                }
            }
            "KeyA" | "ArrowLeft" => {
                if self.input.x < 0.0 {
                    self.input.x = 0.0
                }
            }
            "KeyD" | "ArrowRight" => {
                if self.input.x > 0.0 {
                    self.input.x = 0.0
                }
            }

            "ShiftLeft" | "ShiftRight" => {
                if !swap {
                    drift = true;
                }
            }
            "Space" => {
                self.space_pressed = false;

                if swap {
                    drift = true;
                }
            }

            _ => {}
        }

        if drift {
            self.drift_state = DriftState::None;
        }
    }
}

impl Object for Player {
    fn update(&mut self, ctx: &mut UpdateContext) {
        const MOVE_ACCEL: f32 = 14.5;
        const COIN_BOOST: f32 = 2.0;
        const POS_BOOST: f32 = 0.15;

        const STEER_ACCEL: f32 = 50.0;

        const DRIFT_ACCEL: f32 = 65.0;

        let coin_boost = match self.coins {
            10 => COIN_BOOST * 1.5,
            c => (c as f32 / 10.0) * COIN_BOOST,
        };

        let boost = if self.boost_time > 0.0 {
            self.boost_time -= ctx.dt;
            15.0
        } else {
            0.0
        };

        let mut move_accel = self.input.y
            * (MOVE_ACCEL + coin_boost + POS_BOOST * (self.place as f32).min(25.0))
            + boost;
        let mut steer_accel = self.input.x * STEER_ACCEL;

        // offroad
        let pos_map = world_coord_to_map(Vec2::new(self.physical_pos.x, self.physical_pos.y));
        let pos_map = Point2::new(pos_map.x, pos_map.y);
        let offroad = ctx
            .offroad
            .iter()
            .any(|offroad| point_in_poly2d(&pos_map, &offroad.0));

        if offroad {
            if self.boost_time <= 0.0 && (self.jump_progress >= 1.0 || self.offroad_since.is_some())
            {
                if self.velocity.y > MOVE_ACCEL * 0.75 {
                    move_accel *= 0.0;
                } else {
                    move_accel *= 0.5;
                }

                self.offroad_since = Some(self.offroad_since.unwrap_or(ctx.time()));
            }

            if let Some(offroad_since) = self.offroad_since {
                if ctx.time() - offroad_since > 100.0 {
                    self.drift_state = DriftState::Offroad;
                }
            }
        } else {
            self.offroad_since = None;
            if self.drift_state == DriftState::Offroad {
                self.drift_state = DriftState::None;
            }
        }

        self.drift_state.update(ctx.dt);
        steer_accel += self.drift_state.as_multiplier() * DRIFT_ACCEL;

        if self.hit_time > 0.0 {
            self.hit_time -= ctx.dt;
            move_accel = 0.0;
            steer_accel = 0.0;
        }

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

        if self.hit_rotation_target > self.hit_rotation {
            self.hit_rotation += ctx.dt * 360.0 * 3.0;
        }

        let target_rot = self.physical_rot
            + self.drift_state.as_multiplier() * 75.0
            + self.input.x * 15.0
            + self.hit_rotation;

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
                visual_rot: self.rot.y,
                vel: self.velocity.y,

                jump_height,
                track_pos: self.track_pos,
            }));
        }

        if self.use_item {
            if let Some(item) = self.item.take() {
                let active_item = match item {
                    ItemKind::Boost => {
                        self.boost_time = 0.8;
                        self.velocity.y += 8.0;
                        None
                    }

                    ItemKind::Banana => Some(ActiveItemKind::Banana),
                    ItemKind::RedShell => Some(ActiveItemKind::RedShell { roll: 0.0 }),
                    ItemKind::GreenShell => Some(ActiveItemKind::GreenShell { roll: 0.0 }),
                };

                if let Some(active_item) = active_item {
                    ctx.send_msg(ClientMessage::UseItem(active_item));
                }
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
    target_pos: Vec2,
    name: String,
    velocity: f32,
    physical_rot: f32,

    track_pos: TrackPosition,
}

impl ExternalPlayer {
    pub fn new(ctx: &CreateContext, name: String, start: Vec2, rot: f32) -> Self {
        let transform = Transform::new()
            .position(start.x, -0.18, start.y)
            .rotation(0.0, rot, 0.0);

        let billboard = load_player(ctx, transform);

        Self {
            billboard,
            name,
            target_pos: start,

            velocity: 0.0,
            physical_rot: 0.0,

            track_pos: TrackPosition::default(),
        }
    }

    pub fn update_state(&mut self, state: PlayerState) {
        self.target_pos = state.pos;
        self.pos.y = state.jump_height - 0.18;
        self.rot = Rotation::new(0.0, state.visual_rot, 0.0);

        self.velocity = state.vel;
        self.physical_rot = state.rot;

        self.track_pos = state.track_pos;
    }
}

impl Object for ExternalPlayer {
    fn update(&mut self, ctx: &mut UpdateContext) {
        self.pos.x = f32::lerp(self.pos.x, self.target_pos.x, ctx.dt * 20.0);
        self.pos.z = f32::lerp(self.pos.z, self.target_pos.y, ctx.dt * 20.0);
    }

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
