use crate::engine::{
    CreateContext,
    sprite::{Skybox, UiSprite},
};
use common::types::*;

pub struct SharedAssets {
    pub skybox: Skybox,

    pub game_logo: UiSprite,
}
impl SharedAssets {
    pub fn load(ctx: &CreateContext) -> Self {
        Self {
            skybox: Skybox::load(&ctx, "skybox"),
            game_logo: UiSprite::load_single(&ctx, "logo.png", UVec2::new(0, 0)),
        }
    }
}
