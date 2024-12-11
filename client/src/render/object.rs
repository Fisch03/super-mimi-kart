use crate::render::RenderContext;
use glow::*;

use common::types::*;

pub trait Object
where
    Self: AsRef<Transform>,
{
    fn update(&mut self, dt: f32) {}
    fn render(&self, ctx: &RenderContext);
    fn cleanup(&self, gl: &Context);
}

#[derive(Debug)]
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
    pub fn scale_uniform(self, s: f32) -> Self {
        Self {
            scale: Vec3::new(s, s, s),
            ..self
        }
    }

    pub fn mat(&self) -> Mat4 {
        Mat4::from_translation(self.pos)
            * Mat4::from_rotation_x(self.rot.x.to_radians())
            * Mat4::from_rotation_y(self.rot.y.to_radians())
            * Mat4::from_rotation_z(self.rot.z.to_radians())
            * Mat4::from_scale(self.scale)
    }

    pub fn bind(&self, gl: &Context, loc: &UniformLocation) {
        unsafe { gl.uniform_matrix_4_f32_slice(Some(loc), false, &self.mat().to_cols_array()) }
    }
}
