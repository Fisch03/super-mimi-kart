use crate::game::Shaders;
use glow::*;

mod map;
pub use map::Map;

pub trait Object {
    fn render(&self, gl: &Context, shaders: &Shaders);
    fn cleanup(&self, gl: &Context);
}

pub struct ObjectBuffers {
    vert_buffer: Buffer,
    vert_array: VertexArray,
    index_buffer: Buffer,
}

impl ObjectBuffers {
    pub fn vert_array(&self) -> VertexArray {
        self.vert_array
    }

    pub fn vert_buffer(&self) -> Buffer {
        self.vert_buffer
    }

    pub fn index_buffer(&self) -> Buffer {
        self.index_buffer
    }

    pub fn new(gl: &Context, verts: &[f32], indices: &[u8]) -> Self {
        let (vert_array, vert_buffer, index_buffer) = unsafe {
            let vert_array = gl.create_vertex_array().unwrap();
            let vert_buffer = gl.create_buffer().unwrap();

            gl.bind_vertex_array(Some(vert_array));

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vert_buffer));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&verts),
                glow::STATIC_DRAW,
            );

            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);
            gl.enable_vertex_attrib_array(0);

            let index_buffer = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(index_buffer));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(&indices),
                glow::STATIC_DRAW,
            );

            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);

            (vert_array, vert_buffer, index_buffer)
        };

        Self {
            vert_buffer,
            vert_array,
            index_buffer,
        }
    }

    pub fn cleanup(&self, gl: &Context) {
        unsafe {
            gl.delete_vertex_array(self.vert_array);
            gl.delete_buffer(self.vert_buffer);
        }
    }
}
