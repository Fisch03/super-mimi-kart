use common::types::*;
use glow::*;
use web_sys::WebSocket;

use crate::render::{Camera, Object, RenderContext, Shaders, Transform};

mod objects;

pub struct Game {
    ws: WebSocket,
    gl: glow::Context,
    shaders: Shaders,

    cam: Camera,
    map: objects::Map,
}

impl Game {
    pub fn new(ws: WebSocket, gl: glow::Context, viewport: Vec2) -> Self {
        let shaders = Shaders::new(&gl);
        let map = objects::Map::new(&gl);
        let cam = Camera::new(
            Transform::new()
                .position(0.0, 0.0, 12.0)
                .rotation(-5.0, 0.0, 0.0),
            50.0,
            viewport,
        );

        unsafe {
            gl.clear_color(0.0, 0.0, 0.0, 1.0);
        }

        Self {
            ws,
            gl,
            shaders,

            map,
            cam,
        }
    }

    pub fn resize(&mut self, dim: Vec2) {
        unsafe { self.gl.viewport(0, 0, dim.x as i32, dim.y as i32) };
        self.cam.resize(dim);
    }

    pub fn render(&mut self, dt: f32) {
        // update
        self.map.update(dt);
        self.cam.transform.rot.y += dt * 10.0;

        // render
        unsafe { self.gl.clear(glow::COLOR_BUFFER_BIT) };
        let ctx = RenderContext {
            gl: &self.gl,
            shaders: &self.shaders,
            cam: &self.cam,
        };

        self.map.render(&ctx);
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        self.ws.close().unwrap();
        self.shaders.cleanup(&self.gl);
        self.map.cleanup(&self.gl);
    }
}

impl std::fmt::Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Game")
            .field("shaders", &self.shaders)
            .field("cam", &self.cam)
            .field("map", &self.map)
            .finish()
    }
}
