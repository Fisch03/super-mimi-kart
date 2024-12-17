use common::types::*;
use glow::*;
use web_sys::WebSocket;

use crate::engine::{
    object::{Object, Transform},
    Camera, RenderContext, Shaders, UpdateContext,
};

mod objects;

pub struct Game {
    ws: WebSocket,
    gl: glow::Context,
    shaders: Shaders,

    cam: Camera,
    objects: Vec<Box<dyn Object>>,
}

impl Game {
    pub fn new(ws: WebSocket, gl: glow::Context, viewport: Vec2) -> Self {
        let shaders = Shaders::new(&gl);

        let player = Transform::new()
            .position(79.7, 0.0, 15.0)
            .rotation(0.0, 270.0, 0.0);

        let mut obj: Vec<Box<dyn Object>> = Vec::new();
        obj.push(Box::new(objects::Map::new(&gl)));
        obj.push(Box::new(objects::Player::new(&gl, player.clone())));

        let cam = Camera::new(60.0, viewport);

        unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            gl.clear_color(0.0, 0.0, 0.0, 1.0);
        }

        Self {
            ws,
            gl,
            shaders,

            cam,
            objects: obj,
        }
    }

    pub fn resize(&mut self, dim: Vec2) {
        unsafe { self.gl.viewport(0, 0, dim.x as i32, dim.y as i32) };
        self.cam.resize(dim);
    }

    pub fn key_down(&mut self, key: String) {
        self.objects.iter_mut().for_each(|o| o.key_down(&key));
    }
    pub fn key_up(&mut self, key: String) {
        self.objects.iter_mut().for_each(|o| o.key_up(&key));
    }

    pub fn render(&mut self, dt: f32) {
        // update
        let mut ctx = UpdateContext {
            dt,
            cam: &mut self.cam,
        };
        self.objects.iter_mut().for_each(|o| o.update(&mut ctx));

        // render
        unsafe {
            self.gl
                .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT | glow::STENCIL_BUFFER_BIT);
        };
        let ctx = RenderContext {
            gl: &self.gl,
            shaders: &self.shaders,
            cam: &self.cam,
        };

        self.objects.iter().for_each(|o| o.render(&ctx));
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        self.ws.close().unwrap();
        self.shaders.cleanup(&self.gl);
        self.objects.iter().for_each(|o| o.cleanup(&self.gl));
    }
}

impl std::fmt::Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Game")
            .field("shaders", &self.shaders)
            .field("cam", &self.cam)
            .field("objects", &self.objects)
            .finish()
    }
}
