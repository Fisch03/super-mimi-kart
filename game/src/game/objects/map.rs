use crate::engine::{
    CreateContext, RenderContext,
    cache::MeshRef,
    mesh::Mesh,
    object::{Object, Transform},
    sprite::{SPRITE_QUAD, SpriteSheet},
};
use common::types::*;
use image::DynamicImage;

pub const MAP_SCALE: f32 = 20.0;

pub fn map_coord_to_world(pos: Vec2) -> Vec2 {
    (pos / MAP_SCALE) * 2.0
}

pub fn world_coord_to_map(pos: Vec2) -> Vec2 {
    (pos / 2.0) * MAP_SCALE
}

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
            .load_mesh("map", || Mesh::new(ctx, SPRITE_QUAD, sheet.clone()));

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

    pub fn map_coord_to_world(&self, pos: Vec2) -> Vec2 {
        map_coord_to_world(pos)
    }

    pub fn world_coord_to_map(&self, pos: Vec2) -> Vec2 {
        world_coord_to_map(pos)
    }
}

impl Object for Map {
    fn render(&self, ctx: &RenderContext) {
        ctx.shaders.unlit.render(ctx, self, &self.mesh.get());
    }
}

impl AsRef<Transform> for Map {
    fn as_ref(&self) -> &Transform {
        &self.transform
    }
}
