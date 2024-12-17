use common::types::*;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{convert::FromWasmAbi, prelude::*};
use web_sys::WebSocket;

mod game;
use game::Game;

mod engine;

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

    let window = web_sys::window().unwrap();
    let performance = window.performance().unwrap();

    let canvas = window
        .document()
        .unwrap()
        .get_element_by_id("GameCanvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();

    let dim = Vec2::new(
        window.inner_width().unwrap().as_f64().unwrap() as f32,
        window.inner_height().unwrap().as_f64().unwrap() as f32,
    );

    let scale = dim / Vec2::new(480.0, 240.0);
    let scale = scale.x.max(scale.y).floor().max(1.0);

    let dim = dim / scale;

    canvas.set_width(dim.x as u32);
    canvas.set_height(dim.y as u32);

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

    type GameRef<'a> = &'a Rc<RefCell<Game>>;
    macro_rules! add_event_listener {
        ($event:literal, $evt_type:ty, $f:expr) => {
            let game_clone = game.clone();
            let on_event = Closure::<dyn FnMut(_)>::new(move |e: $evt_type| {
                $f(&game_clone, e);
            });

            web_sys::window()
                .unwrap()
                .add_event_listener_with_callback($event, on_event.as_ref().unchecked_ref())
                .unwrap();

            on_event.forget();
        };
    }

    // --- resize ---
    add_event_listener!("resize", web_sys::Event, |game: GameRef, _| {
        let dim = Vec2::new(
            window.inner_width().unwrap().as_f64().unwrap() as f32,
            window.inner_height().unwrap().as_f64().unwrap() as f32,
        );

        let scale = dim / Vec2::new(480.0, 240.0);
        let scale = scale.x.max(scale.y).floor().max(1.0);

        let dim = dim / scale;

        canvas.set_width(dim.x as u32);
        canvas.set_height(dim.y as u32);

        game.borrow_mut().resize(dim)
    });

    // --- input ---
    add_event_listener!(
        "keydown",
        web_sys::KeyboardEvent,
        |game: GameRef, e: web_sys::KeyboardEvent| {
            game.borrow_mut().key_down(e.code());
        }
    );
    add_event_listener!(
        "keyup",
        web_sys::KeyboardEvent,
        |game: GameRef, e: web_sys::KeyboardEvent| {
            game.borrow_mut().key_up(e.code());
        }
    );

    request_animation_frame(main_loop.borrow().as_ref().unwrap());
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .unwrap();
}
