use crate::render::{RenderContext, UpdateContext};
use glow::*;

use common::types::*;

pub trait Object
where
    Self: AsRef<Transform> + core::fmt::Debug,
{
    fn update(&mut self, _ctx: &mut UpdateContext) {}
    fn render(&self, ctx: &RenderContext);

    fn key_down(&mut self, _key: &str) {}
    fn key_up(&mut self, _key: &str) {}

    fn cleanup(&self, gl: &Context);
}

#[derive(Debug, Clone)]
pub struct Transform {
    pub pos: Position,
    pub rot: Rotation,
    pub scale: Vec3,
}

impl Transform {
    pub const fn new() -> Self {
        Self {
            pos: Position::new(0.0, 0.0, 0.0),
            rot: Rotation::new(0.0, 0.0, 0.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn position(self, x: f32, y: f32, z: f32) -> Self {
        Self {
            pos: Position::new(x, y, z),
            ..self
        }
    }
    pub fn rotation(self, x: f32, y: f32, z: f32) -> Self {
        Self {
            rot: Rotation::new(x, y, z),
            ..self
        }
    }
    pub fn scale(self, x: f32, y: f32, z: f32) -> Self {
        Self {
            scale: Vec3::new(x, y, z),
            ..self
        }
    }

    pub fn scale_uniform(&mut self, s: f32) {
        self.scale = Vec3::new(s, s, s);
    }

    pub fn rotate_around(&mut self, point: Vec3, angle: f32) {
        let axis = Vec3::new(0.0, 1.0, 0.0);
        let rot = Quat::from_axis_angle(axis, angle.to_radians());
        self.pos = rot * (self.pos - point) + point;
        self.rot.y -= angle;
    }

    pub fn model_mat(&self) -> Mat4 {
        Mat4::from_translation(self.pos)
            * Mat4::from_rotation_x(self.rot.x.to_radians())
            * Mat4::from_rotation_y(self.rot.y.to_radians())
            * Mat4::from_rotation_z(self.rot.z.to_radians())
            * Mat4::from_scale(self.scale)
    }

    pub fn bind(&self, gl: &Context, loc: &UniformLocation) {
        unsafe {
            gl.uniform_matrix_4_f32_slice(Some(loc), false, &self.model_mat().to_cols_array())
        }
    }
}
