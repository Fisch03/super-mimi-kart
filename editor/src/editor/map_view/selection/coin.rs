use super::{edit_point, Select, Selection};
use common::{map::Map, types::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coin(pub usize);
impl Selection {
    pub fn coin(c_i: usize) -> Self {
        Selection::Coin(Coin(c_i))
    }
}

impl Select for Coin {
    fn translate(&self, map: &mut Map, delta: Vec2) {
        map.coins[self.0] += delta;
    }

    fn edit_ui<'a>(&self, map: &'a mut Map, ui: &mut egui::Ui) {
        edit_point(ui, &mut map.coins[self.0]);
    }
}
