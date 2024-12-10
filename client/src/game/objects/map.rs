use super::{Object, ObjectBuffers};
use crate::game::Shaders;
use glow::*;

#[rustfmt::skip]
const FLOOR_VERTS: [f32; 12] = [
    -1.0, -1.0,  0.0,
    -1.0,  1.0,  0.0,
     1.0,  1.0,  0.0,
     1.0, -1.0,  0.0,
];

#[rustfmt::skip]
const FLOOR_INDICES: [u8; 6] = [
    0, 1, 2,
    0, 2, 3,
];

pub struct Map {
    buffers: ObjectBuffers,
}

impl Map {
    pub fn new(gl: &Context) -> Self {
        Self {
            buffers: ObjectBuffers::new(gl, &FLOOR_VERTS, &FLOOR_INDICES),
        }
    }
}

impl Object for Map {
    fn render(&self, gl: &Context, shaders: &Shaders) {
        shaders.unlit.render(gl, &self.buffers);
    }

    fn cleanup(&self, gl: &Context) {
        self.buffers.cleanup(gl);
    }
}
