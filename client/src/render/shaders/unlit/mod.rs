use super::load;
use crate::render::{Mesh, Object, RenderContext};
use glow::*;

pub struct UnlitShader {
    program: Program,
    model_loc: UniformLocation,
    view_loc: UniformLocation,
    proj_loc: UniformLocation,
}

impl UnlitShader {
    pub(super) fn new(gl: &Context) -> Self {
        let program = load(gl, "unlit");

        let (model_loc, view_loc, proj_loc) = unsafe {
            (
                gl.get_uniform_location(program, "model")
                    .expect("shader has uniform model"),
                gl.get_uniform_location(program, "view")
                    .expect("shader has uniform view"),
                gl.get_uniform_location(program, "proj")
                    .expect("shader has uniform proj"),
            )
        };

        Self {
            program,
            model_loc,
            view_loc,
            proj_loc,
        }
    }

    pub(super) fn cleanup(&self, gl: &Context) {
        unsafe {
            gl.delete_program(self.program);
        }
    }

    pub fn render(&self, ctx: &RenderContext, obj: &dyn Object, mesh: &Mesh) {
        let obj_transform = obj.as_ref();
        unsafe {
            ctx.use_program(Some(self.program));
            mesh.bind(ctx);
            obj_transform.bind(ctx, &self.model_loc);
            ctx.cam.bind_view(ctx, &self.view_loc);
            ctx.cam.bind_proj(ctx, &self.proj_loc);
            ctx.draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_BYTE, 0);
        }
    }
}

impl std::fmt::Debug for UnlitShader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("UnlitShader").finish()
    }
}
