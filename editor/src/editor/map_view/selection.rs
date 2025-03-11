use common::{map::Map, types::*};
use egui::Grid;

mod collider;
pub use collider::*;
mod offroad;
pub use offroad::*;
mod track;
pub use track::*;
mod coin;
use coin::*;
mod item_box;
use item_box::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
    None,

    Coin(Coin),

    ItemBox(ItemBox),

    TrackPoint(TrackPoint),
    TrackSegment(TrackSegment),

    Collider(Collider),
    ColliderPoint(ColliderPoint),
    ColliderSegment(ColliderSegment),

    Offroad(Offroad),
    OffroadPoint(OffroadPoint),
    OffroadSegment(OffroadSegment),
}

fn edit_point(ui: &mut egui::Ui, point: &mut Vec2) {
    ui.label("X");
    ui.add(egui::DragValue::new(&mut point.x).fixed_decimals(0));
    ui.end_row();

    ui.label("Y");
    ui.add(egui::DragValue::new(&mut point.y).fixed_decimals(0));
    ui.end_row();
}

pub trait Select {
    // fn select<'a>(&self, map: &'a Map) -> &'a Self::Result;
    // fn select_mut<F>(&self, map: &'a mut Map, f: F)
    // where
    //     F: FnOnce(&mut Self::Result);

    fn translate(&self, map: &mut Map, delta: Vec2);

    #[allow(unused_variables)]
    fn edit_ui<'a>(&self, map: &'a mut Map, ui: &mut egui::Ui) {}
}

pub trait PointSelect: Select {
    fn point(&self, map: &Map) -> Vec2;
    fn set_point(&self, map: &mut Map, point: Vec2);
}

pub trait SegmentSelect: Select {
    fn segment(&self, map: &Map) -> Segment;
    fn set_segment(&self, map: &mut Map, segment: Segment);
    fn insert_point(&self, map: &mut Map, pos: Vec2);
}

impl std::fmt::Display for Selection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Selection::None => write!(f, "None"),

            Selection::Coin(_) => write!(f, "Coin"),

            Selection::ItemBox(_) => write!(f, "Item Box"),

            Selection::TrackPoint(_) => write!(f, "Track Point"),
            Selection::TrackSegment(_) => write!(f, "Track Segment"),

            Selection::Collider(_) => write!(f, "Collider"),
            Selection::ColliderPoint(_) => write!(f, "Collider Point"),
            Selection::ColliderSegment(_) => write!(f, "Collider Segment"),

            Selection::Offroad(_) => write!(f, "Offroad"),
            Selection::OffroadPoint(_) => write!(f, "Offroad Point"),
            Selection::OffroadSegment(_) => write!(f, "Offroad Segment"),
        }
    }
}

impl Selection {
    pub fn is_segment(&self) -> bool {
        self.as_segment().is_some()
    }

    pub fn as_point(&self) -> Option<&dyn PointSelect> {
        match self {
            Selection::TrackPoint(p) => Some(p),
            Selection::ColliderPoint(p) => Some(p),
            Selection::OffroadPoint(p) => Some(p),
            _ => None,
        }
    }

    pub fn as_segment(&self) -> Option<&dyn SegmentSelect> {
        match self {
            Selection::TrackSegment(s) => Some(s),
            Selection::ColliderSegment(s) => Some(s),
            Selection::OffroadSegment(s) => Some(s),
            _ => None,
        }
    }

    pub fn translate(&self, map: &mut Map, delta: Vec2) {
        match self {
            Selection::None => {}

            Self::Coin(i) => i.translate(map, delta),

            Selection::ItemBox(i) => i.translate(map, delta),

            Selection::TrackPoint(i) => i.translate(map, delta),
            Selection::TrackSegment(i) => i.translate(map, delta),

            Selection::Collider(i) => i.translate(map, delta),
            Selection::ColliderPoint(i) => i.translate(map, delta),
            Selection::ColliderSegment(i) => i.translate(map, delta),

            Selection::Offroad(i) => i.translate(map, delta),
            Selection::OffroadPoint(i) => i.translate(map, delta),
            Selection::OffroadSegment(i) => i.translate(map, delta),
        }
    }

    pub fn edit_ui<'a>(&self, map: &'a mut Map, ui: &mut egui::Ui) {
        Grid::new("edit_ui")
            .num_columns(2)
            .show(ui, |ui| match self {
                Selection::None => {}

                Selection::Coin(i) => i.edit_ui(map, ui),

                Selection::ItemBox(i) => i.edit_ui(map, ui),

                Selection::TrackPoint(i) => i.edit_ui(map, ui),
                Selection::TrackSegment(i) => i.edit_ui(map, ui),

                Selection::Collider(i) => i.edit_ui(map, ui),
                Selection::ColliderPoint(i) => i.edit_ui(map, ui),
                Selection::ColliderSegment(i) => i.edit_ui(map, ui),

                Selection::Offroad(i) => i.edit_ui(map, ui),
                Selection::OffroadPoint(i) => i.edit_ui(map, ui),
                Selection::OffroadSegment(i) => i.edit_ui(map, ui),
            });
    }
}
