use glow::*;

use web_sys::WebSocket;

mod shaders;
use shaders::Shaders;

mod objects;
use objects::Object;

pub struct Game {
    ws: WebSocket,
    gl: glow::Context,
    shaders: Shaders,
    map: objects::Map,
}

impl Game {
    pub fn new(ws: WebSocket, gl: glow::Context) -> Self {
        let shaders = Shaders::new(&gl);
        let map = objects::Map::new(&gl);

        unsafe {
            gl.clear_color(0.0, 0.0, 0.0, 1.0);
        }

        Self {
            ws,
            gl,
            shaders,
            map,
        }
    }

    pub fn render(&mut self) {
        unsafe {
            self.gl.clear(glow::COLOR_BUFFER_BIT);

            self.map.render(&self.gl, &self.shaders);
        }
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        self.ws.close().unwrap();
        self.shaders.cleanup(&self.gl);
        self.map.cleanup(&self.gl);
    }
}
