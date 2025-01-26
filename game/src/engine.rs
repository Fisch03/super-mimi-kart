use common::ClientMessage;

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
    pub send_msg: &'a mut dyn FnMut(ClientMessage),
    pub dt: f32,
    pub tick: bool,
}

impl<'a> UpdateContext<'a> {
    pub fn send_msg(&mut self, msg: ClientMessage) {
        (self.send_msg)(msg);
    }
}
