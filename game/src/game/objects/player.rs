use crate::engine::{
    object::{Object, Transform},
    sprite::{Billboard, SpriteSheet},
    RenderContext, UpdateContext,
};
use common::types::*;

const ROTATION_OFFSET: f32 = -92.0;

#[derive(Debug)]
pub struct Player {
    billboard: Billboard,
    velocity: Vec2,
    acceleration: Vec2,

    camera_angle: f32,
    camera_angle_velocity: f32,
    look_back: bool,
}

impl Player {
    pub fn new(gl: &glow::Context, transform: Transform) -> Self {
        let sprite_sheet = SpriteSheet::load_multi(gl, "player");

        let mut billboard = Billboard::new(gl, sprite_sheet);
        billboard.pos = transform.pos;

        billboard.rot = transform.rot;
        billboard.rotation_offset = ROTATION_OFFSET;

        billboard.scale_uniform(0.75);

        Self {
            billboard,
            velocity: Vec2::new(0.0, 0.0),
            acceleration: Vec2::new(0.0, 0.0),

            camera_angle: 0.0,
            camera_angle_velocity: 0.0,
            look_back: false,
        }
    }
}

impl Object for Player {
    fn key_down(&mut self, key: &str) {
        match key {
            "KeyW" => self.acceleration.y = 0.4,
            "KeyS" => self.acceleration.y = -0.4,
            "KeyA" => {
                self.acceleration.x = -40.0;
                self.velocity.x = self.velocity.x.min(10.0);
            }
            "KeyD" => {
                self.acceleration.x = 40.0;
                self.velocity.x = self.velocity.x.max(-10.0);
            }

            "ArrowLeft" => {
                self.camera_angle_velocity = 0.6;
                self.look_back = false;
            }
            "ArrowRight" => {
                self.camera_angle_velocity = -0.6;
                self.look_back = false;
            }
            "ArrowDown" => self.look_back = true,
            _ => {}
        }
    }
    fn key_up(&mut self, key: &str) {
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

            "ArrowLeft" => {
                if self.camera_angle_velocity > 0.0 {
                    self.camera_angle_velocity = 0.0
                }
            }
            "ArrowRight" => {
                if self.camera_angle_velocity < 0.0 {
                    self.camera_angle_velocity = 0.0
                }
            }
            "ArrowDown" => self.look_back = false,

            _ => {}
        }
    }

    fn update(&mut self, ctx: &mut UpdateContext) {
        self.velocity += self.acceleration;
        self.velocity.y -= self.velocity.y * 0.01;
        self.velocity.x -= self.velocity.x * 0.05;

        let forward = Vec3::new(
            self.rot.y.to_radians().cos(),
            0.0,
            self.rot.y.to_radians().sin(),
        );

        self.rot.y += self.velocity.x * ctx.dt * 0.1;

        let new_pos = self.pos + forward * self.velocity.y * ctx.dt;
        self.pos = new_pos;

        if self.look_back {
            if self.camera_angle > 0.0 && self.camera_angle < 180.0 {
                self.camera_angle_velocity += 50.0 * ctx.dt;
                self.camera_angle_velocity = self.camera_angle_velocity.max(10.0)
            } else if self.camera_angle <= 0.0 && self.camera_angle > -180.0 {
                self.camera_angle_velocity -= 50.0 * ctx.dt;
                self.camera_angle_velocity = self.camera_angle_velocity.min(-10.0)
            } else {
                self.camera_angle_velocity = 0.0;
            }
        }
        self.camera_angle += self.camera_angle_velocity;
        if self.camera_angle_velocity == 0.0 && !self.look_back {
            self.camera_angle *= 0.9;
        }

        let camera_forward = Vec3::new(
            (self.rot.y + self.camera_angle).to_radians().cos(),
            0.0,
            (self.rot.y + self.camera_angle).to_radians().sin(),
        );
        ctx.cam.transform.pos = self.pos - camera_forward * 8.0 + Vec3::new(0.0, 2.0, 0.0);
        ctx.cam.transform.rot = Rotation::new(-10.0, self.rot.y + self.camera_angle, self.rot.z);
    }

    fn render(&self, ctx: &RenderContext) {
        self.billboard.render(ctx);
    }

    fn cleanup(&self, gl: &glow::Context) {
        self.billboard.cleanup(gl);
    }
}

impl core::ops::Deref for Player {
    type Target = Billboard;
    fn deref(&self) -> &Self::Target {
        &self.billboard
    }
}

impl core::ops::DerefMut for Player {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.billboard
    }
}

impl AsRef<Transform> for Player {
    fn as_ref(&self) -> &Transform {
        &self.billboard
    }
}
