use crate::engine::{
    CreateContext, RenderContext,
    cache::{MeshRef, SheetRef},
    mesh::{Mesh, MeshData},
    object::Transform,
    sprite::SpriteSheet,
};

use common::types::*;

pub struct UiSprite {
    pub pos: UVec2,
    mesh: MeshRef,
    sheet: SheetRef,
}

impl UiSprite {
    pub fn load_single(ctx: &CreateContext, name: &str, pos: UVec2) -> Self {
        let sheet = ctx
            .assets
            .load_sheet(name, || SpriteSheet::load_single(ctx, name));

        Self::load_inner(ctx, name, sheet, pos)
    }

    pub fn load_multi(ctx: &CreateContext, name: &str, pos: UVec2) -> Self {
        let sheet = ctx
            .assets
            .load_sheet(name, || SpriteSheet::load_multi(ctx, name));

        Self::load_inner(ctx, name, sheet, pos)
    }

    fn load_inner(ctx: &CreateContext, name: &str, sheet: SheetRef, pos: UVec2) -> Self {
        let sheet = ctx
            .assets
            .load_sheet(name, || SpriteSheet::load_multi(ctx, name));
        let mesh = ctx
            .assets
            .load_mesh(name, || Mesh::new(ctx, MeshData::QUAD, sheet.clone()));
        Self { mesh, sheet, pos }
    }

    pub fn render(&self, ctx: &RenderContext) {
        let transform =
            Transform::new().position(self.pos.x as f32 + 0.5, self.pos.y as f32 + 0.5, 0.0);

        self.mesh.get().render_ui(ctx, &transform);
    }
}
