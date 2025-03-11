use super::{edit_point, PointSelect, SegmentSelect, Select, Selection};
use common::{map::Map, types::*};

fn next_index(offroad: Offroad, index: usize, map: &Map) -> usize {
    (index + 1) % map.offroad[offroad.0].len()
}

fn prev_index(offroad: Offroad, index: usize, map: &Map) -> usize {
    (index + map.offroad[offroad.0].len() - 1) % map.offroad[offroad.0].len()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Offroad(pub usize);
impl Selection {
    pub fn offroad(c_i: usize) -> Self {
        Selection::Offroad(Offroad(c_i))
    }
}
impl Select for Offroad {
    fn translate(&self, map: &mut Map, delta: Vec2) {
        map.offroad[self.0].translate(delta);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OffroadPoint {
    pub offroad: Offroad,
    pub p_i: usize,
}
impl Selection {
    pub fn offroad_point(c_i: usize, p_i: usize) -> Self {
        Selection::OffroadPoint(OffroadPoint {
            offroad: Offroad(c_i),
            p_i,
        })
    }
}
impl PointSelect for OffroadPoint {
    fn point(&self, map: &Map) -> Vec2 {
        map.offroad[self.offroad.0][self.p_i]
    }
    fn set_point(&self, map: &mut Map, point: Vec2) {
        map.offroad[self.offroad.0][self.p_i] = point;
    }
}
impl OffroadPoint {
    pub fn next(&self, map: &Map) -> OffroadPoint {
        OffroadPoint {
            offroad: self.offroad,
            p_i: next_index(self.offroad, self.p_i, map),
        }
    }

    pub fn prev(&self, map: &Map) -> OffroadPoint {
        OffroadPoint {
            offroad: self.offroad,
            p_i: prev_index(self.offroad, self.p_i, map),
        }
    }

    fn get_point<'a>(&self, map: &'a mut Map) -> &'a mut Vec2 {
        &mut map.offroad[self.offroad.0][self.p_i]
    }
}
impl Select for OffroadPoint {
    fn translate(&self, map: &mut Map, delta: Vec2) {
        *self.get_point(map) += delta;
    }

    fn edit_ui<'a>(&self, map: &'a mut Map, ui: &mut egui::Ui) {
        edit_point(ui, self.get_point(map));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OffroadSegment {
    pub offroad: Offroad,
    pub s_i: usize,
}
impl Selection {
    pub fn offroad_segment(c_i: usize, s_i: usize) -> Self {
        Selection::OffroadSegment(OffroadSegment {
            offroad: Offroad(c_i),
            s_i,
        })
    }
}
impl OffroadSegment {
    pub fn next(&self, map: &Map) -> OffroadSegment {
        OffroadSegment {
            offroad: self.offroad,
            s_i: next_index(self.offroad, self.s_i, map),
        }
    }
    pub fn prev(&self, map: &Map) -> OffroadSegment {
        OffroadSegment {
            offroad: self.offroad,
            s_i: prev_index(self.offroad, self.s_i, map),
        }
    }
}
impl SegmentSelect for OffroadSegment {
    fn segment(&self, map: &Map) -> Segment {
        map.offroad[self.offroad.0].segment(self.s_i)
    }
    fn set_segment(&self, map: &mut Map, segment: Segment) {
        map.offroad[self.offroad.0].set_segment(self.s_i, segment);
    }
    fn insert_point(&self, map: &mut Map, pos: Vec2) {
        map.offroad[self.offroad.0].insert(self.s_i + 1, pos);
    }
}
impl Select for OffroadSegment {
    fn translate(&self, map: &mut Map, delta: Vec2) {
        map.offroad[self.offroad.0][self.s_i] += delta;
        let next_index = next_index(self.offroad, self.s_i, map);
        map.offroad[self.offroad.0][next_index] += delta;
    }

    fn edit_ui<'a>(&self, map: &'a mut Map, ui: &mut egui::Ui) {
        let mut segment = self.segment(map);

        ui.heading("Start");
        ui.end_row();
        edit_point(ui, &mut segment.start);

        ui.strong("End");
        ui.end_row();
        edit_point(ui, &mut segment.end);

        self.set_segment(map, segment);
    }
}
