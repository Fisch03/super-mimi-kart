use crate::types::*;
use serde::{Deserialize, Serialize};

mod track;
pub use track::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Map {
    pub metadata: Metadata,
    pub track: Track,
    pub colliders: Vec<Collider>,
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
