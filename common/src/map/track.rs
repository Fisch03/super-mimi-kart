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
    pub checkpoint_width_left: f32,
    pub checkpoint_width_right: f32,
}

pub struct TrackStartIter<'a> {
    track: &'a Track,
    index: usize,
    offset: f32,
    side: bool,

    step_size: f32,
    center_distance: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct TrackPosition {
    pub lap: usize,
    pub segment: usize,
    pub progress: f32,
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

    pub fn round_all(&mut self) {
        for point in &mut self.path {
            *point = point.to_rounded();
        }
    }

    pub fn calc_position(&self, old_pos: Vec2, new_pos: Vec2, track_pos: &mut TrackPosition) {
        fn intersect(a: &Segment, b: &Segment) -> bool {
            let det = (a.end.x - a.start.x) * (b.end.y - b.start.y)
                - (b.end.x - b.start.x) * (a.end.y - a.start.y);

            if det == 0.0 {
                return false;
            }

            let lambda = ((b.end.y - b.start.y) * (b.end.x - a.start.x)
                + (b.start.x - b.end.x) * (b.end.y - a.start.y))
                / det;
            let gamma = ((a.start.y - a.end.y) * (b.end.x - a.start.x)
                + (a.end.x - a.start.x) * (b.end.y - a.start.y))
                / det;

            return (0.0 < lambda && lambda < 1.0) && (0.0 < gamma && gamma < 1.0);
        }

        let move_segment = Segment::new(old_pos, new_pos);

        // advance checkpoints until we find one that doesn't intersect
        for _ in 0..self.path.len() {
            let (left, right) = self.path[track_pos.segment].checkpoint_positions();
            let checkpoint_segment = Segment::new(left, right);

            if !intersect(&move_segment, &checkpoint_segment) {
                break;
            }

            track_pos.segment = (track_pos.segment + 1) % self.path.len();
            if track_pos.segment == 1 {
                track_pos.lap += 1;
            }
        }

        let track_segment =
            self.segment((track_pos.segment + self.path.len() - 1) % self.path.len());

        // project the new position onto the track segment
        let ab = track_segment.end - track_segment.start;
        let ac = new_pos - track_segment.start;
        let ad = ab * (ab.dot(ac) / ab.dot(ab));
        let projected = track_segment.start + ad;

        track_pos.progress = track_segment.start.distance(projected) / track_segment.length();
    }
}

impl std::ops::Index<usize> for Track {
    type Output = TrackPoint;
    fn index(&self, index: usize) -> &Self::Output {
        &self.path[index]
    }
}

impl std::ops::IndexMut<usize> for Track {
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
            checkpoint_width_left: 10.0,
            checkpoint_width_right: 10.0,
        }
    }

    pub fn checkpoint_positions(&self) -> (Vec2, Vec2) {
        let dir = Vec2::new(
            self.checkpoint_rotation.to_radians().cos(),
            self.checkpoint_rotation.to_radians().sin(),
        );
        let left = self.pos - dir * self.checkpoint_width_left;
        let right = self.pos + dir * self.checkpoint_width_right;
        (left, right)
    }

    pub fn to_rounded(&self) -> Self {
        Self {
            pos: self.pos.round(),
            checkpoint_rotation: self.checkpoint_rotation,
            checkpoint_width_left: self.checkpoint_width_left,
            checkpoint_width_right: self.checkpoint_width_right,
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
            let mut segment = self.current_segment();
            segment.round();
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

impl std::cmp::Eq for TrackPosition {}

impl std::cmp::Ord for TrackPosition {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.lap
            .cmp(&other.lap)
            .then_with(|| self.segment.cmp(&other.segment))
            .then_with(|| {
                self.progress
                    .partial_cmp(&other.progress)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }
}
