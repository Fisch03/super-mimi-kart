use crate::engine::{
    CreateContext, RenderContext, UpdateContext,
    cache::MeshRef,
    mesh::{Mesh, MeshData},
    object::{Object, Transform},
    sprite::SpriteSheet,
};

use image::DynamicImage;

#[derive(Debug)]
pub struct ItemBox {
    transform: Transform,
    mesh: MeshRef,
}

impl ItemBox {
    pub fn new(ctx: &CreateContext, texture: &DynamicImage, transform: Transform) -> Self {
        let sheet = ctx
            .assets
            .load_sheet("item_box", || SpriteSheet::from_images(ctx.gl, &[texture]));

        let mesh = ctx
            .assets
            .load_mesh("item_box", || Mesh::new(ctx, MeshData::CUBE, sheet));

        Self { transform, mesh }
    }
}

impl Object for ItemBox {
    fn update(&mut self, ctx: &mut UpdateContext) {
        self.transform.rot.x += 30.0 * ctx.dt;
        self.transform.rot.y += 30.0 * ctx.dt;
    }

    fn render(&self, ctx: &RenderContext) {
        ctx.shaders.unlit.render(ctx, self, &self.mesh.get());
    }
}

impl AsRef<Transform> for ItemBox {
    fn as_ref(&self) -> &Transform {
        &self.transform
    }
}
