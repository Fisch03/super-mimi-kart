use crate::engine::{
    mesh::Mesh,
    object::{Object, Transform},
    sprite::{SpriteSheet, SPRITE_QUAD},
    RenderContext,
};
use common::types::*;
use glow::*;
use image::DynamicImage;

const SCALE: f32 = 100.0;

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
        let dimensions = Vec2::new(dimensions.x as f32, dimensions.y as f32);
        let aspect = dimensions.x / dimensions.y;

        let transform = Transform::new()
            .scale(SCALE, 1.0, SCALE / aspect)
            .position(0.0, -0.65, 0.0);

        Self {
            transform,
            mesh: Mesh::new(gl, SPRITE_QUAD, sheet),
            dimensions,
        }
    }

    pub fn map_coord_to_world(&self, pos: Vec2) -> Vec2 {
        (pos / self.dimensions) * SCALE * 2.0
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
