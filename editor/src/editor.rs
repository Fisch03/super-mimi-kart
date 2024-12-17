use common::{map::Map, types::Line};
use egui::{CentralPanel, SidePanel, TopBottomPanel};

mod left_panel;
mod map_view;

pub struct Editor {
    map: Map,
    view: map_view::View,
}

impl Editor {
    #[allow(dead_code)]
    pub fn new(cc: &eframe::CreationContext) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        Self {
            map: Map::default(),
            view: map_view::View::default(),
        }
    }
}

impl eframe::App for Editor {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Map Editor");
                ui.separator();

                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("New").clicked() {
                            self.map = Map::default();
                        }
                        if ui.button("Open").clicked() {
                            log::warn!("todo: open map file");
                        }
                        if ui.button("Save").clicked() {
                            log::warn!("todo: save/download map file");
                        }
                    });
                });
            });
        });

        SidePanel::left("left_panel")
            .frame(
                egui::Frame::default()
                    .inner_margin(egui::Margin::same(10.0))
                    .fill(ctx.style().visuals.window_fill()),
            )
            .min_width(250.0)
            .show(ctx, |ui| {
                left_panel::show(ui, &mut self.map);
            });

        CentralPanel::default().show(ctx, |ui| {
            self.view.show(ui, &mut self.map);
        });
    }
}
