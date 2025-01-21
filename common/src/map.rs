use crate::types::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{Read, Seek, Write},
    path::Path,
};
use tar::{Archive, Builder};

mod track;
pub use track::*;

mod asset;
pub use asset::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Map {
    pub metadata: Metadata,
    pub background: Option<AssetId>,
    pub track: Track,
    pub colliders: Vec<Collider>,
    pub asset_paths: HashMap<String, AssetId>,
    #[serde(skip)]
    assets: MapAssets,
}

#[derive(Debug)]
pub enum MapLoadError {
    IoError(std::io::Error),
    DeserializeError(serde_json::Error),
    MissingData,
    MissingAsset((AssetId, String)),
}

impl From<serde_json::Error> for MapLoadError {
    fn from(e: serde_json::Error) -> Self {
        Self::DeserializeError(e)
    }
}

impl From<std::io::Error> for MapLoadError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
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
    pub fn assets(&self) -> &MapAssets {
        &self.assets
    }

    pub fn round_all(&mut self) {
        self.track.round_all();
        self.colliders.iter_mut().for_each(|c| c.round_all());
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
                if let Ok(asset) = Asset::load(entry) {
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
                return Err(MapLoadError::MissingAsset((*id, name.clone())));
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

        for (id, asset) in self.assets.iter_ids() {
            let mut header = tar::Header::new_gnu();
            let mut data = map.append_writer(&mut header, &format!("asset/{}", id.as_usize()))?;
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

impl core::ops::Deref for Collider {
    type Target = Vec<Vec2>;
    fn deref(&self) -> &Self::Target {
        &self.shape
    }
}

impl core::ops::DerefMut for Collider {
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
