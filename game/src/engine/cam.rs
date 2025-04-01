use crate::engine::object::Transform;
use common::types::*;
use glow::*;

#[derive(Debug)]
pub struct Camera {
    pub transform: Transform,
    proj: Mat4,
    fov_rad: f32,
    aspect: f32,
}

#[derive(Debug)]
pub struct UiCamera {
    pub proj: Mat4,
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
            aspect: viewport.x / viewport.y,
        };

        cam.resize(viewport);

        cam
    }

    pub fn set_fov(&mut self, fov: f32) {
        let fov = fov.to_radians();
        if fov != self.fov_rad {
            self.fov_rad = fov;
            self.proj = Mat4::perspective_rh(fov, self.aspect, 0.1, 1000.0);
        }
    }

    pub fn resize(&mut self, viewport: Vec2) {
        self.aspect = viewport.x / viewport.y;
        self.proj = Mat4::perspective_rh(self.fov_rad, self.aspect, 0.1, 1000.0);
    }

    pub fn view(&self) -> Mat4 {
        let direction = Vec3::new(
            self.transform.rot.x.to_radians().cos() * self.transform.rot.y.to_radians().cos(),
            self.transform.rot.x.to_radians().sin(),
            self.transform.rot.x.to_radians().cos() * self.transform.rot.y.to_radians().sin(),
        );

        Mat4::look_to_rh(self.transform.pos, direction, Vec3::new(0.0, 1.0, 0.0))
    }

    pub fn view_no_translation(&self) -> Mat4 {
        let direction = Vec3::new(
            self.transform.rot.x.to_radians().cos() * self.transform.rot.y.to_radians().cos(),
            self.transform.rot.x.to_radians().sin(),
            self.transform.rot.x.to_radians().cos() * self.transform.rot.y.to_radians().sin(),
        );
        Mat4::look_to_rh(Vec3::ZERO, direction, Vec3::new(0.0, 1.0, 0.0))
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

    pub fn bind_no_tranlation(&self, gl: &Context, uniforms: &CameraUniforms) {
        unsafe {
            gl.uniform_matrix_4_f32_slice(Some(&uniforms.proj), false, &self.proj.to_cols_array());
            gl.uniform_matrix_4_f32_slice(
                Some(&uniforms.view),
                false,
                &self.view_no_translation().to_cols_array(),
            );
        }
    }
}

impl UiCamera {
    pub fn new(viewport: Vec2) -> Self {
        let mut cam = Self {
            proj: Mat4::default(),
        };

        cam.resize(viewport);

        cam
    }

    pub fn resize(&mut self, viewport: Vec2) {
        self.proj = Mat4::orthographic_rh(0.0, viewport.x, viewport.y, 0.0, -100.0, 100.0);
    }

    pub fn bind(&self, gl: &Context, uniforms: &CameraUniforms) {
        unsafe {
            gl.uniform_matrix_4_f32_slice(Some(&uniforms.proj), false, &self.proj.to_cols_array());
            gl.uniform_matrix_4_f32_slice(
                Some(&uniforms.view),
                false,
                &Mat4::IDENTITY.to_cols_array(),
            );
        }
    }
}
