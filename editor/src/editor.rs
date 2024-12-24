use common::map::Map;
use egui::{CentralPanel, SidePanel, TopBottomPanel};

mod map_view;
mod properties_panel;

pub struct Editor {
    map: Map,
    view: map_view::View,
}

impl Editor {
    #[allow(dead_code)]
    pub fn new(cc: &eframe::CreationContext) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Bold);
        cc.egui_ctx.set_fonts(fonts);

        egui_extras::install_image_loaders(&cc.egui_ctx);

        Self {
            map: Map::default(),
            view: map_view::View::default(),
        }
    }
}

impl eframe::App for Editor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Map Editor");
                ui.separator();

                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("New").clicked() {
                            // TODO: ask for confirmation if there are unsaved changes
                            self.map = Map::default();
                            self.view = map_view::View::default();
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

        SidePanel::left("tools_panel")
            .min_width(0.0)
            .default_width(0.0)
            .resizable(false)
            .frame(
                egui::Frame::default()
                    .inner_margin(egui::Margin::same(10.0))
                    .fill(ctx.style().visuals.window_fill()),
            )
            .show(ctx, |ui| {
                self.view.show_tools(ui, &mut self.map);
            });

        SidePanel::right("properties_panel")
            .frame(
                egui::Frame::default()
                    .inner_margin(egui::Margin::same(10.0))
                    .fill(ctx.style().visuals.window_fill()),
            )
            .show(ctx, |ui| {
                self.show_properties(ui);
            });

        CentralPanel::default()
            .frame(egui::Frame::default().inner_margin(egui::Margin::same(10.0)))
            .show(ctx, |ui| {
                self.view.show(ui, &mut self.map);
            });
    }
}
