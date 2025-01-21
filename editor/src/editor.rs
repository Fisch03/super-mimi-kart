use common::map::Map;
use egui::{CentralPanel, SidePanel, TopBottomPanel};
use poll_promise::Promise;

mod map_io;
mod map_view;
mod properties_panel;

pub struct Editor {
    map: Map,
    view: map_view::View,
    map_db: map_io::MapDB,
    last_save: f64,
}

impl Editor {
    pub async fn new() -> Self {
        let map_db = map_io::MapDB::open().await.unwrap();
        let map = map_db.load().await;

        let map = match map {
            Ok(Some(map)) => map,
            Ok(None) => Map::default(),
            Err(e) => {
                log::error!("failed to load map: {:?}", e);
                Map::default()
            }
        };

        Self {
            map,
            map_db,
            view: map_view::View::default(),
            last_save: now(),
        }
    }

    #[allow(dead_code)]
    pub fn init_egui(cc: &eframe::CreationContext) {
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Bold);
        cc.egui_ctx.set_fonts(fonts);

        egui_extras::install_image_loaders(&cc.egui_ctx);
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
                            map_io::download_map(&self.map);
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

        if self.last_save + 10.0 < now() {
            self.last_save = now();
            let db = self.map_db.clone();
            let map = self.map.clone();
            Promise::spawn_local(async move {
                log::debug!("saving map");
                match db.save(map).await {
                    Ok(_) => log::debug!("map saved"),
                    Err(e) => log::error!("failed to save map: {:?}", e),
                }
            });
        }
    }
}

pub fn now() -> f64 {
    let performance = web_sys::window().unwrap().performance().unwrap();
    performance.now()
}
