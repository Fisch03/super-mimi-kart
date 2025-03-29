use crate::engine::{
    CreateContext, RenderContext,
    cache::{MeshRef, SheetRef},
    mesh::{Mesh, MeshData},
    object::Transform,
    sprite::SpriteSheet,
};

// use bdf_reader::Font as BdfFont;
// use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer};
// use std::{collections::HashMap, rc::Rc};

use common::types::*;

const CHAR_SET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789.,!?";

#[derive(Debug)]
pub struct UiVec {
    pub x: UiDim,
    pub y: UiDim,
}

impl UiVec {
    pub fn new(x: impl Into<UiDim>, y: impl Into<UiDim>) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }

    fn calculate(&self, full: Vec2, own: Vec2) -> Vec2 {
        Vec2::new(
            self.x.calculate(full.x, own.x),
            self.y.calculate(full.y, own.y),
        )
    }
}

#[derive(Debug)]
pub enum UiDim {
    Pixels(i32),
    Percent(f32),
    Ratio(f32),
}
pub struct Pct(pub f32);
pub struct Px(pub i32);
pub struct Ratio(pub f32);

impl UiDim {
    fn calculate(&self, full: f32, own: f32) -> f32 {
        match self {
            UiDim::Pixels(p) => *p as f32,
            UiDim::Percent(p) => full * (p / 100.0),
            UiDim::Ratio(r) => own * r,
        }
    }
}

