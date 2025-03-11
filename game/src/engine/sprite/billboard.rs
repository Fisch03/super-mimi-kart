use crate::engine::{
    Camera, CreateContext, RenderContext,
    cache::*,
    mesh::Mesh,
    object::Transform,
    sprite::{SPRITE_QUAD, SpriteSheetUniforms},
};
use common::types::*;
use glow::*;

#[derive(Debug)]
pub struct Billboard {
    pub transform: Transform,
    mesh: MeshRef,
    pub rotation_offset: f32,
}

impl Billboard {
    pub fn new(ctx: &CreateContext, name: &str, sheet_ref: SheetRef) -> Self {
        let aspect = {
            let sheet = sheet_ref.get();
            sheet.sprite_dimensions().x as f32 / sheet.sprite_dimensions().y as f32
        };

        let transform = Transform::new().scale(1.0, 1.0, 1.0 / aspect);

        let mesh = ctx
            .assets
            .load_mesh(name, || Mesh::new(ctx, SPRITE_QUAD, sheet_ref));

        Self {
            transform,
            mesh,
            rotation_offset: 0.0,
        }
    }

    pub fn render(&self, ctx: &RenderContext) {
        ctx.shaders.billboard.render(ctx, self);
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
        let forward = Vec2::new(1.0, 0.0);
        let to_cam = ctx.cam.transform.pos - self.transform.pos;
        let to_cam = Vec2::new(to_cam.x, to_cam.z); // project onto xz plane, ignore y
        let angle = (to_cam.angle_to(forward)
            + self.rotation_offset.to_radians()
            + self.rot.y.to_radians())
            % (std::f32::consts::PI * 2.0);

        let mesh = self.mesh.get();

        let sprite_amt = mesh.sprite_ref.get().sprite_amount();
        let sprite_index =
            (angle / (std::f32::consts::PI * 2.0) * sprite_amt as f32).round() as i32 / 2
                % sprite_amt as i32;
        let sprite_index = (sprite_index + sprite_amt as i32 / 2)
            .max(0)
            .min(sprite_amt as i32 - 1);

        mesh.bind_index(ctx, sprite_sheet_uniforms, sprite_index as u32);

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
