use super::load;
use crate::engine::{
    RenderContext,
    cam::CameraUniforms,
    mesh::Primitive,
    object::Transform,
    sprite::{Billboard, SpriteSheetUniforms},
};
use glow::*;

pub struct UnlitShader {
    program: Program,

    model_loc: UniformLocation,
    camera_uniforms: CameraUniforms,
    sprite_sheet_uniforms: SpriteSheetUniforms,
}

impl UnlitShader {
    pub(super) fn new(gl: &Context) -> Self {
        let program = load(gl, "unlit");

        let model_loc = unsafe {
            gl.get_uniform_location(program, "model")
                .expect("shader has uniform model")
        };

        Self {
            program,

            model_loc,
            camera_uniforms: CameraUniforms::from_program(gl, program),
            sprite_sheet_uniforms: SpriteSheetUniforms::from_program(gl, program),
        }
    }

    pub fn render(&self, ctx: &RenderContext, transform: &Transform, primitive: &Primitive) {
        unsafe {
            ctx.use_program(Some(self.program));

            primitive.bind(ctx, &self.sprite_sheet_uniforms);
            ctx.cam.bind(ctx, &self.camera_uniforms);
            transform.bind(ctx, &self.model_loc);

            ctx.draw_arrays(glow::TRIANGLES, 0, primitive.vert_count() as i32);
        }
    }
}

impl std::fmt::Debug for UnlitShader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("UnlitShader").finish()
    }
}

pub struct BillboardShader {
    program: Program,

    model_loc: UniformLocation,
    camera_uniforms: CameraUniforms,
    sprite_sheet_uniforms: SpriteSheetUniforms,
}

impl BillboardShader {
    pub(super) fn new(gl: &Context) -> Self {
        let program = load(gl, "unlit");

        let model_loc = unsafe {
            gl.get_uniform_location(program, "model")
                .expect("shader has uniform model")
        };

        Self {
            program,

            model_loc,
            camera_uniforms: CameraUniforms::from_program(gl, program),
            sprite_sheet_uniforms: SpriteSheetUniforms::from_program(gl, program),
        }
    }

    pub fn render(&self, ctx: &RenderContext, obj: &Billboard) {
        unsafe {
            ctx.use_program(Some(self.program));

            obj.bind(ctx, &self.model_loc, &self.sprite_sheet_uniforms);
            ctx.cam.bind(ctx, &self.camera_uniforms);

            ctx.draw_arrays(glow::TRIANGLES, 0, 6);
        }
    }
}

impl std::fmt::Debug for BillboardShader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("BillboardShader").finish()
    }
}
