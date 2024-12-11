mod shaders;
pub use shaders::Shaders;

mod object;
pub use object::{Object, Transform};

mod mesh;
pub use mesh::{Mesh, MeshData, MeshVert};

mod texture;
pub use texture::Texture;

mod cam;
pub use cam::Camera;

pub struct RenderContext<'a> {
    pub gl: &'a glow::Context,
    pub cam: &'a Camera,
    pub shaders: &'a Shaders,
}

impl core::ops::Deref for RenderContext<'_> {
    type Target = glow::Context;
    fn deref(&self) -> &Self::Target {
        self.gl
    }
}
