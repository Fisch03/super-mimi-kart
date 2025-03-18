use crate::game::{
    Collider, Offroad,
    objects::{
        Coin, ItemBox,
        map::{map_coord_to_world, world_coord_to_map},
    },
};
use common::{ClientMessage, types::*};

pub mod cache;
pub use cache::AssetCache;

mod shaders;
pub use shaders::Shaders;

pub mod mesh;
pub mod object;
pub mod sprite;

mod cam;
pub use cam::Camera;

pub struct CreateContext<'a> {
    pub gl: &'a glow::Context,
    pub assets: &'a AssetCache,
}

impl AsRef<AssetCache> for CreateContext<'_> {
    fn as_ref(&self) -> &AssetCache {
        self.assets
    }
}

pub struct RenderContext<'a> {
    pub gl: &'a glow::Context,
    pub cam: &'a Camera,
    pub shaders: &'a Shaders,
    pub assets: &'a AssetCache,
}

impl AsRef<AssetCache> for RenderContext<'_> {
    fn as_ref(&self) -> &AssetCache {
        self.assets
    }
}

impl std::ops::Deref for RenderContext<'_> {
    type Target = glow::Context;
    fn deref(&self) -> &Self::Target {
        self.gl
    }
}

pub struct UpdateContext<'a> {
    pub dt: f32,
    pub tick: bool,
    pub send_msg: &'a mut dyn FnMut(ClientMessage),
    pub assets: &'a AssetCache,

    pub colliders: &'a [Collider],
    pub offroad: &'a [Offroad],
}

impl UpdateContext<'_> {
    pub fn world_coord_to_map(&self, pos: Vec2) -> Vec2 {
        world_coord_to_map(pos)
    }

    pub fn map_coord_to_world(&self, pos: Vec2) -> Vec2 {
        map_coord_to_world(pos)
    }

    pub fn time(&self) -> f64 {
        let performance = web_sys::window().unwrap().performance().unwrap();
        performance.now()
    }
}

impl<'a> UpdateContext<'a> {
    pub fn send_msg(&mut self, msg: ClientMessage) {
        (self.send_msg)(msg);
    }
}
