use crate::types::*;
use serde::{Deserialize, Serialize};
use std::iter::Iterator;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub path: Vec<TrackPoint>,

    pub start_offset_h: f32,
    pub start_offset_v: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackPoint {
    pub pos: Vec2,
    pub checkpoint_rotation: f32,
    pub checkpoint_width: f32,
}

pub struct TrackStartIter<'a> {
    track: &'a Track,
    index: usize,
    offset: f32,
    side: bool,

    step_size: f32,
    center_distance: f32,
}

impl Track {
    pub fn iter_starts(&self) -> impl Iterator<Item = Vec2> + '_ {
        TrackStartIter::new(self)
    }

    pub fn iter_segments(&self) -> impl Iterator<Item = Segment> + '_ {
        (0..self.path.len()).map(move |i| self.segment(i))
    }

    pub fn segment(&self, index: usize) -> Segment {
        Segment {
            start: self.path[index].pos,
            end: self.path[(index + 1) % self.path.len()].pos,
        }
    }

    pub fn set_segment(&mut self, index: usize, segment: Segment) {
        let next_index = (index + 1) % self.path.len();
        self.path[index].pos = segment.start;
        self.path[next_index].pos = segment.end;
    }
}

impl core::ops::Index<usize> for Track {
    type Output = TrackPoint;
    fn index(&self, index: usize) -> &Self::Output {
        &self.path[index]
    }
}

impl core::ops::IndexMut<usize> for Track {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.path[index]
    }
}

impl Default for Track {
    fn default() -> Self {
        Self {
            path: vec![
                TrackPoint::new((-100.0, -100.0)),
                TrackPoint::new((100.0, -100.0)),
                TrackPoint::new((100.0, 100.0)),
                TrackPoint::new((-100.0, 100.0)),
            ],
            start_offset_h: 20.0,
            start_offset_v: 15.0,
        }
    }
}

impl TrackPoint {
    pub fn new<P: Into<Vec2>>(pos: P) -> Self {
        let pos = pos.into();

        Self {
            pos,
            checkpoint_rotation: 0.0,
            checkpoint_width: 0.0,
        }
    }
}

impl<'a> TrackStartIter<'a> {
    pub fn new(track: &'a Track) -> Self {
        Self {
            track,
            index: track.path.len() - 1,
            offset: 0.0,
            side: false,
            step_size: track.start_offset_h,
            center_distance: track.start_offset_v,
        }
    }

    fn current_segment(&self) -> Segment {
        self.track.segment(self.index)
    }

    fn prev_segment(&mut self) {
        self.index = (self.index + self.track.path.len() - 1) % self.track.path.len();
    }
}

impl<'a> Iterator for TrackStartIter<'a> {
    type Item = Vec2;
    fn next(&mut self) -> Option<Self::Item> {
        self.offset += self.step_size;
        loop {
            let segment = self.current_segment();
            let segment_length = segment.length();
            if self.offset > segment_length {
                self.offset -= segment_length;
                self.prev_segment();
            } else {
                let ratio = 1.0 - (self.offset / segment_length);
                let p = segment.interpolate(ratio);

                // move the point to the side
                let side = if self.side { 1.0 } else { -1.0 };
                self.side = !self.side;
                let normal = Vec2::new(-segment.dy(), segment.dx()).normalize();

                // return Some(Vec2::new(
                //     p.x + normal.x * side * self.center_distance,
                //     p.y + normal.y * side * self.center_distance,
                // ));
                return Some(p + normal * side * self.center_distance);
            }
        }
    }
}
