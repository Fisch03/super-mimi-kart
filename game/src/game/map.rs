use super::Scene;
use crate::engine::{Camera, CreateContext, object::Object, sprite::Skybox};
use common::{RoundInitParams, map::*, map_coord_to_world, types::*};
use nalgebra::Point2;
use parry2d::shape::Polyline;
use poll_promise::Promise;

pub struct Collider(pub Polyline);
impl std::fmt::Debug for Collider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Collider").finish()
    }
}

#[derive(Debug)]
pub struct Offroad(pub Vec<Point2<f32>>);

pub trait MapToScene {
    fn to_scene(&self, gl: &CreateContext, viewport: Vec2, params: &RoundInitParams) -> Scene;
}

impl MapToScene for Map {
    fn to_scene(&self, ctx: &CreateContext, viewport: Vec2, params: &RoundInitParams) -> Scene {
        use crate::game::objects;
        let objects: Vec<Box<dyn Object>> = Vec::new();

        let map_image = &self.assets()[self.background.unwrap()].image;
        let map = objects::Map::new(ctx, &map_image);

        let player_start = self.track.iter_starts().nth(params.start_pos).unwrap();
        let player_start = map_coord_to_world(player_start);
        let player = objects::Player::new(ctx, params.start_pos, player_start);

        let players = params
            .players
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != params.start_pos)
            .map(|(i, (id, name))| {
                let start = self.track.iter_starts().nth(i).unwrap();
                let start = map_coord_to_world(start);
                (*id, objects::ExternalPlayer::new(ctx, name.clone(), start))
            })
            .collect();

        let cam = Camera::new(60.0, viewport);

        let colliders = self
            .colliders
            .iter()
            .map(|c| {
                let points = c
                    .shape
                    .iter()
                    .map(|p| Point2::new(p.x, p.y))
                    .chain(std::iter::once(Point2::new(c.shape[0].x, c.shape[0].y)))
                    .collect();
                Collider(Polyline::new(points, None))
            })
            .collect();

        let offroad = self
            .offroad
            .iter()
            .map(|c| {
                let points = c
                    .shape
                    .iter()
                    .map(|p| Point2::new(p.x, p.y))
                    .chain(std::iter::once(Point2::new(c.shape[0].x, c.shape[0].y)))
                    .collect();
                Offroad(points)
            })
            .collect();

        let coin_texture = &self.assets()[self.coin.unwrap()].image;
        let coins = self
            .coins
            .iter()
            .map(|c| {
                let pos = map_coord_to_world(*c);
                objects::Coin::new(ctx, coin_texture, pos)
            })
            .collect();

        let item_box_texture = &self.assets()[self.item_box.unwrap()].image;
        let item_boxes = self
            .item_spawns
            .iter()
            .map(|c| {
                let pos = map_coord_to_world(*c);
                objects::ItemBox::new(ctx, item_box_texture, pos)
            })
            .collect();

        Scene {
            cam,

            own_id: params.client_id,

            player,
            players,

            colliders,
            offroad,

            item_boxes,
            coins,
            items: Vec::new(),

            map,

            static_objects: objects,

            map_dimensions: Vec2::new(map_image.width() as f32, map_image.height() as f32),
        }
    }
}

pub struct MapDownload {
    promise: Option<Promise<Result<Map, MapDownloadError>>>,
}

impl std::fmt::Debug for MapDownload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
