use image::{
    GenericImageView,
    codecs::png::{CompressionType, FilterType, PngEncoder},
};
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read, Write};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetId(usize);

impl AssetId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct Asset {
    pub name: String,
    pub image: image::DynamicImage,
}

#[derive(Debug, Clone, Default)]
pub struct MapAssets(Vec<Asset>);

#[derive(Debug, Error)]
pub enum AssetLoadError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Image error: {0}")]
    ImageError(#[from] image::ImageError),
}

impl Asset {
    pub fn dimensions(&self) -> (u32, u32) {
        self.image.dimensions()
    }

    pub fn load<R: Read>(name: &str, mut reader: R) -> Result<Self, AssetLoadError> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        let image = image::load_from_memory(&data)?;
        Ok(Self {
            name: name.to_string(),
            image,
        })
    }

    pub fn save<W: Write>(&self, mut writer: W) {
        let mut data = Vec::new();
        let mut cursor = Cursor::new(&mut data);
        self.image
            .write_with_encoder(PngEncoder::new_with_quality(
                &mut cursor,
                CompressionType::Best,
                FilterType::NoFilter,
            ))
            .unwrap();
        writer.write_all(&data).unwrap();
    }
}

impl MapAssets {
    pub fn from_loaded_assets(mut assets: Vec<(AssetId, Asset)>) -> Self {
        assets.sort_by_key(|(id, _)| id.0);

        let assets = assets.into_iter().map(|(_, asset)| asset).collect();
        Self(assets)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn insert(&mut self, asset: Asset) -> AssetId {
        let id = AssetId(self.0.len());
        self.0.push(asset);
        id
    }

    pub fn remove(&mut self, id: AssetId) -> Option<Asset> {
        if id.0 >= self.0.len() {
            return None;
        }
        Some(self.0.remove(id.0))
    }

    pub fn get(&self, id: AssetId) -> Option<&Asset> {
        self.0.get(id.0)
    }

    pub fn get_mut(&mut self, id: AssetId) -> Option<&mut Asset> {
        self.0.get_mut(id.0)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Asset> {
        self.0.iter()
    }

    pub fn iter_ids(&self) -> impl Iterator<Item = (AssetId, &Asset)> {
        self.0
            .iter()
            .enumerate()
            .map(|(i, asset)| (AssetId(i), asset))
    }
}

impl std::ops::Index<AssetId> for MapAssets {
    type Output = Asset;
    fn index(&self, id: AssetId) -> &Self::Output {
        &self.0[id.0]
    }
}
