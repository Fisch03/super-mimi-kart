use super::Scene;
use crate::engine::{
    object::{Object, Transform},
    Camera,
};
use common::{map::*, types::*, RoundInitParams};
use nalgebra::Point2;
use ncollide2d::shape::Polyline;
use poll_promise::Promise;

pub struct Collider(pub Polyline<f32>);
impl std::fmt::Debug for Collider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Collider").finish()
    }
}

pub trait MapToScene {
    fn to_scene(&self, gl: &glow::Context, viewport: Vec2, params: &RoundInitParams) -> Scene;
}

impl MapToScene for Map {
    fn to_scene(&self, gl: &glow::Context, viewport: Vec2, params: &RoundInitParams) -> Scene {
        use crate::game::objects;
        let mut objects: Vec<Box<dyn Object>> = Vec::new();

        let map_image = &self.assets()[self.background.unwrap()].image;
        let map = objects::Map::new(gl, &map_image);
        let player_start = self.track.iter_starts().nth(params.start_pos).unwrap();
        let player_start = map.map_coord_to_world(player_start);
        objects.push(Box::new(map));

        let player = objects::Player::new(
            gl,
            Transform::new()
                .position(player_start.x, 0.0, player_start.y)
                .rotation(0.0, 270.0, 0.0),
        );

        let cam = Camera::new(60.0, viewport);

        let colliders = self
            .colliders
            .iter()
            .map(|c| {
                let points = c.shape.iter().map(|p| Point2::new(p.x, p.y)).collect();
                log::info!("collider: {:?}", points);
                Collider(Polyline::new(points, None))
            })
            .collect();

        Scene {
            player,
            colliders,
            objects,
            cam,
            map_dimensions: Vec2::new(map_image.width() as f32, map_image.height() as f32),
        }
    }
}

pub struct MapDownload {
    promise: Option<Promise<Result<Map, MapDownloadError>>>,
}

impl core::fmt::Debug for MapDownload {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MapDownload").finish()
    }
}

#[derive(Debug)]
pub enum MapDownloadError {
    Fetch(String),
    Parse(MapLoadError),
}

impl From<MapLoadError> for MapDownloadError {
    fn from(err: MapLoadError) -> Self {
        Self::Parse(err)
    }
}

impl From<wasm_bindgen::JsValue> for MapDownloadError {
    fn from(err: wasm_bindgen::JsValue) -> Self {
        Self::Fetch(format!("{:?}", err))
    }
}

impl MapDownload {
    pub fn start(url: String) -> Self {
        use js_sys::{ArrayBuffer, Uint8Array};
        use std::io::Cursor;
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{Request, RequestInit, RequestMode, Response};

        let promise = Promise::spawn_local(async move {
            let opts = RequestInit::new();
            opts.set_method("GET");
            opts.set_mode(RequestMode::Cors);

            let request = Request::new_with_str_and_init(&url, &opts)?;

            let window = web_sys::window().unwrap();
            let res = JsFuture::from(window.fetch_with_request(&request))
                .await?
                .dyn_into::<Response>()?;

            let array_buffer = JsFuture::from(res.array_buffer()?)
                .await?
                .dyn_into::<ArrayBuffer>()?;
            let buffer = Uint8Array::new(&array_buffer).to_vec();

            let map = Map::load(&mut Cursor::new(&buffer))?;

            Ok(map)
        });

        Self {
            promise: Some(promise),
        }
    }

    pub fn poll(&mut self) -> Option<Result<Map, MapDownloadError>> {
        let promise = self.promise.take()?;

        match promise.try_take() {
            Ok(map) => Some(map),
            Err(promise) => {
                self.promise = Some(promise);
                None
            }
        }
    }
}
