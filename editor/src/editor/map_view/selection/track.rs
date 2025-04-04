use super::{edit_point, PointSelect, SegmentSelect, Select, Selection};
use common::{
    map::{self, Map},
    types::*,
};

fn next_index(index: usize, map: &Map) -> usize {
    (index + 1) % map.track.path.len()
}
fn prev_index(index: usize, map: &Map) -> usize {
    (index + map.track.path.len() - 1) % map.track.path.len()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrackSegment(pub usize);
impl Selection {
    pub fn track_segment(s_i: usize) -> Self {
        Selection::TrackSegment(TrackSegment(s_i))
    }
}
impl TrackSegment {
    pub fn next(&self, map: &Map) -> TrackSegment {
        TrackSegment(next_index(self.0, map))
    }
    pub fn prev(&self, map: &Map) -> TrackSegment {
        TrackSegment(prev_index(self.0, map))
    }
}
impl SegmentSelect for TrackSegment {
    fn segment(&self, map: &Map) -> Segment {
        map.track.segment(self.0)
    }

    fn set_segment(&self, map: &mut Map, segment: Segment) {
        map.track.set_segment(self.0, segment);
    }

    fn insert_point(&self, map: &mut Map, pos: Vec2) {
        map.track.path.insert(self.0 + 1, map::TrackPoint::new(pos));
        // TrackPoint(self.0 + 1)
    }
}
impl Select for TrackSegment {
    fn translate(&self, map: &mut Map, delta: Vec2) {
        map.track.path[self.0].pos += delta;
        let next_index = next_index(self.0, map);
        map.track.path[next_index].pos += delta;
    }

    fn edit_ui<'a>(&self, map: &'a mut Map, ui: &mut egui::Ui) {
        let mut segment = self.segment(map);

        ui.strong("Start");
        ui.end_row();
        edit_point(ui, &mut segment.start);

        ui.strong("End");
        ui.end_row();
        edit_point(ui, &mut segment.end);

        self.set_segment(map, segment);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrackPoint(pub usize);
impl Selection {
    pub fn track_point(p_i: usize) -> Self {
        Selection::TrackPoint(TrackPoint(p_i))
    }
}
impl TrackPoint {
    pub fn next(&self, map: &Map) -> TrackPoint {
        TrackPoint(next_index(self.0, map))
    }
    pub fn prev(&self, map: &Map) -> TrackPoint {
        TrackPoint(prev_index(self.0, map))
    }
}
impl PointSelect for TrackPoint {
    fn point(&self, map: &Map) -> Vec2 {
        map.track[self.0].pos
    }
    fn set_point(&self, map: &mut Map, point: Vec2) {
        map.track[self.0].pos = point;
    }
}
impl Select for TrackPoint {
    fn translate(&self, map: &mut Map, delta: Vec2) {
        map.track[self.0].pos += delta;
    }

    fn edit_ui<'a>(&self, map: &'a mut Map, ui: &mut egui::Ui) {
        let point = &mut map.track[self.0];

        ui.strong("Position");
        ui.end_row();
        edit_point(ui, &mut point.pos);

        ui.strong("Checkpoint");
        ui.end_row();

        ui.label("Rotation");
        ui.add(egui::DragValue::new(&mut point.checkpoint_rotation));
        point.checkpoint_rotation = point.checkpoint_rotation.rem_euclid(360.0);
        ui.end_row();

        ui.label("Width Left");
        ui.add(egui::DragValue::new(&mut point.checkpoint_width_left).range(0.0..=f32::INFINITY));
        ui.end_row();

        ui.label("Width Right");
        ui.add(egui::DragValue::new(&mut point.checkpoint_width_right).range(0.0..=f32::INFINITY));
        ui.end_row();
    }
}
