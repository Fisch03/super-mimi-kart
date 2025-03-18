use crate::engine::{
    CreateContext, RenderContext, UpdateContext,
    object::{Object, Transform},
    sprite::{Billboard, SpriteSheet},
};
use common::types::*;

use image::DynamicImage;

#[derive(Debug)]
pub struct Coin {
    pos: Vec3,
    pub state: bool,
    billboard: Billboard,
}

impl Coin {
    pub fn new(ctx: &CreateContext, texture: &DynamicImage, transform: Transform) -> Self {
        let sheet = ctx
            .assets
            .load_sheet("coin", || SpriteSheet::from_images(ctx.gl, &[texture]));

        let mut billboard = Billboard::new(ctx, "coin", sheet);

        billboard.pos = transform.pos;
        billboard.rot = transform.rot;

        billboard.scale_uniform(0.35);

        Self {
            pos: transform.pos,
            state: true,
            billboard,
        }
    }

    pub fn pos(&self) -> Vec2 {
        Vec2::new(self.pos.x, self.pos.z)
    }
}

impl Object for Coin {
    fn update(&mut self, ctx: &mut UpdateContext) {
        let y_offset = f32::sin(ctx.time() as f32 / 100.0) * 0.05;
        self.billboard.pos = self.pos + Vec3::new(0.0, y_offset, 0.0);
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
