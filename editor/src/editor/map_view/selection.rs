use common::{map::Map, types::*};

mod collider;
pub use collider::*;
mod track;
pub use track::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
    None,
    TrackPoint(TrackPoint),
    TrackSegment(TrackSegment),

    Collider(Collider),
    ColliderPoint(ColliderPoint),
    ColliderSegment(ColliderSegment),
}

pub enum GeometryType {
    None,
    Polygon,
    Segment,
    Point,
}

pub enum ObjectType {
    None,
    Track,
    Collider,
}

pub trait Select {
    fn geometry_type(&self) -> GeometryType;
    fn object_type(&self) -> ObjectType;

    // fn select<'a>(&self, map: &'a Map) -> &'a Self::Result;
    // fn select_mut<F>(&self, map: &'a mut Map, f: F)
    // where
    //     F: FnOnce(&mut Self::Result);

    fn translate(&self, map: &mut Map, delta: Vec2);

    #[allow(unused_variables)]
    fn edit_ui<'a>(&self, map: &'a mut Map, ui: &egui::Ui) {}
}

pub trait SegmentSelect: Select {
    fn segment(&self, map: &Map) -> Segment;
    fn insert_point(&self, map: &mut Map, pos: Vec2);
}

impl core::fmt::Display for Selection {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Selection::None => write!(f, "None"),

            Selection::TrackPoint(_) => write!(f, "Track Point"),
            Selection::TrackSegment(_) => write!(f, "Track Segment"),

            Selection::Collider(_) => write!(f, "Collider"),
            Selection::ColliderPoint(_) => write!(f, "Collider Point"),
            Selection::ColliderSegment(_) => write!(f, "Collider Segment"),
        }
    }
}

impl Selection {
    pub fn is_segment(&self) -> bool {
        self.as_segment().is_some()
    }

    pub fn as_segment(&self) -> Option<&dyn SegmentSelect> {
        match self {
            Selection::TrackSegment(s) => Some(s),
            Selection::ColliderSegment(s) => Some(s),
            _ => None,
        }
    }

    pub fn translate(&self, map: &mut Map, delta: Vec2) {
        match self {
            Selection::None => {}
            Selection::TrackPoint(i) => i.translate(map, delta),
            Selection::TrackSegment(i) => i.translate(map, delta),
            Selection::Collider(i) => i.translate(map, delta),
            Selection::ColliderPoint(i) => i.translate(map, delta),
            Selection::ColliderSegment(i) => i.translate(map, delta),
        }
    }

    pub fn edit_ui<'a>(&self, map: &'a mut Map, ui: &egui::Ui) {
        match self {
            Selection::None => {}
            Selection::TrackPoint(i) => i.edit_ui(map, ui),
            Selection::TrackSegment(i) => i.edit_ui(map, ui),
            Selection::Collider(i) => i.edit_ui(map, ui),
            Selection::ColliderPoint(i) => i.edit_ui(map, ui),
            Selection::ColliderSegment(i) => i.edit_ui(map, ui),
        }
    }
}
