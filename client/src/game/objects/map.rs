use crate::render::{Mesh, MeshData, MeshVert, Object, RenderContext, Texture, Transform};
use glow::*;

const SCALE: f32 = 100.0;

#[rustfmt::skip]
const FLOOR_VERTS: [MeshVert; 4] = [
    MeshVert { pos: [-1.0, 0.0,-1.0], uv: [0.0, 0.0] },
    MeshVert { pos: [-1.0, 0.0, 1.0], uv: [0.0, 1.0] },
    MeshVert { pos: [ 1.0, 0.0, 1.0], uv: [1.0, 1.0] },
    MeshVert { pos: [ 1.0, 0.0,-1.0], uv: [1.0, 0.0] }
];

#[rustfmt::skip]
const FLOOR_INDICES: [u8; 6] = [
    0, 1, 2,
    0, 2, 3,
];

pub struct Map {
    transform: Transform,
    mesh: Mesh,
}

impl Map {
    pub fn new(gl: &Context) -> Self {
        let texture = Texture::new(gl, "maps/mcircuit1/map.png");
        let aspect = texture.width() as f32 / texture.height() as f32;

        let transform = Transform::new()
            .scale(SCALE, 1.0, SCALE / aspect)
            .position(0.0, -5.0, 0.0);

        Self {
            transform,
            mesh: Mesh::new(
                gl,
                MeshData {
                    verts: &FLOOR_VERTS,
                    indices: &FLOOR_INDICES,
                },
                texture,
            ),
        }
    }
}

impl Object for Map {
    fn update(&mut self, dt: f32) {}

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

impl std::fmt::Debug for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Map")
            .field("transform", &self.transform)
            .finish()
    }
}
