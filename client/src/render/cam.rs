use crate::render::Transform;
use common::types::*;
use glow::*;

#[derive(Debug)]
pub struct Camera {
    pub transform: Transform,
    proj: Mat4,
    fov_rad: f32,
}

impl Camera {
    pub fn new(transform: Transform, fov: f32, viewport: Vec2) -> Self {
        let mut cam = Self {
            transform,
            proj: Mat4::default(),
            fov_rad: fov.to_radians(),
        };

        cam.resize(viewport);

        cam
    }

    pub fn resize(&mut self, viewport: Vec2) {
        let aspect = viewport.x / viewport.y;
        self.proj = Mat4::perspective_rh(self.fov_rad, aspect, 0.1, 1000.0);
    }

    pub fn view(&self) -> Mat4 {
        let direction = Vec3::new(
            self.transform.rot.x.to_radians().cos() * self.transform.rot.y.to_radians().cos(),
            self.transform.rot.x.to_radians().sin(),
            self.transform.rot.x.to_radians().cos() * self.transform.rot.y.to_radians().sin(),
        );

        Mat4::look_to_rh(self.transform.pos, direction, Vec3::new(0.0, 1.0, 0.0))
    }

    pub fn bind_proj(&self, gl: &Context, loc: &UniformLocation) {
        unsafe {
            gl.uniform_matrix_4_f32_slice(Some(loc), false, &self.proj.to_cols_array());
        }
    }

    pub fn bind_view(&self, gl: &Context, loc: &UniformLocation) {
        unsafe {
            gl.uniform_matrix_4_f32_slice(Some(loc), false, &self.view().to_cols_array());
        }
    }
}
