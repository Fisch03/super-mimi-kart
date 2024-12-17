use crate::engine::object::Transform;
use common::types::*;
use glow::*;

#[derive(Debug)]
pub struct Camera {
    pub transform: Transform,
    proj: Mat4,
    fov_rad: f32,
}

pub struct CameraUniforms {
    pub view: UniformLocation,
    pub proj: UniformLocation,
}

impl CameraUniforms {
    pub fn from_program(gl: &Context, program: Program) -> Self {
        let view = unsafe {
            gl.get_uniform_location(program, "view")
                .expect("shader has uniform proj")
        };
        let proj = unsafe {
            gl.get_uniform_location(program, "proj")
                .expect("shader has uniform proj")
        };

        Self { view, proj }
    }
}

impl Camera {
    pub fn new(fov: f32, viewport: Vec2) -> Self {
        let mut cam = Self {
            transform: Transform::new(),
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

    pub fn bind(&self, gl: &Context, uniforms: &CameraUniforms) {
        unsafe {
            gl.uniform_matrix_4_f32_slice(Some(&uniforms.proj), false, &self.proj.to_cols_array());
            gl.uniform_matrix_4_f32_slice(
                Some(&uniforms.view),
                false,
                &self.view().to_cols_array(),
            );
        }
    }
}
