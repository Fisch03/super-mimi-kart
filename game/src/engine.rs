use crate::game::{Collider, Offroad};
use common::{ClientMessage, map::Map, types::*};
use include_dir::{Dir, include_dir};

static ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/../assets");

pub mod cache;
pub use cache::AssetCache;

mod shaders;
pub use shaders::Shaders;

pub mod mesh;
pub mod object;
pub mod sprite;
pub mod ui;

mod cam;
pub use cam::{Camera, UiCamera};

pub struct CreateContext<'a> {
    pub gl: &'a glow::Context,
    pub assets: &'a AssetCache,

    pub viewport: Vec2,
}

impl CreateContext<'_> {
    pub fn time(&self) -> f64 {
        let performance = web_sys::window().unwrap().performance().unwrap();
        performance.now()
    }
}

impl std::ops::Deref for CreateContext<'_> {
    type Target = glow::Context;
    fn deref(&self) -> &Self::Target {
        self.gl
    }
}

impl AsRef<AssetCache> for CreateContext<'_> {
    fn as_ref(&self) -> &AssetCache {
        self.assets
    }
}

pub struct RenderContext<'a> {
    pub gl: &'a glow::Context,
    pub cam: &'a Camera,
    pub ui_cam: &'a UiCamera,
    pub shaders: &'a Shaders,
    pub assets: &'a AssetCache,
    pub viewport: Vec2,
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

    pub rng: &'a mut rand::rngs::SmallRng,

    pub map: &'a Map,

    pub colliders: &'a [Collider],
    pub offroad: &'a [Offroad],
}

impl UpdateContext<'_> {
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
