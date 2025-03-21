use super::load;
use crate::engine::{RenderContext, cam::CameraUniforms, sprite::Skybox};
use glow::*;

pub struct SkyboxShader {
    program: Program,

    camera_uniforms: CameraUniforms,
}

impl SkyboxShader {
    pub(super) fn new(gl: &Context) -> Self {
        let program = load(gl, "skybox");
        Self {
            program,
            camera_uniforms: CameraUniforms::from_program(gl, program),
        }
    }

    pub fn render(&self, ctx: &RenderContext, cube: &Skybox) {
        unsafe {
            ctx.depth_func(glow::LEQUAL);

            ctx.use_program(Some(self.program));
            cube.bind(ctx);
            ctx.cam.bind_no_tranlation(ctx, &self.camera_uniforms);

            ctx.draw_arrays(glow::TRIANGLES, 0, 36 as i32);

            ctx.depth_func(glow::LESS);
        }
    }
}

impl std::fmt::Debug for SkyboxShader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SkyboxShader")
    }
}
