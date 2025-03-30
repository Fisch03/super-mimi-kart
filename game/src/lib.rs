use common::{ServerMessage, types::*};
use std::{cell::RefCell, rc::Rc, sync::mpsc};
use wasm_bindgen::prelude::*;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

mod game;
use game::Game;

mod engine;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen(start)]
pub fn start() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init().unwrap();

    let window = web_sys::window().unwrap();

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

    let context_attrs = web_sys::WebGlContextAttributes::new();
    context_attrs.set_antialias(false);
    let webgl2_context = canvas
        .get_context_with_context_options("webgl2", &context_attrs)
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::WebGl2RenderingContext>()
        .unwrap();

    let gl = glow::Context::from_webgl2_context(webgl2_context);

    let (ws, rx) = {
        let ws_protocol = if window.location().protocol().unwrap() == "https:" {
            "wss"
        } else {
            "ws"
        };

        let server_host = window.location().host().unwrap();

        let ws = WebSocket::new(&format!("{}://{}/ws", ws_protocol, server_host)).unwrap();
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        let (tx, rx) = mpsc::channel();
        let on_message = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
            if let Ok(buf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
                let array = js_sys::Uint8Array::new(&buf);
                match ServerMessage::from_bytes(&array.to_vec()) {
                    Ok(msg) => {
                        tx.send(msg).unwrap();
                    }
                    Err(e) => {
                        log::warn!("Error parsing message: {:?}", e);
                    }
                }
            }
        });
        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        on_message.forget();

        let on_error = Closure::<dyn FnMut(_)>::new(|e: ErrorEvent| {
            log::error!("WebSocket error: {:?}", e);
        });
        ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        on_error.forget();

        (ws, rx)
    };

    let game = Rc::new(RefCell::new(Game::new(ws.clone(), rx, gl, dim)));
    let on_open = Closure::<dyn FnMut()>::new(move || {
        let window = web_sys::window().unwrap();
        let performance = window.performance().unwrap();

        let canvas = window
            .document()
            .unwrap()
            .get_element_by_id("GameCanvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        let canvas = Rc::new(canvas);

        let mut time = performance.now();
        let mut next_tick = time;

        // game.borrow_mut().connect();

        // --- main loop ---
        let main_loop = Rc::new(RefCell::new(None));
        {
            let game_clone = game.clone();
            let main_loop_clone = main_loop.clone();
            *main_loop.borrow_mut() = Some(Closure::wrap(Box::new(move || {
                let new_time = performance.now();
                let tick = new_time >= next_tick;
                if tick {
                    next_tick += 1000.0 / common::TICKS_PER_SECOND as f64;
                }

                let dt = (new_time - time) as f32 / 1000.0;
                time = new_time;
                game_clone.borrow_mut().update(dt, tick);

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
        let canvas_clone = canvas.clone();
        add_event_listener!("resize", web_sys::Event, |game: GameRef, _| {
            let dim = Vec2::new(
                window.inner_width().unwrap().as_f64().unwrap() as f32,
                window.inner_height().unwrap().as_f64().unwrap() as f32,
            );

            let scale = dim / Vec2::new(480.0, 240.0);
            let scale = scale.x.max(scale.y).floor().max(1.0);

            let dim = dim / scale;

            canvas_clone.set_width(dim.x as u32);
            canvas_clone.set_height(dim.y as u32);

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

        let canvas_clone = canvas.clone();
        add_event_listener!(
            "mousemove",
            web_sys::MouseEvent,
            |game: GameRef, e: web_sys::MouseEvent| {
                let rect = canvas_clone.get_bounding_client_rect();

                let scale = Vec2::new(
                    canvas.width() as f32 / rect.width() as f32,
                    canvas.height() as f32 / rect.height() as f32,
                );

                let pos = Vec2::new(
                    (e.client_x() as f32 - rect.left() as f32) * scale.x,
                    (e.client_y() as f32 - rect.top() as f32) * scale.y,
                );

                game.borrow_mut().mouse_move(pos);
            }
        );
        add_event_listener!(
            "mousedown",
            web_sys::MouseEvent,
            |game: GameRef, e: web_sys::MouseEvent| {
                game.borrow_mut().mouse_down();
            }
        );
        add_event_listener!(
            "mouseup",
            web_sys::MouseEvent,
            |game: GameRef, e: web_sys::MouseEvent| {
                game.borrow_mut().mouse_up();
            }
        );

        request_animation_frame(main_loop.borrow().as_ref().unwrap());
    });
    ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
    on_open.forget();
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .unwrap();
}
