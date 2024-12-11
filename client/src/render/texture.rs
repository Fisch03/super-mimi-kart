use glow::*;
use include_dir::{include_dir, Dir};

static ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/../assets");

pub struct Texture {
    texture: glow::Texture,
    image: image::RgbaImage,
}

impl Texture {
    pub fn width(&self) -> u32 {
        self.image.width()
    }
    pub fn height(&self) -> u32 {
        self.image.height()
    }

    pub fn gl_texture(&self) -> glow::Texture {
        self.texture
    }

    pub fn new(gl: &Context, asset: &str) -> Self {
        let map_tex = ASSETS.get_file(asset).unwrap().contents();
        let map_img = image::load_from_memory(map_tex).unwrap();
        let map_img = map_img.into_rgba8();

        let texture = unsafe {
            let tex = gl.create_texture().unwrap();

            gl.bind_texture(glow::TEXTURE_2D, Some(tex));

            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                map_img.width() as i32,
                map_img.height() as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                PixelUnpackData::Slice(Some(&map_img)),
            );

            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            );

            tex
        };

        Self {
            texture,
            image: map_img,
        }
    }
}
