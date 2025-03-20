use crate::engine::{
    CreateContext, RenderContext, UpdateContext,
    object::{Object, Transform},
    sprite::{Billboard, SpriteSheet},
};
use common::types::*;

use image::DynamicImage;

#[derive(Debug)]
pub struct Coin {
    pos: Vec2,
    pub state: bool,
    billboard: Billboard,
}

impl Coin {
    pub fn new(ctx: &CreateContext, texture: &DynamicImage, pos: Vec2) -> Self {
        let sheet = ctx
            .assets
            .load_sheet("coin", || SpriteSheet::from_images(ctx.gl, &[texture]));

        let mut billboard = Billboard::new(ctx, "coin", sheet);
        billboard.scale_uniform(0.35);

        Self {
            pos,
            state: true,
            billboard,
        }
    }

    pub fn pos(&self) -> Vec2 {
        self.pos
    }
}

impl Object for Coin {
    fn update(&mut self, ctx: &mut UpdateContext) {
        let y_offset = f32::sin(ctx.time() as f32 / 100.0) * 0.05;
        self.billboard.pos = Vec3::new(self.pos.x, y_offset, self.pos.y);
    }

    fn render(&self, ctx: &RenderContext) {
        if self.state {
            self.billboard.render(ctx);
        }
    }
}

impl AsRef<Transform> for Coin {
    fn as_ref(&self) -> &Transform {
        &self.billboard.transform
    }
}
