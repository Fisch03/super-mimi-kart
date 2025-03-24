use crate::engine::{
    CreateContext, RenderContext,
    cache::MeshRef,
    mesh::{Mesh, MeshData},
    object::{Object, Transform},
    sprite::SpriteSheet,
};
use common::{MAP_SCALE, types::*};
use image::DynamicImage;

#[derive(Debug)]
pub struct Map {
    transform: Transform,
    mesh: MeshRef,
    dimensions: Vec2,
}

impl Map {
    pub fn new(ctx: &CreateContext, texture: &DynamicImage) -> Self {
        let sheet = ctx
            .assets
            .load_sheet("map", || SpriteSheet::from_images(ctx.gl, &[texture]));

        let mesh = ctx
            .assets
            .load_mesh("map", || Mesh::new(ctx, MeshData::QUAD, sheet.clone()));

        let dimensions = sheet.get().sprite_dimensions();
        let dimensions = Vec2::new(dimensions.x as f32, dimensions.y as f32) / MAP_SCALE;
        // let aspect = dimensions.x / dimensions.y;

        let transform = Transform::new()
            .scale(dimensions.x, 1.0, dimensions.y)
            .position(0.0, -0.65, 0.0);

        Self {
            transform,
            mesh,
            dimensions,
        }
    }
}

impl Object for Map {
    fn render(&self, ctx: &RenderContext) {
        self.mesh.get().render(ctx, &self.transform);
    }
}

impl AsRef<Transform> for Map {
    fn as_ref(&self) -> &Transform {
        &self.transform
    }
}
