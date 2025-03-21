use crate::engine::{
    CreateContext, RenderContext,
    cache::SheetRef,
    sprite::{SpriteSheetUniforms, SpriteSheet},
    object::Transform,
    ASSETS,
};

use glow::*;
use gltf::Gltf;

#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C)]
pub struct MeshVert {
    pub pos: [f32; 3],
    pub uv: [f32; 2],
}

pub struct MeshData<'a> {
    pub verts: &'a [MeshVert],
}

pub struct Primitive {
    vert_count: usize,
    vert_buffer: Buffer,
    vert_array: VertexArray,
    pub sprite_ref: SheetRef,
}

pub struct Mesh{
    pub primitives: Vec<Primitive>,
}

impl Primitive{
    pub fn new(ctx: &CreateContext, data: MeshData, sprite_ref: SheetRef) -> Self {

        let (vert_array, vert_buffer) = unsafe {
            let vert_array = ctx.gl.create_vertex_array().unwrap();
            let vert_buffer = ctx.gl.create_buffer().unwrap();

            ctx.gl.bind_vertex_array(Some(vert_array));

            ctx.gl.bind_buffer(glow::ARRAY_BUFFER, Some(vert_buffer));
            ctx.gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&data.verts),
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


            (vert_array, vert_buffer)
        };

        Self {
            vert_count: data.verts.len(),
            vert_buffer,
            vert_array,
            sprite_ref,
        }
    }

    pub fn vert_count(&self) -> usize {
        self.vert_count
    }

    fn bind_common(&self, gl: &Context) {
        unsafe {
            gl.bind_vertex_array(Some(self.vert_array));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vert_buffer));
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

impl Mesh{
    pub fn new(ctx: &CreateContext, data: MeshData, sprite_ref: SheetRef) -> Self {
        let primitive = Primitive::new(ctx, data, sprite_ref);

        Self {
            primitives: vec![primitive],
        }
    }


    pub fn load(ctx: &CreateContext, asset: &str) -> Self {
        let file = ASSETS.get_file(asset).unwrap();
        let Gltf {document, mut blob}= Gltf::from_slice(file.contents()).unwrap();

        log::info!("{:#?}", document);

        let buffers = document.buffers().map(|buffer| {
            match buffer.source() {
                gltf::buffer::Source::Bin => blob.take().unwrap(),
                gltf::buffer::Source::Uri(_) => unimplemented!("Uri buffers not supported"),
            }
        }).collect::<Vec<_>>();

        let images = document.images().enumerate().map(|(i, image)| {
            match image.source() {
                gltf::image::Source::View { view, ..} =>{
                    let buffer = &buffers[view.buffer().index()];
                    let start = view.offset();
                    let end = start + view.length();
                    let data = &buffer[start..end];
                    let image = image::load_from_memory(data).unwrap();

                    ctx.assets.load_sheet(&format!("{}_{}", asset, i), || SpriteSheet::from_images(&ctx, &[&image]))
                }
                gltf::image::Source::Uri { .. } => unimplemented!("Uri images not supported"),
            }
        }).collect::<Vec<_>>();


        let mesh = document.meshes().next().unwrap();
        let primitives = mesh.primitives().map(|primitive| {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let indices = reader.read_indices().unwrap().into_u32();

            let positions = reader.read_positions().unwrap().collect::<Vec<_>>();
            let tex_coords = reader.read_tex_coords(0).unwrap().into_f32().collect::<Vec<_>>();

            let mesh_verts = indices.map(|indices| 
                MeshVert {
                    pos: positions[indices as usize],
                    uv: tex_coords[indices as usize],
                }
            ).collect::<Vec<_>>();

            let mesh_data = MeshData {
                verts: &mesh_verts,
            };

            let material = primitive.material();
            
            let image = if let Some(texture) = material.pbr_metallic_roughness().base_color_texture() {
                let index = texture.texture().index();
                images[index].clone()
            } else {
                let color = material.pbr_metallic_roughness().base_color_factor();
                ctx.assets.load_sheet(&format!("{}_color", asset), || {
                    let image = image::Rgba32FImage::from_pixel(1, 1, image::Rgba::from(color));
                    let image = image::DynamicImage::ImageRgba32F(image);
                    SpriteSheet::from_images(&ctx, &[&image])
                })
            };

            Primitive::new(ctx, mesh_data, image)
        }).collect();

        Self { primitives }
    }

    pub fn render(&self, ctx: &RenderContext, transform: &Transform) {
        for primitive in &self.primitives {
            ctx.shaders.unlit.render(ctx, transform, primitive);
        }
    }
}

