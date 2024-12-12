use crate::render::{
    mesh::Mesh,
    object::{Object, Transform},
    sprite::{SpriteSheet, SPRITE_QUAD},
    RenderContext,
};
use glow::*;

const SCALE: f32 = 100.0;

#[derive(Debug)]
pub struct Map {
    transform: Transform,
    mesh: Mesh,
}

impl Map {
    pub fn new(gl: &Context) -> Self {
        let sheet = SpriteSheet::load_single(gl, "maps/mcircuit1/map.png");
        let aspect = sheet.sprite_dimensions().x as f32 / sheet.sprite_dimensions().y as f32;

        let transform = Transform::new()
            .scale(SCALE, 1.0, SCALE / aspect)
            .position(0.0, -0.65, 0.0);

        Self {
            transform,
            mesh: Mesh::new(gl, SPRITE_QUAD, sheet),
        }
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
