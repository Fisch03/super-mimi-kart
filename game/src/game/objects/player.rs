use crate::engine::{
    Camera, CreateContext, RenderContext, UpdateContext,
    object::{Object, Transform},
    sprite::{Billboard, SpriteSheet},
};
use common::{ClientMessage, PlayerState, types::*};

use nalgebra::Point2;
use parry2d::math::{Isometry, Vector};
use parry2d::shape::Ball;
use parry2d::utils::point_in_poly2d;

const ROTATION_OFFSET: f32 = -92.0;

fn load_player(ctx: &CreateContext, transform: Transform) -> Billboard {
    let sprite_sheet = ctx
        .assets
        .load_sheet("player", || SpriteSheet::load_multi(ctx.gl, "player"));

    let mut billboard = Billboard::new(ctx, "player", sprite_sheet);
    billboard.pos = transform.pos;

    billboard.rot = transform.rot;
    billboard.rotation_offset = ROTATION_OFFSET;

    billboard.scale_uniform(0.75);

    billboard
}

#[derive(Debug)]
pub struct Player {
    billboard: Billboard,
    velocity: Vec2,
    acceleration: Vec2,

    camera_angle: f32,
    look_back: bool,
    collider: Ball,
}

impl Player {
    pub fn new(ctx: &CreateContext, transform: Transform) -> Self {
        let billboard = load_player(ctx, transform);

        Self {
            billboard,
            velocity: Vec2::new(0.0, 0.0),
            acceleration: Vec2::new(0.0, 0.0),

            camera_angle: 0.0,
            look_back: false,
            collider: Ball::new(6.0),
        }
    }

    pub fn update_cam(&self, cam: &mut Camera) {
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

        cam.transform.pos =
            self.pos - camera_forward * 5.0 + Vec3::new(0.0, 1.2, 0.0) /* + camera_shift */;
        cam.transform.rot = Rotation::new(-10.0, self.camera_angle, self.rot.z);
        cam.set_fov(60.0 + self.velocity.y * 0.3);
    }

    pub fn key_down(&mut self, key: &str) {
        match key {
            "KeyW" => self.acceleration.y = 0.4,
            "KeyS" => self.acceleration.y = -0.4,
            "KeyA" => {
                self.acceleration.x = -80.0;
                self.velocity.x = self.velocity.x.min(10.0);
            }
            "KeyD" => {
                self.acceleration.x = 80.0;
                self.velocity.x = self.velocity.x.max(-10.0);
            }

            "ArrowDown" => self.look_back = true,
            _ => {}
        }
    }
    pub fn key_up(&mut self, key: &str) {
        match key {
            "KeyW" => {
                if self.acceleration.y > 0.0 {
                    self.acceleration.y = 0.0
                }
            }
            "KeyS" => {
                if self.acceleration.y < 0.0 {
                    self.acceleration.y = 0.0
                }
            }
            "KeyA" => {
                if self.acceleration.x < 0.0 {
                    self.acceleration.x = 0.0
                }
            }
            "KeyD" => {
                if self.acceleration.x > 0.0 {
                    self.acceleration.x = 0.0
                }
            }

            "ArrowDown" => self.look_back = false,

            _ => {}
        }
    }
}

impl Object for Player {
    fn update(&mut self, ctx: &mut UpdateContext) {
        self.velocity += self.acceleration;
        self.velocity.y -= self.velocity.y * 0.015;
        self.velocity.x -= self.velocity.x * 0.09;

        let forward = Vec3::new(
            self.rot.y.to_radians().cos(),
            0.0,
            self.rot.y.to_radians().sin(),
        );

        self.rot.y += self.velocity.x * ctx.dt * 0.1;

        let pos_map = ctx.world_coord_to_map(Vec2::new(self.pos.x, self.pos.z));
        let pos_map = Point2::new(pos_map.x, pos_map.y);
        if ctx
            .offroad
            .iter()
            .any(|offroad| point_in_poly2d(&pos_map, &offroad.0))
        {
            self.velocity.y -= self.velocity.y * 0.02;
        }

        let mut new_pos = self.pos + forward * self.velocity.y * ctx.dt;

        //TODO: it would be a lot nicer to perform the collider pos translations at the beginning
        //but that didnt work the last time i tried it(?)
        let collider_pos = Isometry::new(nalgebra::zero(), 0.0);
        let new_pos_map = ctx.world_coord_to_map(Vec2::new(new_pos.x, new_pos.z));
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
                let translation = ctx.map_coord_to_world(translation_map);
                new_pos = Vec3::new(
                    new_pos.x - translation.x,
                    new_pos.y,
                    new_pos.z - translation.y,
                );

                self.velocity.y = 0.0;
                // self.acceleration.y = 0.0;
            }
        }

        self.pos = new_pos;

        let rot_diff = self.rot.y - self.camera_angle;
        self.camera_angle = self.camera_angle + 0.012 * rot_diff;

        if ctx.tick {
            ctx.send_msg(ClientMessage::PlayerUpdate(PlayerState {
                pos: Vec2::new(self.pos.x, self.pos.z),
                rot: self.rot.y,
            }));
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
}

impl ExternalPlayer {
    pub fn new(ctx: &CreateContext, name: String, transform: Transform) -> Self {
        let billboard = load_player(ctx, transform);

        Self { billboard, name }
    }

    pub fn update_state(&mut self, state: PlayerState) {
        self.pos = Vec3::new(state.pos.x, 0.0, state.pos.y);
        self.rot = Rotation::new(0.0, state.rot, 0.0);
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
