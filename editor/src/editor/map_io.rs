use common::map::*;
use indexed_db_futures::{
    KeyPath, KeyPathSeq,
    database::Database,
    error::{Error as DbError, OpenDbError},
    prelude::*,
    transaction::TransactionMode,
};
use js_sys::{Array, ArrayBuffer, Uint8Array};
use serde::{Deserialize, Serialize};
use std::{io::Cursor, sync::mpsc};
use thiserror::Error;
use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::{Blob, BlobPropertyBag, FileReader, HtmlElement, Url};

const MAP_ID: u32 = 1;

#[derive(Debug, Clone)]
pub struct MapDB {
    db: Database,
}

#[derive(Debug, Serialize, Deserialize)]
struct MapRef {
    id: u32,
    map: Map,
}

#[derive(Debug, Serialize, Deserialize)]
struct AssetsDataRef {
    map_id: u32,
    assets: Vec<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AssetsNameRef {
    map_id: u32,
    assets: Vec<String>,
}

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("Failed to load map")]
    Load(#[from] AssetLoadError),
    #[error("Failed to load map from DB")]
    Db(#[from] DbError),
}

impl MapDB {
    pub async fn open() -> Result<Self, OpenDbError> {
        let db = Database::open("map_db")
            .with_version(1u8)
            .with_on_blocked(|event| {
                log::warn!("DB upgrade blocked: {:?}", event);
                Ok(())
            })
            .with_on_upgrade_needed_fut(|event, db| async move {
                match (event.old_version(), event.new_version()) {
                    (0.0, Some(1.0)) => {
                        db.create_object_store("maps")
                            .with_key_path(KeyPath::One("id"))
                            .build()?;

                        db.create_object_store("assets_name")
                            .with_key_path(KeyPath::One("map_id"))
                            .build()?;
                        db.create_object_store("assets_data")
                            .with_key_path(KeyPath::One("map_id"))
                            .build()?;
                    }
                    (from, to) => {
                        log::warn!("DB upgrade from {} to {:?} not supported", from, to);
                    }
                }

                Ok(())
            })
            .await?;

        Ok(Self { db })
    }
    pub async fn load(&self) -> Result<Option<Map>, LoadError> {
        let tx = self
            .db
            .transaction(["maps", "assets_name", "assets_data"])
            .with_mode(TransactionMode::Readonly)
            .build()?;
        let map_store = tx.object_store("maps")?;
        let asset_name_store = tx.object_store("assets_name")?;
        let asset_data_store = tx.object_store("assets_data")?;

        let map_ref: Option<MapRef> = map_store.get(MAP_ID).serde()?.await?;
        let mut map = match map_ref {
            Some(map) => map.map,
            None => return Ok(None),
        };

        let assets_name: Option<AssetsNameRef> = asset_name_store.get(MAP_ID).serde()?.await?;
        let assets_name = match assets_name {
            Some(assets) => assets.assets,
            None => {
                log::warn!("No asset names found for map");
                Vec::new()
            }
        };
        let assets_data: Option<AssetsDataRef> = asset_data_store.get(MAP_ID).serde()?.await?;
        let assets_data = match assets_data {
            Some(assets) => assets.assets,
            None => {
                log::warn!("No asset data found for map");
                Vec::new()
            }
        };
        for (name, data) in assets_name.into_iter().zip(assets_data.into_iter()) {
            let data = Cursor::new(&data);
            let asset = Asset::load(&name, data)?;
            map.add_asset(asset);
        }

        tx.commit().await?;

        Ok(Some(map))
    }

    pub async fn save(&self, map: Map) -> Result<(), DbError> {
        let tx = self
            .db
            .transaction(["maps", "assets_name"])
            .with_mode(TransactionMode::Readwrite)
            .build()?;

        let assets_name_ref = AssetsNameRef {
            map_id: MAP_ID,
            assets: map
                .assets()
                .iter()
                .map(|asset| asset.name.clone())
                .collect(),
        };

        let map_ref = MapRef { id: MAP_ID, map };

        let map_store = tx.object_store("maps")?;
        let asset_name_store = tx.object_store("assets_name")?;

        map_store.put(map_ref).serde()?.await?;
        asset_name_store.put(assets_name_ref).serde()?.await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn save_with_assets(&self, map: Map) -> Result<(), DbError> {
        let tx = self
            .db
            .transaction(["assets_data"])
            .with_mode(TransactionMode::Readwrite)
            .build()?;

        let assets_data_ref = AssetsDataRef {
            map_id: MAP_ID,
            assets: map
                .assets()
                .iter()
                .map(|asset| {
                    let mut data = Vec::new();
                    asset.save(&mut data);
                    data
                })
                .collect(),
        };

        let assets_data_store = tx.object_store("assets_data")?;

        assets_data_store.put(assets_data_ref).serde()?.await?;

        tx.commit().await?;

        self.save(map).await?;

        Ok(())
    }
}

pub struct MapUpload {
    input: web_sys::HtmlInputElement,
    closure: Closure<dyn FnMut()>,
    rx: mpsc::Receiver<Result<Map, MapLoadError>>,
}

impl MapUpload {
    pub fn start() -> Result<Self, wasm_bindgen::JsValue> {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();

        let input = document
            .create_element("input")?
            .dyn_into::<web_sys::HtmlInputElement>()?;
        input.set_attribute("type", "file")?;
        input.set_attribute("accept", ".smk")?;
        input.set_attribute("style", "display: none")?;

        body.append_child(&input)?;

        let (tx, rx) = mpsc::channel();
        let closure = {
            let input = input.clone();
            Closure::once(move || {
                let file = input.files().and_then(|files| files.get(0));
                if let Some(file) = file {
                    let reader = FileReader::new().unwrap();
                    let reader_clone = reader.clone();
                    let on_load = Closure::once(Box::new(move || {
                        let array_buffer = reader_clone
                            .result()
                            .unwrap()
                            .dyn_into::<ArrayBuffer>()
                            .unwrap();
                        let buffer = Uint8Array::new(&array_buffer).to_vec();
                        tx.send(Map::load(&mut Cursor::new(&buffer))).ok();
                    }));
                    reader.set_onload(Some(on_load.as_ref().unchecked_ref()));
                    reader.read_as_array_buffer(&file).unwrap();
                    on_load.forget();
                }
            })
        };

        input.set_onchange(Some(closure.as_ref().unchecked_ref()));
        input.click();
        Ok(Self { input, closure, rx })
    }

    pub fn poll(&self) -> Option<Result<Map, MapLoadError>> {
        self.rx.try_recv().ok()
    }
}

impl Drop for MapUpload {
    fn drop(&mut self) {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();

        body.remove_child(&self.input).unwrap();
    }
}

pub fn download_map(map: &Map) -> Result<(), wasm_bindgen::JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();

    let file_name = format!("{}.smk", map.metadata.name);

    let mut data = Vec::new();
    let mut cursor = Cursor::new(&mut data);
    map.save(&mut cursor).unwrap();

    let data = Uint8Array::from(data.as_slice());
    let array = Array::new();
    array.push(&data.buffer());

    let options = BlobPropertyBag::new();
    options.set_type("application/x-tar");
    let blob = Blob::new_with_u8_array_sequence_and_options(&array, &options)?;

    let a = document.create_element("a")?;
    a.set_attribute("href", &Url::create_object_url_with_blob(&blob)?)?;
    a.set_attribute("download", &file_name)?;

    let a = a.dyn_into::<HtmlElement>()?;
    body.append_child(&a)?;
    a.click();
    body.remove_child(&a)?;

    Ok(())
}
