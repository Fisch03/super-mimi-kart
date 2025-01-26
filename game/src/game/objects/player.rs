use crate::engine::{
    object::{Object, Transform},
    sprite::{Billboard, SpriteSheet},
    RenderContext, UpdateContext,
};
use common::{types::*, ClientMessage, PlayerUpdate};

const ROTATION_OFFSET: f32 = -92.0;

#[derive(Debug)]
pub struct Player {
    billboard: Billboard,
    velocity: Vec2,
    acceleration: Vec2,

    camera_angle: f32,
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

            "ArrowDown" => self.look_back = false,

            _ => {}
        }
    }

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

        let new_pos = self.pos + forward * self.velocity.y * ctx.dt;
        self.pos = new_pos;

        let rot_diff = self.rot.y - self.camera_angle;
        self.camera_angle = self.camera_angle + 0.012 * rot_diff;

        let camera_forward = Vec3::new(
            self.camera_angle.to_radians().cos(),
            0.0,
            self.camera_angle.to_radians().sin(),
        );
        let camera_right = Vec3::new(
            (self.camera_angle + 90.0).to_radians().cos(),
            0.0,
            (self.camera_angle + 90.0).to_radians().sin(),
        );
        let camera_shift = camera_right * rot_diff * 0.005;
        ctx.cam.transform.pos =
            self.pos - camera_forward * 5.0 + Vec3::new(0.0, 1.2, 0.0) + camera_shift;
        ctx.cam.transform.rot = Rotation::new(-10.0, self.camera_angle, self.rot.z);
        ctx.cam.set_fov(60.0 + self.velocity.y * 0.3);

        if ctx.tick {
            ctx.send_msg(ClientMessage::PlayerUpdate(PlayerUpdate {
                pos: Vec2::new(self.pos.x, self.pos.z),
                rot: self.rot.y,
            }));
        }
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
