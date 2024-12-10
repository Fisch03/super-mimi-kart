use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
use web_sys::WebSocket;

mod game;
use game::Game;

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) =>  {
        use crate::log;
        log(&format_args!($($t)*).to_string());
    };
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
pub fn start() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    console_log!("hello world!");

    let ctx = {
        use wasm_bindgen::JsCast;
        let canvas = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("GameCanvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        let webgl2_context = canvas
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::WebGl2RenderingContext>()
            .unwrap();

        let gl = glow::Context::from_webgl2_context(webgl2_context);
        gl
    };

    let ws = {
        let ws_protocol = if web_sys::window().unwrap().location().protocol().unwrap() == "https:" {
            "wss"
        } else {
            "ws"
        };

        let server_host = web_sys::window().unwrap().location().host().unwrap();

        WebSocket::new(&format!("{}://{}/ws", ws_protocol, server_host)).unwrap()
    };

    let mut game = Game::new(ws, ctx);

    let main_loop = Rc::new(RefCell::new(None));
    let main_loop_clone = main_loop.clone();

    *main_loop_clone.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        game.render();

        request_animation_frame(main_loop.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(main_loop_clone.borrow().as_ref().unwrap());
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .unwrap();
}
