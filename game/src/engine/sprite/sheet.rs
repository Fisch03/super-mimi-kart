use super::SPRITE_ASSETS;
use common::types::*;
use glow::*;
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer};

pub struct SpriteSheet {
    texture: glow::Texture,
    sprite_dimensions: UVec2,
    sprite_amount: u32,
    pub active_sprite: u32,
}

pub struct SpriteSheetUniforms {
    pub sprite_size: UniformLocation,
    pub sprite_sheet_size: UniformLocation,
    pub sprite_index: UniformLocation,
}

impl SpriteSheetUniforms {
    pub fn from_program(gl: &Context, program: Program) -> Self {
        let sprite_size = unsafe {
            gl.get_uniform_location(program, "sprite_size")
                .expect("shader has uniform sprite_size")
        };
        let sprite_sheet_size = unsafe {
            gl.get_uniform_location(program, "sprite_sheet_size")
                .expect("shader has uniform sprite_sheet_size")
        };
        let sprite_index = unsafe {
            gl.get_uniform_location(program, "sprite_index")
                .expect("shader has uniform sprite_index")
        };
        Self {
            sprite_size,
            sprite_sheet_size,
            sprite_index,
        }
    }
}

impl SpriteSheet {
    pub fn sprite_dimensions(&self) -> UVec2 {
        self.sprite_dimensions
    }
    pub fn sprite_amount(&self) -> u32 {
        self.sprite_amount
    }

    pub fn load_single(gl: &Context, asset: &str) -> Self {
        let map_tex = SPRITE_ASSETS.get_file(asset).unwrap().contents();
        let map_img = image::load_from_memory(map_tex).unwrap();

        Self::from_images(gl, &[map_img])
    }

    pub fn load_multi(gl: &Context, asset: &str) -> Self {
        let sprite_dir = SPRITE_ASSETS.get_dir(asset).unwrap();
        let mut sprite_imgs = Vec::new();
        for sprite in sprite_dir.files() {
            let sprite_tex = sprite.contents();
            match image::load_from_memory(sprite_tex) {
                Ok(sprite_img) => sprite_imgs.push(sprite_img),
                Err(e) => {
                    crate::console_log!("skipping invalid sprite '{:?}': {:?}", sprite.path(), e);
                    continue;
                }
            }
        }

        Self::from_images(gl, &sprite_imgs)
    }

    pub fn from_image(
        gl: &Context,
        img: DynamicImage,
        sprite_dimensions: UVec2,
        sprite_amount: u32,
    ) -> Self {
        debug_assert!(sprite_amount > 0, "sprite_amount must be greater than 0");
        debug_assert_eq!(
            sprite_dimensions.y,
            img.height(),
            "image not tall enough to sprites"
        );
        debug_assert_eq!(
            sprite_dimensions.x * sprite_amount,
            img.width(),
            "image not wide enough to fit all sprites"
        );

        let img = img.into_rgba8();

        let texture = unsafe {
            let tex = gl.create_texture().unwrap();
            gl.bind_texture(glow::TEXTURE_2D, Some(tex));

            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                img.width() as i32,
                img.height() as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                PixelUnpackData::Slice(Some(&img.into_raw())),
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
            sprite_dimensions,
            sprite_amount,
            active_sprite: 0,
        }
    }

    pub fn from_images(gl: &Context, imgs: &[DynamicImage]) -> Self {
        debug_assert!(!imgs.is_empty(), "spritesheet must have at least one image");

        let sprite_dimension = imgs[0].dimensions();
        imgs.iter().for_each(|img| {
            debug_assert_eq!(
                img.dimensions(),
                sprite_dimension,
                "images must have the same dimensions"
            );
        });
        let sprite_dimension = UVec2::from(sprite_dimension);

        let mut sheet =
            ImageBuffer::new(sprite_dimension.x * imgs.len() as u32, sprite_dimension.y);
        imgs.iter().enumerate().for_each(|(i, img)| {
            sheet
                .copy_from(img, sprite_dimension.x * i as u32, 0)
                .unwrap();
        });

        Self::from_image(
            gl,
            DynamicImage::ImageRgba8(sheet),
            sprite_dimension,
            imgs.len() as u32,
        )
    }

    pub fn bind(&self, gl: &Context, uniforms: &SpriteSheetUniforms) {
        self.bind_index(gl, uniforms, self.active_sprite);
    }

    pub fn bind_index(&self, gl: &Context, uniforms: &SpriteSheetUniforms, index: u32) {
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
            gl.uniform_2_u32(
                Some(&uniforms.sprite_size),
                self.sprite_dimensions.x,
                self.sprite_dimensions.y,
            );
            gl.uniform_1_u32(Some(&uniforms.sprite_sheet_size), self.sprite_amount);
            gl.uniform_1_u32(Some(&uniforms.sprite_index), index);
        }
    }

    pub fn cleanup(&self, gl: &Context) {
        unsafe {
            gl.delete_texture(self.texture);
        }
    }
}

impl core::fmt::Debug for SpriteSheet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SpriteSheet")
            .field("sprite_dimensions", &self.sprite_dimensions)
            .field("sprite_amount", &self.sprite_amount)
            .field("active_sprite", &self.active_sprite)
            .finish()
    }
}
