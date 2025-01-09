use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read, Seek, Write};

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
pub struct Asset(image::DynamicImage);

#[derive(Debug, Clone, Default)]
pub struct MapAssets(Vec<Asset>);

impl Asset {
    pub fn load<R: Read>(mut reader: R) -> Self {
        let mut data = Vec::new();
        reader.read_to_end(&mut data).unwrap();
        let image = image::load_from_memory(&data).unwrap();
        Self(image)
    }

    pub fn save<W: Write>(&self, mut writer: W) {
        let mut data = Vec::new();
        let mut cursor = Cursor::new(&mut data);
        self.0
            .write_to(&mut cursor, image::ImageFormat::Png)
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

    pub fn insert(&mut self, asset: Asset) -> AssetId {
        let id = AssetId(self.0.len());
        self.0.push(asset);
        id
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
