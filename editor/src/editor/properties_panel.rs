use super::{map_view::Selection, Editor};
use common::map::AssetId;
use egui::{vec2, Grid, Image, SliderClamping, TopBottomPanel};
use egui_phosphor::bold;

impl Editor {
    pub(super) fn show_properties(&mut self, ui: &mut egui::Ui) {
        ui.spacing_mut().item_spacing = egui::vec2(5.0, 5.0);

        ui.strong("Metadata");
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

        ui.separator();

        ui.strong("Track");
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

        ui.separator();

        ui.horizontal(|ui| {
            ui.strong("Assets");
            if ui.button(format!("{}", bold::PLUS)).clicked() {
                self.upload_asset(format!("Asset {}", self.map.assets().len() + 1));
            }
        });
        for i in 0..self.map.assets().len() {
            ui.push_id(i, |ui| {
                let id = AssetId::new(i);
                ui.horizontal(|ui| {
                    ui.add(
                        Image::new(format!("smk://asset/{}", id.as_usize()))
                            .max_size(vec2(75.0, 75.0))
                            .fit_to_original_size(100.0),
                    );

                    ui.vertical(|ui| {
                        self.map.asset_name_mut(id, |name| {
                            ui.text_edit_singleline(name);
                        });

                        ui.horizontal(|ui| {
                            if ui
                                .button(format!("{}", bold::IMAGE))
                                .on_hover_text("Set as background")
                                .clicked()
                            {
                                self.map.background = Some(AssetId::new(i));
                            }
                            if ui
                                .button(format!("{}", bold::TRASH))
                                .on_hover_text("Remove asset")
                                .clicked()
                            {
                                log::warn!("TODO: remove all references to asset {}", i);
                            }
                        });
                    });
                });
            });
        }

        /*
        for (i, collider) in self.map.colliders.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label("Collider");
                if ui.button(format!("{}", bold::CURSOR_CLICK)).clicked() {
                    self.view.select(Selection::collider(i))
                }
            });

            ui.strong("Collider");
            Grid::new("collider_grid").num_columns(2).show(ui, |ui| {});
        }
        */

        TopBottomPanel::bottom("view_settings")
            .frame(
                egui::Frame::default()
                    .inner_margin(egui::Margin {
                        top: 10.0,
                        ..Default::default()
                    })
                    .fill(ui.style().visuals.window_fill()),
            )
            .show_inside(ui, |ui| {
                ui.strong("View Settings");
                ui.add_space(2.0);
                self.view.show_view_settings(ui, &mut self.map);
            });
    }
}
