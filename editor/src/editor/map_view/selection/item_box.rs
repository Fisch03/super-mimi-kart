use super::{edit_point, Select, Selection};
use common::{map::Map, types::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemBox(pub usize);
impl Selection {
    pub fn item_box(c_i: usize) -> Self {
        Selection::ItemBox(ItemBox(c_i))
    }
}

impl Select for ItemBox {
    fn translate(&self, map: &mut Map, delta: Vec2) {
        map.item_spawns[self.0] += delta;
    }

    fn edit_ui<'a>(&self, map: &'a mut Map, ui: &mut egui::Ui) {
        edit_point(ui, &mut map.item_spawns[self.0]);
    }
}
