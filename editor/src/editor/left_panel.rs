use common::map::Map;
use egui::{Grid, SliderClamping};

pub fn show(ui: &mut egui::Ui, map: &mut Map) {
    ui.strong("Metadata");
    ui.add_space(2.0);

    Grid::new("metadata_grid").num_columns(2).show(ui, |ui| {
        ui.label("Name");
        ui.text_edit_singleline(&mut map.metadata.name);
        ui.end_row();

        ui.label("Author");
        ui.text_edit_singleline(&mut map.metadata.author);
        ui.end_row();

        ui.label("Description");
        ui.text_edit_multiline(&mut map.metadata.description);
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
            egui::Slider::new(&mut map.track.start_offset_h, 0.0..=50.0)
                .clamping(SliderClamping::Never),
        );
        map.track.start_offset_h = map.track.start_offset_h.max(0.0);
        ui.end_row();

        ui.label("Start Offset V");
        ui.add(
            egui::Slider::new(&mut map.track.start_offset_v, 0.0..=50.0)
                .clamping(SliderClamping::Never),
        );
        map.track.start_offset_v = map.track.start_offset_v.max(10.0);
        ui.end_row();
    });
}