impl MeshData<'_> {
#[rustfmt::skip]
    pub const QUAD: MeshData<'static> = MeshData {
        verts: &[
            MeshVert { pos: [-1.0, 0.0,-1.0], uv: [0.0, 0.0] },
            MeshVert { pos: [-1.0, 0.0, 1.0], uv: [0.0, 1.0] },
            MeshVert { pos: [ 1.0, 0.0, 1.0], uv: [1.0, 1.0] },

            MeshVert { pos: [-1.0, 0.0,-1.0], uv: [0.0, 0.0] },
            MeshVert { pos: [ 1.0, 0.0, 1.0], uv: [1.0, 1.0] },
            MeshVert { pos: [ 1.0, 0.0,-1.0], uv: [1.0, 0.0] }
        ],
    };

    #[rustfmt::skip]
    pub const CUBE : MeshData<'static> = MeshData {
        verts: &[
            MeshVert { pos: [-1.0,-1.0,-1.0], uv: [0.0, 0.0] },
            MeshVert { pos: [ 1.0,-1.0,-1.0], uv: [1.0, 0.0] },
            MeshVert { pos: [ 1.0, 1.0,-1.0], uv: [1.0, 1.0] },
            MeshVert { pos: [ 1.0, 1.0,-1.0], uv: [1.0, 1.0] },
            MeshVert { pos: [-1.0, 1.0,-1.0], uv: [0.0, 1.0] },
            MeshVert { pos: [-1.0,-1.0,-1.0], uv: [0.0, 0.0] },

            MeshVert { pos: [-1.0,-1.0, 1.0], uv: [0.0, 0.0] },
            MeshVert { pos: [ 1.0,-1.0, 1.0], uv: [1.0, 0.0] },
            MeshVert { pos: [ 1.0, 1.0, 1.0], uv: [1.0, 1.0] },
            MeshVert { pos: [ 1.0, 1.0, 1.0], uv: [1.0, 1.0] },
            MeshVert { pos: [-1.0, 1.0, 1.0], uv: [0.0, 1.0] },
            MeshVert { pos: [-1.0,-1.0, 1.0], uv: [0.0, 0.0] },

            MeshVert { pos: [-1.0, 1.0, 1.0], uv: [1.0, 0.0] },
            MeshVert { pos: [-1.0, 1.0,-1.0], uv: [1.0, 1.0] },
            MeshVert { pos: [-1.0,-1.0,-1.0], uv: [0.0, 1.0] },
            MeshVert { pos: [-1.0,-1.0,-1.0], uv: [0.0, 1.0] },
            MeshVert { pos: [-1.0,-1.0, 1.0], uv: [0.0, 0.0] },
            MeshVert { pos: [-1.0, 1.0, 1.0], uv: [1.0, 0.0] },

            MeshVert { pos: [ 1.0, 1.0, 1.0], uv: [1.0, 0.0] },
            MeshVert { pos: [ 1.0, 1.0,-1.0], uv: [1.0, 1.0] },
            MeshVert { pos: [ 1.0,-1.0,-1.0], uv: [0.0, 1.0] },
            MeshVert { pos: [ 1.0,-1.0,-1.0], uv: [0.0, 1.0] },
            MeshVert { pos: [ 1.0,-1.0, 1.0], uv: [0.0, 0.0] },
            MeshVert { pos: [ 1.0, 1.0, 1.0], uv: [1.0, 0.0] },

            MeshVert { pos: [-1.0,-1.0,-1.0], uv: [0.0, 1.0] },
            MeshVert { pos: [ 1.0,-1.0,-1.0], uv: [1.0, 1.0] },
            MeshVert { pos: [ 1.0,-1.0, 1.0], uv: [1.0, 0.0] },
            MeshVert { pos: [ 1.0,-1.0, 1.0], uv: [1.0, 0.0] },
            MeshVert { pos: [-1.0,-1.0, 1.0], uv: [0.0, 0.0] },
            MeshVert { pos: [-1.0,-1.0,-1.0], uv: [0.0, 1.0] },

            MeshVert { pos: [-1.0, 1.0,-1.0], uv: [0.0, 1.0] },
            MeshVert { pos: [ 1.0, 1.0,-1.0], uv: [1.0, 1.0] },
            MeshVert { pos: [ 1.0, 1.0, 1.0], uv: [1.0, 0.0] },
            MeshVert { pos: [ 1.0, 1.0, 1.0], uv: [1.0, 0.0] },
            MeshVert { pos: [-1.0, 1.0, 1.0], uv: [0.0, 0.0] },
            MeshVert { pos: [-1.0, 1.0,-1.0], uv: [0.0, 1.0] },
        ],
        
        
    };
}


