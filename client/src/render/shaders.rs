use glow::*;
use include_dir::{include_dir, Dir};

static SHADERS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/render/shaders");

mod unlit;

#[derive(Debug)]
pub struct Shaders {
    pub unlit: unlit::UnlitShader,
}

impl Shaders {
    pub fn new(gl: &Context) -> Self {
        Self {
            unlit: unlit::UnlitShader::new(gl),
        }
    }

    pub fn cleanup(&self, gl: &Context) {
        self.unlit.cleanup(gl);
    }
}

fn load(gl: &Context, name: &str) -> Program {
    let program = unsafe { gl.create_program().expect("Cannot create program") };
    let dir = SHADERS_DIR
        .get_dir(name)
        .expect(&format!("cannot find shader directory: {}", name));

    let mut shaders = Vec::new();
    for file in dir.files() {
        if file.path().extension().unwrap() != "glsl" {
            continue;
        }

        let source = file.contents_utf8().unwrap();

        let shader_type = match file.path().file_stem().unwrap().to_str().unwrap() {
            "vert" => glow::VERTEX_SHADER,
            "frag" => glow::FRAGMENT_SHADER,
            _ => panic!("Unknown shader type"),
        };

        unsafe {
            let shader = gl.create_shader(shader_type).unwrap();
            gl.shader_source(shader, source);
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                panic!(
                    "compiling shader {} for program {} failed: {}",
                    file.path().display(),
                    name,
                    gl.get_shader_info_log(shader)
                );
            }
            gl.attach_shader(program, shader);
            shaders.push(shader);
        };
    }

    unsafe {
        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            panic!(
                "linking program {} failed: {}",
                name,
                gl.get_program_info_log(program)
            );
        }

        shaders
            .into_iter()
            .for_each(|shader| gl.delete_shader(shader));
    };

    program
}
