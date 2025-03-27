use crate::engine::{
    CreateContext, RenderContext,
    cache::{MeshRef, SheetRef},
    mesh::{Mesh, MeshData},
    object::Transform,
    sprite::SpriteSheet,
};

use common::types::*;

#[derive(Debug)]
pub struct UiVec {
    x: UiDim,
    y: UiDim,
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

pub struct UiSprite {
    pub pos: UiVec,
    pub width: UiDim,
    pub local_anchor: Anchor,
    pub global_anchor: Anchor,

    mesh: MeshRef,
    sheet: SheetRef,

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

            local_anchor: Anchor::TOP_LEFT,
            global_anchor: Anchor::TOP_LEFT,
        }
    }

    pub fn render(&mut self, ctx: &RenderContext) {
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
            .position(position.x.round(), position.y.round(), 0.0)
            .rotation(-90.0, 0.0, 0.0)
            .scale(width, 1.0, width * self.aspect);

        self.mesh.get().render_ui(ctx, &transform);
    }
}
