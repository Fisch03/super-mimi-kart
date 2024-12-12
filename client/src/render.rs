mod shaders;
pub use shaders::Shaders;

pub mod mesh;
pub mod object;
pub mod sprite;

mod cam;
pub use cam::Camera;

pub struct RenderContext<'a> {
    pub gl: &'a glow::Context,
    pub cam: &'a Camera,
    pub shaders: &'a Shaders,
}

impl core::ops::Deref for RenderContext<'_> {
    type Target = glow::Context;
    fn deref(&self) -> &Self::Target {
        self.gl
    }
}

pub struct UpdateContext<'a> {
    pub cam: &'a mut Camera,
    pub dt: f32,
}
