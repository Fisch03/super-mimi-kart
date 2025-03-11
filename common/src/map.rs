use crate::types::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{Read, Seek, Write},
    path::Path,
};
use tar::{Archive, Builder};
use thiserror::Error;

mod track;
pub use track::*;

mod asset;
pub use asset::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Map {
    pub metadata: Metadata,

    #[serde(default)]
    pub background: Option<AssetId>,
    #[serde(default)]
    pub coin: Option<AssetId>,
    #[serde(default)]
    pub item_box: Option<AssetId>,

    pub track: Track,

    #[serde(default)]
    pub colliders: Vec<Collider>,
    #[serde(default)]
    pub offroad: Vec<Offroad>,
    #[serde(default)]
    pub coins: Vec<Vec2>,
    #[serde(default)]
    pub item_spawns: Vec<Vec2>,

    pub asset_paths: HashMap<String, AssetId>,
    #[serde(skip)]
    assets: MapAssets,
}

#[derive(Debug, Error)]
pub enum MapLoadError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Invalid map data: {0}")]
    DeserializeError(#[from] serde_json::Error),
    #[error("Missing data")]
    MissingData,
    #[error("Missing asset: {1}")]
    MissingAsset(AssetId, String),
}

#[derive(Debug)]
pub enum MapSaveError {
    IoError(std::io::Error),
    EncodeError(serde_json::Error),
}

impl From<serde_json::Error> for MapSaveError {
    fn from(e: serde_json::Error) -> Self {
        Self::EncodeError(e)
    }
}

impl From<std::io::Error> for MapSaveError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl Map {
    pub fn asset(&self, id: AssetId) -> Option<&Asset> {
        self.assets.get(id)
    }

    pub fn add_asset(&mut self, asset: Asset) -> AssetId {
        let name = asset.name.clone();
        let id = self.assets.insert(asset);
        self.asset_paths.insert(name, id);
        id
    }

    pub fn remove_asset(&mut self, id: AssetId) -> Option<Asset> {
        let asset = self.assets.remove(id);
        if let Some(asset) = &asset {
            self.asset_paths.remove(&asset.name);

            if let Some(bg_id) = self.background {
                if bg_id == id {
                    self.background = None;
                }
                if bg_id.as_usize() > id.as_usize() {
                    self.background = Some(AssetId::new(bg_id.as_usize() - 1));
                }
            }

            if let Some(coin_id) = self.coin {
                if coin_id == id {
                    self.coin = None;
                }
                if coin_id.as_usize() > id.as_usize() {
                    self.coin = Some(AssetId::new(coin_id.as_usize() - 1));
                }
            }
        }
        asset
    }

    pub fn assets(&self) -> &MapAssets {
        &self.assets
    }

    pub fn round_all(&mut self) {
        self.track.round_all();
        self.colliders.iter_mut().for_each(|c| c.round_all());
        self.coins.iter_mut().for_each(|p| *p = p.round());
        self.item_spawns.iter_mut().for_each(|p| *p = p.round());
    }

    pub fn asset_name_mut(&mut self, id: AssetId, f: impl FnOnce(&mut String)) {
        let asset = match self.assets.get_mut(id) {
            Some(asset) => asset,
            None => return,
        };

        let current_name = asset.name.clone();
        f(&mut asset.name);
        if current_name != asset.name {
            self.asset_paths.remove(&current_name);
            self.asset_paths.insert(asset.name.clone(), id);
        }
    }

    pub fn load<R: Read + Seek>(mut map: R) -> Result<Self, MapLoadError> {
        map.seek(std::io::SeekFrom::Start(0))?;
        let mut map = Archive::new(map);

        let mut data = None;

        let mut all_assets = HashMap::new();
        for entry in map.entries_with_seek()? {
            let entry = entry?;

            if &entry.path()? == Path::new("data") {
                data = Some(serde_json::from_reader(entry)?);
            } else if let Some(name) = entry.path()?.file_name() {
                let name = name.to_string_lossy().to_string();
                if let Ok(asset) = Asset::load(&name, entry) {
                    all_assets.insert(name, asset);
                }
            }
        }

        let mut data: Map = data.ok_or(MapLoadError::MissingData)?;
        let mut assets = Vec::with_capacity(data.asset_paths.len());

        for (name, id) in data.asset_paths.iter() {
            if let Some(asset) = all_assets.remove(name) {
                assets.push((*id, asset));
            } else {
                return Err(MapLoadError::MissingAsset(*id, name.clone()));
            }
        }

        data.assets = MapAssets::from_loaded_assets(assets);
        Ok(data)
    }

    pub fn save<W: Write + Seek>(&self, map: W) -> Result<(), MapSaveError> {
        let mut map = Builder::new(map);

        let mut header = tar::Header::new_gnu();

        {
            let mut data = map.append_writer(&mut header, "data")?;
            serde_json::to_writer(&mut data, self)?;
        }

        for asset in self.assets.iter() {
            let mut header = tar::Header::new_gnu();
            let mut data = map.append_writer(&mut header, &asset.name)?;
            asset.save(&mut data);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub name: String,
    pub description: String,
    pub author: String,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            name: "Untitled Map".to_string(),
            description: "".to_string(),
            author: "".to_string(),
        }
    }
}

pub type Offroad = Collider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collider {
    pub shape: Vec<Vec2>,
}

impl Collider {
    pub fn translate(&mut self, delta: Vec2) {
        self.shape.iter_mut().for_each(|p| *p += delta);
    }

    pub fn segment(&self, i: usize) -> Segment {
        Segment {
            start: self.shape[i],
            end: self.shape[(i + 1) % self.shape.len()],
        }
    }

    pub fn set_segment(&mut self, i: usize, segment: Segment) {
        let next_index = (i + 1) % self.shape.len();
        self.shape[i] = segment.start;
        self.shape[next_index] = segment.end;
    }

    pub fn round_all(&mut self) {
        self.shape.iter_mut().for_each(|p| *p = p.round());
    }
}

impl std::ops::Deref for Collider {
    type Target = Vec<Vec2>;
    fn deref(&self) -> &Self::Target {
        &self.shape
    }
}

impl std::ops::DerefMut for Collider {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.shape
    }
}

impl Default for Collider {
    fn default() -> Self {
        Self {
            shape: vec![
                Vec2::new(-100.0, -100.0),
                Vec2::new(100.0, -100.0),
                Vec2::new(100.0, 100.0),
                Vec2::new(-100.0, 100.0),
            ],
        }
    }
}
