use crate::game::{
    objects::map::{map_coord_to_world, world_coord_to_map},
    Collider,
};
use common::{types::*, ClientMessage};

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
    pub dt: f32,
    pub tick: bool,
    pub send_msg: &'a mut dyn FnMut(ClientMessage),
    pub colliders: &'a [Collider],
}

impl UpdateContext<'_> {
    pub fn world_coord_to_map(&self, pos: Vec2) -> Vec2 {
        world_coord_to_map(pos)
    }

    pub fn map_coord_to_world(&self, pos: Vec2) -> Vec2 {
        map_coord_to_world(pos)
    }
}

impl<'a> UpdateContext<'a> {
    pub fn send_msg(&mut self, msg: ClientMessage) {
        (self.send_msg)(msg);
    }
}
