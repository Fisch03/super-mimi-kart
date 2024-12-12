use common::types::*;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{convert::FromWasmAbi, prelude::*};
use web_sys::WebSocket;

mod game;
use game::Game;

mod render;

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) =>  {{
        use crate::log;
        log(&format_args!($($t)*).to_string());
    }};
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

    let window = web_sys::window().unwrap();
    let performance = window.performance().unwrap();

    let canvas = window
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

    let ws = {
        let ws_protocol = if window.location().protocol().unwrap() == "https:" {
            "wss"
        } else {
            "ws"
        };

        let server_host = window.location().host().unwrap();

        WebSocket::new(&format!("{}://{}/ws", ws_protocol, server_host)).unwrap()
    };

    let dim = Vec2::new(canvas.width() as f32, canvas.height() as f32);
    let game = Rc::new(RefCell::new(Game::new(ws, gl, dim)));
    let mut time = performance.now();

    let main_loop = Rc::new(RefCell::new(None));

    // --- main loop ---
    {
        let game_clone = game.clone();
        let main_loop_clone = main_loop.clone();
        *main_loop.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            let new_time = performance.now();
            let dt = (new_time - time) as f32 / 1000.0;
            time = new_time;
            game_clone.borrow_mut().render(dt);

            request_animation_frame(main_loop_clone.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));
    }

    // --- resize ---
    {
        let game_clone = game.clone();
        let on_resize = Closure::<dyn FnMut(_)>::new(move |_: web_sys::Event| {
            let inner_width = window.inner_width().unwrap().as_f64().unwrap();
            let inner_height = window.inner_height().unwrap().as_f64().unwrap();

            let scale_x = inner_width / 480.0;
            let scale_y = inner_height / 240.0;
            let scale = scale_x.max(scale_y).floor().max(1.0);

            let width = inner_width / scale;
            let height = inner_height / scale;

            canvas.set_width(width as u32);
            canvas.set_height(height as u32);

            game_clone
                .borrow_mut()
                .resize(Vec2::new(width as f32, height as f32));
        });
        add_event_listener("resize", on_resize);
    }

    // --- input ---
    {
        let game_clone = game.clone();
        let on_key_down =
            Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
                game_clone.borrow_mut().key_down(e.code());
            });
        add_event_listener("keydown", on_key_down);
    }
    {
        let game_clone = game.clone();
        let on_key_down =
            Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
                game_clone.borrow_mut().key_up(e.code());
            });
        add_event_listener("keyup", on_key_down);
    }

    request_animation_frame(main_loop.borrow().as_ref().unwrap());
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .unwrap();
}

fn add_event_listener<T>(event: &str, f: Closure<dyn FnMut(T)>)
where
    T: FromWasmAbi + 'static,
{
    web_sys::window()
        .unwrap()
        .add_event_listener_with_callback(event, f.as_ref().unchecked_ref())
        .unwrap();
    f.forget();
}
