use super::Editor;
use common::map::AssetId;
use egui::{Grid, Image, SliderClamping, TextureFilter, TextureOptions, TopBottomPanel, vec2};
use egui_phosphor::bold;

impl Editor {
    pub(super) fn show_properties(&mut self, ui: &mut egui::Ui) {
        ui.spacing_mut().item_spacing = egui::vec2(5.0, 5.0);

        ui.heading("Metadata");
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

        ui.heading("Track");
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
                egui::Slider::new(&mut self.map.track.start_offset_v, -50.0..=50.0)
                    .clamping(SliderClamping::Never),
            );
            ui.end_row();
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.heading("Assets");

            if ui.button(format!("{} Upload Asset", bold::PLUS)).clicked() {
                self.upload_asset();
            }
        });

        for i in 0..self.map.assets().len() {
            ui.push_id(i, |ui| {
                let id = AssetId::new(i);
                ui.horizontal(|ui| {
                    ui.add(
                        Image::new(format!("smk://asset/{}", id.as_usize()))
                            .texture_options(TextureOptions {
                                magnification: TextureFilter::Nearest,
                                ..Default::default()
                            })
                            .max_size(vec2(75.0, 75.0))
                            .fit_to_original_size(100.0),
                    );

                    Grid::new("asset_options").num_columns(2).show(ui, |ui| {
                        ui.label("Name");
                        self.map.asset_name_mut(id, |name| {
                            ui.text_edit_singleline(name);
                        });
                        ui.end_row();

                        ui.label("Usage");
                        ui.horizontal(|ui| {
                            macro_rules! usage_btn {
                                ($icon:expr, $usage:expr, $tooltip:expr) => {
                                    let selected = $usage == Some(id);
                                    if ui
                                        .selectable_label(selected, format!("{}", $icon))
                                        .on_hover_text($tooltip)
                                        .clicked()
                                    {
                                        if selected {
                                            $usage = None;
                                        } else {
                                            $usage = Some(id);
                                        }
                                    }
                                };
                            }

                            usage_btn!(
                                bold::IMAGE,
                                self.map.background,
                                "Set as background texture"
                            );
                            usage_btn!(bold::CUBE, self.map.item_box, "Set as item box texture");
                            usage_btn!(bold::COINS, self.map.coin, "Set as coin texture");
                        });
                        ui.end_row();

                        ui.label("Actions");
                        if ui
                            .button(format!("{} Delete", bold::TRASH))
                            .on_hover_text("Remove asset")
                            .clicked()
                        {
                            self.map.remove_asset(id);
                            self.asset_loader.load_map(&self.map);
                            ui.ctx().forget_all_images();
                        }
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

            ui.heading("Collider");
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
                ui.heading("View Settings");
                ui.add_space(2.0);
                self.view.show_view_settings(ui, &mut self.map);
            });
    }
}
