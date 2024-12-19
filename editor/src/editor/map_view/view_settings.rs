use super::View;
use common::map::Map;
use egui::{Layout, Grid};
use egui_phosphor::bold;

impl View {
    pub fn show_view_settings(&mut self, ui: &mut egui::Ui, map: &mut Map) {
        Grid::new("view_settings").num_columns(2).show(ui, |ui| {
            ui.label("Zoom");
            ui.add(egui::Slider::new(&mut self.zoom, 0.1..=10.0));
            ui.end_row();

            ui.label("Start Vizualisation Amount");
            ui.add(egui::Slider::new(&mut self.start_viz_amt, 0..=50));
            ui.end_row();
        });
    }
}
