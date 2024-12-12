use glow::*;

use crate::render::sprite::{SpriteSheet, SpriteSheetUniforms};

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

impl MeshData<'_> {
    fn transfom_uv_to_sheet(&self, sheet: &SpriteSheet) -> Vec<MeshVert> {
        let u_scale = 1.0 / sheet.sprite_amount() as f32;
        self.verts
            .iter()
            .map(|vert| MeshVert {
                pos: vert.pos,
                uv: [vert.uv[0] * u_scale, vert.uv[1]],
            })
            .collect()
    }
}

pub struct Mesh {
    vert_buffer: Buffer,
    vert_array: VertexArray,
    index_buffer: Buffer,
    pub sprite_sheet: SpriteSheet,
}

impl Mesh {
    pub fn new(gl: &Context, data: MeshData, sprite_sheet: SpriteSheet) -> Self {
        let transformed_verts = data.transfom_uv_to_sheet(&sprite_sheet);

        let (vert_array, vert_buffer, index_buffer) = unsafe {
            let vert_array = gl.create_vertex_array().unwrap();
            let vert_buffer = gl.create_buffer().unwrap();

            gl.bind_vertex_array(Some(vert_array));

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vert_buffer));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&transformed_verts),
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
            sprite_sheet,
        }
    }

    fn bind_common(&self, gl: &Context) {
        unsafe {
            gl.bind_vertex_array(Some(self.vert_array));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vert_buffer));
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.index_buffer));
        }
    }

    pub fn bind(&self, gl: &Context, sprite_sheet_uniforms: &SpriteSheetUniforms) {
        self.bind_common(gl);
        self.sprite_sheet.bind(gl, sprite_sheet_uniforms);
    }

    pub fn bind_index(
        &self,
        gl: &Context,
        sprite_sheet_uniforms: &SpriteSheetUniforms,
        index: u32,
    ) {
        self.bind_common(gl);
        self.sprite_sheet
            .bind_index(gl, sprite_sheet_uniforms, index);
    }

    pub fn cleanup(&self, gl: &Context) {
        unsafe {
            gl.delete_vertex_array(self.vert_array);
            gl.delete_buffer(self.vert_buffer);
            gl.delete_buffer(self.index_buffer);
            self.sprite_sheet.cleanup(gl);
        }
    }
}

impl core::fmt::Debug for Mesh {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Mesh").finish()
    }
}
