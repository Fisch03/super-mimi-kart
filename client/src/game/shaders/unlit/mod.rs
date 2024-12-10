use super::load;
use crate::game::objects::ObjectBuffers;
use glow::*;

pub struct UnlitShader(Program);

impl UnlitShader {
    pub(super) fn new(gl: &Context) -> Self {
        Self(load(gl, "unlit"))
    }

    pub(super) fn cleanup(&self, gl: &Context) {
        unsafe {
            gl.delete_program(self.0);
        }
    }

    pub fn render(&self, gl: &Context, obj: &ObjectBuffers) {
        unsafe {
            gl.use_program(Some(self.0));

            gl.bind_vertex_array(Some(obj.vert_array()));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(obj.vert_buffer()));
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(obj.index_buffer()));

            gl.draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_BYTE, 0);

            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
        }
    }
}
