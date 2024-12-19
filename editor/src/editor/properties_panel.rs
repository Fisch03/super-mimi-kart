use egui::{Align, Grid, Layout, SliderClamping};
use super::{Editor};

impl Editor {
    pub(super) fn show_properties(&mut self, ui: &mut egui::Ui) {
        ui.strong("Metadata");
        ui.add_space(2.0);

        Grid::new("metadata_grid").num_columns(2).show(ui, |ui| {
            ui.label("Name");
            ui.text_edit_singleline(&mut self.map.metadata.name);
            ui.end_row();

            ui.label("Author");
            ui.text_edit_singleline(&mut self.map.metadata.author);
            ui.end_row();

            ui.label("Description");
            ui.text_edit_multiline(&mut self.map.metadata.description);
            ui.end_row();
        });

        ui.add_space(5.0);
        ui.separator();
        ui.add_space(5.0);

        ui.strong("Track");
        ui.add_space(2.0);

        Grid::new("track_grid").num_columns(2).show(ui, |ui| {
            ui.label("Start Offset H");
            ui.add(
                egui::Slider::new(&mut self.map.track.start_offset_h, 0.0..=50.0)
                    .clamping(SliderClamping::Never),
            );
            self.map.track.start_offset_h = self.map.track.start_offset_h.max(0.0);
            ui.end_row();

            ui.label("Start Offset V");
            ui.add(
                egui::Slider::new(&mut self.map.track.start_offset_v, 0.0..=50.0)
                    .clamping(SliderClamping::Never),
            );
            self.map.track.start_offset_v = self.map.track.start_offset_v.max(10.0);
            ui.end_row();
        });

        ui.with_layout(Layout::bottom_up(Align::default()), |ui| {
            ui.strong("View Settings");
            ui.add_space(2.0);
            self.view.show_view_settings(ui, &mut self.map);
        });
    }
}
