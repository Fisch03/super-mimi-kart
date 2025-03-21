use super::SPRITE_ASSETS;
use crate::engine::{CreateContext, RenderContext};

use glow::*;

const SKYBOX_FACES: [&str; 6] = ["px.png", "nx.png", "py.png", "ny.png", "pz.png", "nz.png"];

#[rustfmt::skip]
const SKYBOX_VERTS: [f32; 36 * 3] = [
    -1.0,  1.0, -1.0,
    -1.0, -1.0, -1.0,
     1.0, -1.0, -1.0,
     1.0, -1.0, -1.0,
     1.0,  1.0, -1.0,
    -1.0,  1.0, -1.0,

    -1.0, -1.0,  1.0,
    -1.0, -1.0, -1.0,
    -1.0,  1.0, -1.0,
    -1.0,  1.0, -1.0,
    -1.0,  1.0,  1.0,
    -1.0, -1.0,  1.0,

     1.0, -1.0, -1.0,
     1.0, -1.0,  1.0,
     1.0,  1.0,  1.0,
     1.0,  1.0,  1.0,
     1.0,  1.0, -1.0,
     1.0, -1.0, -1.0,

    -1.0, -1.0,  1.0,
    -1.0,  1.0,  1.0,
     1.0,  1.0,  1.0,
     1.0,  1.0,  1.0,
     1.0, -1.0,  1.0,
    -1.0, -1.0,  1.0,

    -1.0,  1.0, -1.0,
     1.0,  1.0, -1.0,
     1.0,  1.0,  1.0,
     1.0,  1.0,  1.0,
    -1.0,  1.0,  1.0,
    -1.0,  1.0, -1.0,

    -1.0, -1.0, -1.0,
    -1.0, -1.0,  1.0,
     1.0, -1.0, -1.0,
     1.0, -1.0, -1.0,
    -1.0, -1.0,  1.0,
     1.0, -1.0,  1.0,
];

#[derive(Debug)]
pub struct Skybox {
    texture: Texture,
    vert_buffer: Buffer,
    vert_array: VertexArray,
}

impl Skybox {
    pub fn load(ctx: &CreateContext, asset: &str) -> Self {
        let dir = SPRITE_ASSETS.get_dir(asset).unwrap();

        let texture = unsafe { ctx.create_texture().unwrap() };
        unsafe { ctx.bind_texture(glow::TEXTURE_CUBE_MAP, Some(texture)) };

        for (i, face) in SKYBOX_FACES
            .iter()
            .map(|f| format!("{}/{}", asset, f))
            .enumerate()
        {
            let face = match dir.get_file(&face) {
                Some(f) => f,
                None => {
                    log::error!("missing skybox face '{}'!", face);
                    continue;
                }
            };
            let face_tex = face.contents();
            let face_img = image::load_from_memory(face_tex).unwrap();
            let (width, height) = (face_img.width(), face_img.height());
            let data = face_img.into_rgb8().into_raw();

            unsafe {
                ctx.tex_image_2d(
                    glow::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
                    0,
                    glow::RGB as i32,
                    width as i32,
                    height as i32,
                    0,
                    glow::RGB,
                    glow::UNSIGNED_BYTE,
                    PixelUnpackData::Slice(Some(&data)),
                )
            }
        }

        unsafe {
            ctx.tex_parameter_i32(
                glow::TEXTURE_CUBE_MAP,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );
            ctx.tex_parameter_i32(
                glow::TEXTURE_CUBE_MAP,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            );

            ctx.tex_parameter_i32(
                glow::TEXTURE_CUBE_MAP,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            ctx.tex_parameter_i32(
                glow::TEXTURE_CUBE_MAP,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );
            ctx.tex_parameter_i32(
                glow::TEXTURE_CUBE_MAP,
                glow::TEXTURE_WRAP_R,
                glow::CLAMP_TO_EDGE as i32,
            );
        }

        let (vert_array, vert_buffer) = unsafe {
            let vert_array = ctx.create_vertex_array().unwrap();
            let vert_buffer = ctx.create_buffer().unwrap();

            ctx.bind_vertex_array(Some(vert_array));
            ctx.bind_buffer(glow::ARRAY_BUFFER, Some(vert_buffer));
            ctx.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&SKYBOX_VERTS),
                glow::STATIC_DRAW,
            );
            ctx.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 3 * size_of::<f32>() as i32, 0);
            ctx.enable_vertex_attrib_array(0);
            (vert_array, vert_buffer)
        };

        Self {
            texture,
            vert_buffer,
            vert_array,
        }
    }

    pub fn render(&self, ctx: &RenderContext) {
        ctx.shaders.skybox.render(ctx, self);
    }

    pub fn bind(&self, gl: &Context) {
        unsafe {
            gl.bind_vertex_array(Some(self.vert_array));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vert_buffer));
            gl.bind_texture(glow::TEXTURE_CUBE_MAP, Some(self.texture))
        }
    }
}
