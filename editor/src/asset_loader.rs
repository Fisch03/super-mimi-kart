use common::{map::*, types::GenericImageView};
use egui::{
    load::{ImageLoadResult, ImageLoader, ImagePoll, LoadError},
    ColorImage, Context, SizeHint,
};
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub struct AssetLoader {
    assets: Mutex<MapAssets>,
    cache: Mutex<Vec<Option<Arc<ColorImage>>>>,
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
            todo!("Load default asset");
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

        let (width, height) = asset.0.dimensions();
        let asset_raw = asset.0.to_rgba8();
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
