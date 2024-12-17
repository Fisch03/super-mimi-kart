use include_dir::{include_dir, Dir};

use crate::engine::mesh::{MeshData, MeshVert};

static SPRITE_ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/../assets");

#[rustfmt::skip]
const QUAD_VERTS: [MeshVert; 4] = [
    MeshVert { pos: [-1.0, 0.0,-1.0], uv: [0.0, 0.0] },
    MeshVert { pos: [-1.0, 0.0, 1.0], uv: [0.0, 1.0] },
    MeshVert { pos: [ 1.0, 0.0, 1.0], uv: [1.0, 1.0] },
    MeshVert { pos: [ 1.0, 0.0,-1.0], uv: [1.0, 0.0] }
];

#[rustfmt::skip]
const QUAD_INDICES: [u8; 6] = [
    0, 1, 2,
    0, 2, 3,
];

pub const SPRITE_QUAD: MeshData = MeshData {
    verts: &QUAD_VERTS,
    indices: &QUAD_INDICES,
};

mod sheet;
pub use sheet::{SpriteSheet, SpriteSheetUniforms};

mod billboard;
pub use billboard::Billboard;
