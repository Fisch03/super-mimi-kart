use glow::*;

use crate::engine::{
    CreateContext, RenderContext,
    cache::SheetRef,
    sprite::{SpriteSheet, SpriteSheetUniforms},
};

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
    pub sprite_ref: SheetRef,
}

impl Mesh {
    pub fn new(ctx: &CreateContext, data: MeshData, sprite_ref: SheetRef) -> Self {
        let transformed_verts = data.transfom_uv_to_sheet(&sprite_ref.get());

        let (vert_array, vert_buffer, index_buffer) = unsafe {
            let vert_array = ctx.gl.create_vertex_array().unwrap();
            let vert_buffer = ctx.gl.create_buffer().unwrap();

            ctx.gl.bind_vertex_array(Some(vert_array));

            ctx.gl.bind_buffer(glow::ARRAY_BUFFER, Some(vert_buffer));
            ctx.gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&transformed_verts),
                glow::STATIC_DRAW,
            );

            ctx.gl.vertex_attrib_pointer_f32(
                0,
                3,
                glow::FLOAT,
                false,
                size_of::<MeshVert>() as i32,
                0,
            );
            ctx.gl.enable_vertex_attrib_array(0);
            ctx.gl.vertex_attrib_pointer_f32(
                1,
                2,
                glow::FLOAT,
                false,
                size_of::<MeshVert>() as i32,
                3 * size_of::<f32>() as i32,
            );
            ctx.gl.enable_vertex_attrib_array(1);

            let index_buffer = ctx.gl.create_buffer().unwrap();
            ctx.gl
                .bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(index_buffer));
            ctx.gl.buffer_data_u8_slice(
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
            sprite_ref,
        }
    }

    fn bind_common(&self, gl: &Context) {
        unsafe {
            gl.bind_vertex_array(Some(self.vert_array));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vert_buffer));
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.index_buffer));
        }
    }

    pub fn bind(&self, ctx: &RenderContext, sprite_sheet_uniforms: &SpriteSheetUniforms) {
        self.bind_common(ctx.gl);
        self.sprite_ref.get().bind(ctx.gl, sprite_sheet_uniforms);
    }

    pub fn bind_index(
        &self,
        ctx: &RenderContext,
        sprite_sheet_uniforms: &SpriteSheetUniforms,
        index: u32,
    ) {
        self.bind_common(ctx.gl);
        self.sprite_ref
            .get()
            .bind_index(ctx.gl, sprite_sheet_uniforms, index);
    }
}

impl std::fmt::Debug for Mesh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mesh").finish()
    }
}
