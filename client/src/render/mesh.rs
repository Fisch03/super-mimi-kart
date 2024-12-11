use glow::*;

use crate::render::Texture;

#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C)]
pub struct MeshVert {
    pub pos: [f32; 3],
    pub uv: [f32; 2],
}

pub struct MeshData<'a> {
    pub verts: &'a [MeshVert],
    pub indices: &'a [u8],
}

pub struct Mesh {
    vert_buffer: Buffer,
    vert_array: VertexArray,
    index_buffer: Buffer,
    texture: Texture,
}

impl Mesh {
    pub fn new(gl: &Context, data: MeshData, texture: Texture) -> Self {
        let (vert_array, vert_buffer, index_buffer) = unsafe {
            let vert_array = gl.create_vertex_array().unwrap();
            let vert_buffer = gl.create_buffer().unwrap();

            gl.bind_vertex_array(Some(vert_array));

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vert_buffer));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&data.verts),
                glow::STATIC_DRAW,
            );

            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, size_of::<MeshVert>() as i32, 0);
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(
                1,
                2,
                glow::FLOAT,
                false,
                size_of::<MeshVert>() as i32,
                3 * size_of::<f32>() as i32,
            );
            gl.enable_vertex_attrib_array(1);

            let index_buffer = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(index_buffer));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(&data.indices),
                glow::STATIC_DRAW,
            );

            (vert_array, vert_buffer, index_buffer)
        };

        Self {
            vert_buffer,
            vert_array,
            index_buffer,
            texture,
        }
    }

    pub fn bind(&self, gl: &Context) {
        unsafe {
            gl.bind_vertex_array(Some(self.vert_array));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vert_buffer));
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.index_buffer));
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture.gl_texture()));
        }
    }

    pub fn cleanup(&self, gl: &Context) {
        unsafe {
            gl.delete_vertex_array(self.vert_array);
            gl.delete_buffer(self.vert_buffer);
        }
    }
}
