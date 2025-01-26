use common::map::*;
use egui::{
    load::{ImageLoadResult, ImageLoader, ImagePoll, LoadError},
    ColorImage, Context, SizeHint,
};
use js_sys::{ArrayBuffer, Uint8Array};
use std::{
    io::Cursor,
    sync::{mpsc, Arc, Mutex},
};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{Blob, BlobPropertyBag, FileReader, HtmlElement, Url};

#[derive(Debug, Default)]
pub struct AssetLoader {
    assets: Mutex<MapAssets>,
    cache: Mutex<Vec<Option<Arc<ColorImage>>>>,
    default_img: Mutex<Option<Arc<ColorImage>>>,
}

impl AssetLoader {
    pub fn new(map: &Map) -> Self {
        let new = Self::default();
        new.load_map(map);
        new
    }

    pub fn load_map(&self, map: &Map) {
        let mut assets = self.assets.lock().unwrap();
        *assets = map.assets().clone();
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
        cache.resize(assets.len(), None);
    }
}

impl ImageLoader for AssetLoader {
    fn id(&self) -> &str {
        concat!(module_path!(), "::AssetLoader")
    }

    fn load(&self, _: &Context, uri: &str, _: SizeHint) -> ImageLoadResult {
        if !uri.starts_with("smk://asset/") {
            return Err(LoadError::NotSupported);
        }

        let asset_id = uri.split('/').last().unwrap();
        if asset_id == "default" {
            let mut default_img = self.default_img.lock().unwrap();
            let default_img = default_img.get_or_insert_with(|| {
                let data = (0..4 * 1000 * 1000).map(|_| 100).collect::<Vec<_>>();
                let image = ColorImage::from_rgba_unmultiplied([1000, 1000], &data);
                Arc::new(image)
            });

            return Ok(ImagePoll::Ready {
                image: default_img.clone(),
            });
        }

        let asset_id = asset_id.parse();
        let asset_id = match asset_id {
            Ok(asset_id) => AssetId::new(asset_id),
            Err(_) => return Err(LoadError::Loading("Invalid asset ID".to_string())),
        };

        let mut cache = self.cache.lock().unwrap();

        let cached = match cache.get(asset_id.as_usize()) {
            Some(cached) => cached.clone(),
            None => None,
        };
        if let Some(cached) = cached {
            return Ok(ImagePoll::Ready { image: cached });
        }

        let assets = self.assets.lock().unwrap();
        let asset = match assets.get(asset_id) {
            Some(asset) => asset.clone(),
            None => return Err(LoadError::Loading("Asset not found".to_string())),
        };

        let (width, height) = asset.dimensions();
        let asset_raw = asset.image.to_rgba8();
        let image = ColorImage::from_rgba_unmultiplied(
            [width as usize, height as usize],
            asset_raw.as_raw(),
        );
        let image = Arc::new(image);
        cache[asset_id.as_usize()] = Some(image.clone());
        Ok(ImagePoll::Ready { image })
    }

    fn forget(&self, _uri: &str) {}

    fn forget_all(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
        cache.resize(self.assets.lock().unwrap().len(), None);
    }

    fn byte_size(&self) -> usize {
        let cache = self.cache.lock().unwrap();
        cache.iter().fold(0, |size, image| {
            size + image.as_ref().map_or(0, |image| image.pixels.len() * 4)
        })
    }
}

pub struct AssetUpload {
    input: web_sys::HtmlInputElement,
    closure: Closure<dyn FnMut()>,
    rx: mpsc::Receiver<Result<Asset, AssetLoadError>>,
}

impl AssetUpload {
    pub fn start(name: String) -> Result<Self, wasm_bindgen::JsValue> {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();

        let input = document
            .create_element("input")?
            .dyn_into::<web_sys::HtmlInputElement>()?;
        input.set_attribute("type", "file")?;
        input.set_attribute("accept", "image/*")?;
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
                        tx.send(Asset::load(&name, &mut Cursor::new(&buffer))).ok();
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

    pub fn poll(&self) -> Option<Result<Asset, AssetLoadError>> {
        self.rx.try_recv().ok()
    }
}

impl Drop for AssetUpload {
    fn drop(&mut self) {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();

        body.remove_child(&self.input).unwrap();
    }
}