impl From<Px> for UiDim {
    fn from(px: Px) -> Self {
        UiDim::Pixels(px.0)
    }
}
impl From<Pct> for UiDim {
    fn from(pct: Pct) -> Self {
        UiDim::Percent(pct.0)
    }
}
impl From<Ratio> for UiDim {
    fn from(ratio: Ratio) -> Self {
        UiDim::Ratio(ratio.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Anchor(Vec2);

impl Anchor {
    pub const CENTER: Self = Self::new(0.0, 0.0);
    pub const TOP_LEFT: Self = Self::new(1.0, 1.0);
    pub const TOP_CENTER: Self = Self::new(0.0, 1.0);
    pub const TOP_RIGHT: Self = Self::new(-1.0, 1.0);
    pub const BOTTOM_LEFT: Self = Self::new(1.0, -1.0);
    pub const BOTTOM_CENTER: Self = Self::new(0.0, -1.0);
    pub const BOTTOM_RIGHT: Self = Self::new(-1.0, -1.0);

    pub const fn new(x: f32, y: f32) -> Self {
        if x.abs() > 1.0 || y.abs() > 1.0 {
            panic!("Anchor values must be between -1.0 and 1.0");
        }

        Self(Vec2::new(x, y))
    }

    pub const fn as_vec(&self) -> Vec2 {
        self.0
    }
}

#[derive(Debug)]
pub struct UiSprite {
    pub pos: UiVec,
    pub width: UiDim,
    pub local_anchor: Anchor,
    pub global_anchor: Anchor,

    mesh: MeshRef,
    pub sheet: SheetRef,

    aspect: f32,
}

impl UiSprite {
    pub fn load_single(ctx: &CreateContext, name: &str, pos: UiVec) -> Self {
        let sheet = ctx
            .assets
            .load_sheet(name, || SpriteSheet::load_single(ctx, name));

        Self::load_inner(ctx, name, sheet, pos)
    }

    pub fn load_multi(ctx: &CreateContext, name: &str, pos: UiVec) -> Self {
        let sheet = ctx
            .assets
            .load_sheet(name, || SpriteSheet::load_multi(ctx, name));

        Self::load_inner(ctx, name, sheet, pos)
    }

    fn load_inner(ctx: &CreateContext, name: &str, sheet: SheetRef, pos: UiVec) -> Self {
        let mesh = ctx
            .assets
            .load_mesh(name, || Mesh::new(ctx, MeshData::QUAD, sheet.clone()));

        let dim = sheet.get().sprite_dimensions();
        let aspect = dim.y as f32 / dim.x as f32;

        Self {
            mesh,
            sheet,
            pos,
            aspect,
            width: UiDim::Pixels(dim.x as i32),

            local_anchor: Anchor::CENTER,
            global_anchor: Anchor::TOP_LEFT,
        }
    }

    pub fn anchor(self, anchor: Anchor) -> Self {
        self.local_anchor(anchor).global_anchor(anchor)
    }

    pub fn local_anchor(self, anchor: Anchor) -> Self {
        Self {
            local_anchor: anchor,
            ..self
        }
    }

    pub fn global_anchor(self, anchor: Anchor) -> Self {
        Self {
            global_anchor: anchor,
            ..self
        }
    }

    pub fn width(&self, ctx: &RenderContext) -> f32 {
        self.width.calculate(
            ctx.viewport.x / 2.0,
            self.sheet.get().sprite_dimensions().x as f32,
        )
    }

    pub fn height(&self, ctx: &RenderContext) -> f32 {
        self.width(ctx) * self.aspect
    }

    pub fn render(&self, ctx: &RenderContext) {
        let sprite_dim = self.sheet.get().sprite_dimensions();
        let sprite_dim = Vec2::new(sprite_dim.x as f32, sprite_dim.y as f32);

        let width = self.width.calculate(ctx.viewport.x / 2.0, sprite_dim.x);

        let half_size = Vec2::new(width, width * self.aspect);
        let local_offset = self.local_anchor.as_vec() * half_size;

        let half_viewport = ctx.viewport / 2.0;
        let global_offset = self.global_anchor.as_vec() * half_viewport;

        let position = half_viewport + self.pos.calculate(ctx.viewport, sprite_dim) - global_offset
            + local_offset;

        let transform = Transform::new()
            .position(position.x.round() + 0.1, position.y.round() + 0.1, 0.0)
            .rotation(-90.0, 0.0, 0.0)
            .scale(width, 1.0, width * self.aspect);

        self.mesh.get().render_ui(ctx, &transform);
    }
}

// #[derive(Debug, Clone)]
// pub struct FontAtlas(Rc<HashMap<char, (MeshRef, SheetRef)>>);
//
// #[derive(Debug)]
// pub struct Font {
//     pub atlas: FontAtlas,
//
//     pub pos: UiVec,
//     pub local_anchor: Anchor,
//     pub global_anchor: Anchor,
// }
//
// impl Font {
//     pub fn load(ctx: &CreateContext, name: &str, pos: UiVec) -> Self {
//         use std::io::{BufReader, Cursor};
//
//         let bdf_file = ASSETS.get_file(name).unwrap().contents();
//         let bdf_file = BufReader::new(Cursor::new(bdf_file));
//         let font = BdfFont::read(bdf_file).unwrap();
//
//         log::info!("'{}' loaded - {:?}", name, font.size());
//
//         let atlas = CHAR_SET
//             .chars()
//             .map(|c| {
//                 let sheet = ctx.assets.load_sheet(&format!("{}_{}", name, c), || {
//                     let glyph = font.glyph(c).unwrap();
//                     let bitmap = glyph.bitmap();
//                     let bounds = glyph.bounding_box();
//                     let img = ImageBuffer::from_fn(bounds.width, bounds.height, |x, y| {
//                         let filled = bitmap.get(x as usize, y as usize).unwrap_or(false);
//                         if filled {
//                             image::Rgb([255, 255, 255])
//                         } else {
//                             image::Rgb([0, 0, 0])
//                         }
//                     });
//                     SpriteSheet::from_images(ctx, &[&DynamicImage::ImageRgb8(img)])
//                 });
//
//                 let mesh = ctx.assets.load_mesh(&format!("{}_{}", name, c), || {
//                     Mesh::new(ctx, MeshData::QUAD, sheet.clone())
//                 });
//
//                 (c, (mesh, sheet))
//             })
//             .collect::<HashMap<_, _>>();
//
//         Self {
//             atlas: FontAtlas(Rc::new(atlas)),
//             pos,
//             local_anchor: Anchor::CENTER,
//             global_anchor: Anchor::TOP_LEFT,
//         }
//     }
//
//     pub fn anchor(self, anchor: Anchor) -> Self {
//         self.local_anchor(anchor).global_anchor(anchor)
//     }
//
//     pub fn local_anchor(self, anchor: Anchor) -> Self {
//         Self {
//             local_anchor: anchor,
//             ..self
//         }
//     }
//
//     pub fn global_anchor(self, anchor: Anchor) -> Self {
//         Self {
//             global_anchor: anchor,
//             ..self
//         }
//     }
//
//     pub fn render(&self, ctx: &RenderContext, text: &str) {
//         let text_width = text
//             .chars()
//             .map(|c| {
//                 self.atlas.0.get(&c).map_or(0.0, |(_, sheet)| {
//                     sheet.get().sprite_dimensions().x as f32 * 2.0
//                 })
//             })
//             .sum::<f32>();
//
//         let text_height = self
//             .atlas
//             .0
//             .values()
//             .next()
//             .unwrap()
//             .1
//             .get()
//             .sprite_dimensions()
//             .y as f32;
//
//         let text_dim = Vec2::new(text_width, text_height);
//
//         let local_offset = self.local_anchor.as_vec() * text_dim;
//
//         let half_viewport = ctx.viewport / 2.0;
//         let global_offset = self.global_anchor.as_vec() * half_viewport;
//
//         let position = half_viewport + self.pos.calculate(ctx.viewport, text_dim) - global_offset
//             + local_offset;
//
//         let mut x = position.x;
//         for c in text.chars() {
//             if let Some((mesh, sheet)) = self.atlas.0.get(&c) {
//                 let sprite_dim = sheet.get().sprite_dimensions();
//
//                 let position = Vec2::new(x, position.y);
//                 let transform = Transform::new()
//                     .position(position.x.round(), position.y.round(), 0.0)
//                     .rotation(-90.0, 0.0, 0.0)
//                     .scale(sprite_dim.x as f32, 1.0, sprite_dim.y as f32);
//
//                 mesh.get().render_ui(ctx, &transform);
//                 x += sprite_dim.x as f32 * 2.0;
//             }
//         }
//     }
// }
