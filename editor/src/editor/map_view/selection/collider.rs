use super::{edit_point, GeometryType, ObjectType, SegmentSelect, Select, Selection};
use common::{map::Map, types::*};

fn next_index(collider: Collider, index: usize, map: &Map) -> usize {
    (index + 1) % map.colliders[collider.0].len()
}

fn prev_index(collider: Collider, index: usize, map: &Map) -> usize {
    (index + map.colliders[collider.0].len() - 1) % map.colliders[collider.0].len()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Collider(pub usize);
impl Selection {
    pub fn collider(c_i: usize) -> Self {
        Selection::Collider(Collider(c_i))
    }
}
impl Select for Collider {
    fn geometry_type(&self) -> GeometryType {
        GeometryType::Polygon
    }
    fn object_type(&self) -> ObjectType {
        ObjectType::Collider
    }

    fn translate(&self, map: &mut Map, delta: Vec2) {
        map.colliders[self.0].translate(delta);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColliderPoint {
    pub collider: Collider,
    pub p_i: usize,
}
impl Selection {
    pub fn collider_point(c_i: usize, p_i: usize) -> Self {
        Selection::ColliderPoint(ColliderPoint {
            collider: Collider(c_i),
            p_i,
        })
    }
}
impl ColliderPoint {
    pub fn next(&self, map: &Map) -> ColliderPoint {
        ColliderPoint {
            collider: self.collider,
            p_i: next_index(self.collider, self.p_i, map),
        }
    }

    pub fn prev(&self, map: &Map) -> ColliderPoint {
        ColliderPoint {
            collider: self.collider,
            p_i: prev_index(self.collider, self.p_i, map),
        }
    }

    fn get_point<'a>(&self, map: &'a mut Map) -> &'a mut Vec2 {
        &mut map.colliders[self.collider.0][self.p_i]
    }
}
impl Select for ColliderPoint {
    fn geometry_type(&self) -> GeometryType {
        GeometryType::Point
    }
    fn object_type(&self) -> ObjectType {
        ObjectType::Collider
    }

    fn translate(&self, map: &mut Map, delta: Vec2) {
        *self.get_point(map) += delta;
    }

    fn edit_ui<'a>(&self, map: &'a mut Map, ui: &mut egui::Ui) {
        edit_point(ui, self.get_point(map));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColliderSegment {
    pub collider: Collider,
    pub s_i: usize,
}
impl Selection {
    pub fn collider_segment(c_i: usize, s_i: usize) -> Self {
        Selection::ColliderSegment(ColliderSegment {
            collider: Collider(c_i),
            s_i,
        })
    }
}
impl ColliderSegment {
    pub fn next(&self, map: &Map) -> ColliderSegment {
        ColliderSegment {
            collider: self.collider,
            s_i: next_index(self.collider, self.s_i, map),
        }
    }
    pub fn prev(&self, map: &Map) -> ColliderSegment {
        ColliderSegment {
            collider: self.collider,
            s_i: prev_index(self.collider, self.s_i, map),
        }
    }
}
impl SegmentSelect for ColliderSegment {
    fn segment(&self, map: &Map) -> Segment {
        map.colliders[self.collider.0].segment(self.s_i)
    }
    fn set_segment(&self, map: &mut Map, segment: Segment) {
        map.colliders[self.collider.0].set_segment(self.s_i, segment);
    }
    fn insert_point(&self, map: &mut Map, pos: Vec2) {
        map.colliders[self.collider.0].insert(self.s_i + 1, pos);
        // ColliderPoint {
        //     collider: self.collider,
        //     p_i: self.s_i + 1,
        // }
    }
}
impl Select for ColliderSegment {
    fn geometry_type(&self) -> GeometryType {
        GeometryType::Segment
    }
    fn object_type(&self) -> ObjectType {
        ObjectType::Collider
    }

    fn translate(&self, map: &mut Map, delta: Vec2) {
        map.colliders[self.collider.0][self.s_i] += delta;
        let next_index = next_index(self.collider, self.s_i, map);
        map.colliders[self.collider.0][next_index] += delta;
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
