use crate::engine::{
    mesh::Mesh,
    object::Transform,
    sprite::{SpriteSheet, SpriteSheetUniforms, SPRITE_QUAD},
    Camera, RenderContext,
};
use common::types::*;
use glow::*;

#[derive(Debug)]
pub struct Billboard {
    pub transform: Transform,
    mesh: Mesh,
    pub rotation_offset: f32,
}

impl Billboard {
    pub fn new(gl: &Context, sheet: SpriteSheet) -> Self {
        let aspect = sheet.sprite_dimensions().x as f32 / sheet.sprite_dimensions().y as f32;

        let transform = Transform::new().scale(1.0, 1.0, 1.0 / aspect);

        Self {
            transform,
            mesh: Mesh::new(gl, SPRITE_QUAD, sheet),
            rotation_offset: 0.0,
        }
    }

    pub fn camera_depth(&self, cam: &Camera) -> f32 {
        let to_cam = cam.transform.pos - self.transform.pos;
        to_cam.length_squared()
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

        let sprite_amt = self.mesh.sprite_sheet.sprite_amount();
        let sprite_index =
            (angle / (std::f32::consts::PI * 2.0) * sprite_amt as f32).round() as i32 / 2
                % sprite_amt as i32;
        let sprite_index = (sprite_index + sprite_amt as i32 / 2)
            .max(0)
            .min(sprite_amt as i32 - 1);

        self.mesh
            .bind_index(ctx, sprite_sheet_uniforms, sprite_index as u32);

        // self.mesh.bind(ctx, sprite_sheet_uniforms);

        unsafe {
            ctx.uniform_matrix_4_f32_slice(
                Some(model_loc),
                false,
                &self.model_mat(ctx.cam).to_cols_array(),
            )
        };
    }

    pub fn cleanup(&self, gl: &glow::Context) {
        self.mesh.cleanup(gl);
    }
}

impl core::ops::Deref for Billboard {
    type Target = Transform;
    fn deref(&self) -> &Self::Target {
        &self.transform
    }
}

impl core::ops::DerefMut for Billboard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transform
    }
}
