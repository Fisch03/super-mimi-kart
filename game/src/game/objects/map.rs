use crate::engine::{
    mesh::Mesh,
    object::{Collision, Object, Transform},
    sprite::{SpriteSheet, SPRITE_QUAD},
    RenderContext,
};
use common::types::*;
use glow::*;
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
    mesh: Mesh,
    dimensions: Vec2,
}

impl Map {
    pub fn new(gl: &Context, texture: &DynamicImage) -> Self {
        let sheet = SpriteSheet::from_images(gl, &[texture]);

        let dimensions = sheet.sprite_dimensions();
        let dimensions = Vec2::new(dimensions.x as f32, dimensions.y as f32) / MAP_SCALE;
        // let aspect = dimensions.x / dimensions.y;

        let transform = Transform::new()
            .scale(dimensions.x, 1.0, dimensions.y)
            .position(0.0, -0.65, 0.0);

        Self {
            transform,
            mesh: Mesh::new(gl, SPRITE_QUAD, sheet),
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
        ctx.shaders.unlit.render(ctx, self, &self.mesh);
    }

    fn cleanup(&self, gl: &Context) {
        self.mesh.cleanup(gl);
    }
}

impl AsRef<Transform> for Map {
    fn as_ref(&self) -> &Transform {
        &self.transform
    }
}
