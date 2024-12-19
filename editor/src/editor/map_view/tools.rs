use super::View;
use common::map::Map;
use egui_phosphor::bold;

impl View {
    pub fn show_tools(&self, ui: &mut egui::Ui, map: &mut Map) {
        if ui
            .heading(format!("{}", bold::PLUS_SQUARE))
            .on_hover_text("Add a collider")
            .clicked()
        {
            map.colliders.push(Default::default());
        }
        ui.separator();
    }
}
