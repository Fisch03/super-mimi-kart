use crate::types::*;
use geo::{prelude::*, Closest, Line};
use serde::{Deserialize, Serialize};
use std::collections::{vec_deque, VecDeque};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Map {
    pub metadata: Metadata,
    pub track: Track,
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
pub struct Track {
    pub path: VecDeque<Segment>,

    pub start_offset_h: f32,
    pub start_offset_v: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub line: Line<f32>,
    pub checkpoint_rotation: f32,
    pub checkpoint_width: f32,
}

use std::iter::{Cycle, Iterator, Peekable, Rev};
pub struct TrackStartIter<'a> {
    segments: Peekable<Cycle<Rev<vec_deque::Iter<'a, Segment>>>>,
    offset: f32,
    side: bool,

    step_size: f32,
    center_distance: f32,
}

impl Track {
    pub fn iter_starts(&self) -> impl Iterator<Item = Vec2> + '_ {
        TrackStartIter::new(self)
    }
}

impl<'a> TrackStartIter<'a> {
    pub fn new(track: &'a Track) -> Self {
        let segments = track.path.iter().rev().cycle().peekable();

        Self {
            segments,
            offset: 0.0,
            side: false,
            step_size: track.start_offset_h,
            center_distance: track.start_offset_v,
        }
    }
}

impl<'a> Iterator for TrackStartIter<'a> {
    type Item = Vec2;
    fn next(&mut self) -> Option<Self::Item> {
        self.offset += self.step_size;
        while let Some(segment) = self.segments.peek() {
            if self.offset > segment.line.length::<Euclidean>() {
                self.offset -= segment.line.length::<Euclidean>();
                self.segments.next();
            } else {
                let ratio = 1.0 - (self.offset / segment.line.length::<Euclidean>());
                let p = segment.line.line_interpolate_point(ratio).unwrap();

                // move the point to the side
                let side = if self.side { 1.0 } else { -1.0 };
                self.side = !self.side;
                let normal = Vec2::new(-segment.line.dy(), segment.line.dx()).normalize();

                return Some(Vec2::new(
                    p.x() + normal.x * side * self.center_distance,
                    p.y() + normal.y * side * self.center_distance,
                ));
            }
        }

        None
    }
}

impl Segment {
    pub fn new(start: Vec2, end: Vec2) -> Self {
        Self {
            line: Line::new((start.x, start.y), (end.x, end.y)),
            checkpoint_rotation: 0.0,
            checkpoint_width: 0.0,
        }
    }

    pub fn distance(&self, point: Vec2) -> f32 {
        Euclidean::distance(
            &self.line,
            geo::Coord {
                x: point.x,
                y: point.y,
            },
        )
    }

    pub fn closest_point(&self, point: Vec2) -> Vec2 {
        let closest = self.line.closest_point(&geo::Point::new(point.x, point.y));
        match closest {
            Closest::Intersection(point) => Vec2::new(point.x(), point.y()),
            Closest::SinglePoint(point) => Vec2::new(point.x(), point.y()),
            Closest::Indeterminate => panic!("indeterminate closest point"),
        }
    }
}

impl Default for Track {
    fn default() -> Self {
        Self {
            path: VecDeque::from(vec![
                Segment::new(Vec2::new(-100.0, -100.0), Vec2::new(100.0, -100.0)),
                Segment::new(Vec2::new(100.0, -100.0), Vec2::new(100.0, 100.0)),
                Segment::new(Vec2::new(100.0, 100.0), Vec2::new(-100.0, 100.0)),
                Segment::new(Vec2::new(-100.0, 100.0), Vec2::new(-100.0, -100.0)),
            ]),
            start_offset_h: 20.0,
            start_offset_v: 15.0,
        }
    }
}
