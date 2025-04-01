use crate::engine::{
    Camera, CreateContext, RenderContext,
    cache::*,
    mesh::{Mesh, MeshData},
    object::{Transform, Object},
    sprite::SpriteSheetUniforms,
};
use common::types::*;
use glow::*;

#[derive(Debug)]
pub struct Billboard {
    pub transform: Transform,
    mesh: MeshRef,
    pub sheet: SheetRef,
    pub mode: BillboardMode,
}

#[derive(Debug)]
pub enum BillboardMode {
    Static {
        index: u32,
    },
    Rotate {
        offset: f32,
    }
}

impl Billboard {
    pub fn new(ctx: &CreateContext, name: &str, sheet: SheetRef) -> Self {
        let aspect = {
            let sheet = sheet.get();
            sheet.sprite_dimensions().x as f32 / sheet.sprite_dimensions().y as f32
        };

        let transform = Transform::new().scale(1.0, 1.0, 1.0 / aspect);

        let mesh = ctx
            .assets
            .load_mesh(name, || Mesh::new(ctx, MeshData::QUAD, sheet.clone()));

        Self {
            transform,
            mesh,
            sheet,
            mode: BillboardMode::Static { index: 0 },
        }
    }

    pub fn next_frame(&mut self) {
        if let BillboardMode::Static { index } = self.mode {
            self.mode = BillboardMode::Static { index: (index + 1) % self.sheet.get().sprite_amount() };
        }
    }

    pub fn render(&self, ctx: &RenderContext) {
        ctx.shaders.unlit.render_billboard(ctx, self);
    }

    pub fn model_mat(&self, cam: &Camera) -> Mat4 {
        Mat4::from_translation(self.transform.pos)
            * Mat4::from_rotation_x(90.0_f32.to_radians())
            * Mat4::from_rotation_y((cam.transform.rot.z).to_radians())
            * Mat4::from_rotation_z((cam.transform.rot.y + 90.0).to_radians())
            * Mat4::from_scale(self.transform.scale)
    }

    pub fn bind(
        &self,
        ctx: &RenderContext,
        model_loc: &UniformLocation,
        sprite_sheet_uniforms: &SpriteSheetUniforms,
    ) {
        let mesh = self.mesh.get();
        let primitive = mesh.primitives.first().unwrap();

        let sprite_index = match self.mode {
            BillboardMode::Static { index } => index,
            BillboardMode::Rotate { offset } => {
                let forward = Vec2::new(1.0, 0.0);
                let to_cam = ctx.cam.transform.pos - self.transform.pos;
                let to_cam = Vec2::new(to_cam.x, to_cam.z); // project onto xz plane, ignore y
                let angle = (to_cam.angle_to(forward)
                    + offset.to_radians()
                    + self.rot.y.to_radians())
                    % (std::f32::consts::PI * 2.0);
        
        
                let sprite_amt = primitive.sprite_ref.get().sprite_amount();
        
                (angle / (std::f32::consts::PI * 2.0) * sprite_amt as f32).floor() as u32
            }
        };

        primitive.bind_index(ctx, sprite_sheet_uniforms, sprite_index);

        // self.mesh.bind(ctx, sprite_sheet_uniforms);

        unsafe {
            ctx.uniform_matrix_4_f32_slice(
                Some(model_loc),
                false,
                &self.model_mat(ctx.cam).to_cols_array(),
            )
        };
    }
}

impl Object for Billboard {
    fn render(&self, ctx: &RenderContext) {
        self.render(ctx);
    }
}

impl AsRef<Transform> for Billboard {
    fn as_ref(&self) -> &Transform {
        &self.transform
    }
}

impl std::ops::Deref for Billboard {
    type Target = Transform;
    fn deref(&self) -> &Self::Target {
        &self.transform
    }
}

impl std::ops::DerefMut for Billboard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transform
    }
}
