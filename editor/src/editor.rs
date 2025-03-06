use crate::{AssetLoader, AssetUpload};
use common::map::Map;
use egui::{CentralPanel, SidePanel, TopBottomPanel};
use poll_promise::Promise;
use std::sync::Arc;

mod map_io;
mod map_view;
mod properties_panel;

pub struct Editor {
    map: Map,
    map_upload: Option<map_io::MapUpload>,
    view: map_view::View,
    map_db: map_io::MapDB,

    last_save: f64,
    full_save: bool,

    asset_loader: Arc<AssetLoader>,
    asset_uploads: Vec<AssetUpload>,
}

impl Editor {
    pub async fn new() -> Self {
        let map_db = map_io::MapDB::open().await.unwrap();
        let map = map_db.load().await;

        let mut map = match map {
            Ok(Some(map)) => map,
            Ok(None) => Map::default(),
            Err(e) => {
                log::error!("failed to load map: {:?}", e);
                Map::default()
            }
        };
        map.round_all();

        let asset_loader = Arc::new(AssetLoader::new(&map));

        Self {
            map,
            map_upload: None,
            map_db,
            view: map_view::View::default(),

            last_save: now(),
            full_save: false,

            asset_loader,
            asset_uploads: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn init_egui(&self, cc: &eframe::CreationContext) {
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Bold);
        cc.egui_ctx.set_fonts(fonts);

        egui_extras::install_image_loaders(&cc.egui_ctx);
        cc.egui_ctx.add_image_loader(self.asset_loader.clone());
    }

    fn replace_map(&mut self, mut map: Map) {
        map.round_all();
        self.asset_loader.load_map(&map);
        self.map = map;
        self.view = map_view::View::default();
    }

    fn upload_asset(&mut self) {
        let upload = AssetUpload::start();
        let upload = match upload {
            Ok(upload) => upload,
            Err(e) => {
                log::error!("failed to start asset upload: {:?}", e);
                return;
            }
        };
        self.asset_uploads.push(upload);
    }
}

impl eframe::App for Editor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(map_upload) = &mut self.map_upload {
            match map_upload.poll() {
                Some(Ok(map)) => {
                    self.replace_map(map);
                    self.map_upload = None;
                    self.full_save = true;
                }
                Some(Err(e)) => {
                    log::error!("failed to load map: {:?}", e);
                    self.map_upload = None;
                }
                None => {
                    ctx.request_repaint();
                }
            }
        }

        for upload in &mut self.asset_uploads {
            match upload.poll() {
                Some(Ok(asset)) => {
                    self.map.add_asset(asset);
                    self.asset_loader.load_map(&self.map);
                    self.full_save = true;
                }
                Some(Err(e)) => {
                    log::error!("failed to upload asset: {:?}", e);
                }
                None => {
                    ctx.request_repaint();
                }
            }
        }

        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Map Editor");
                ui.separator();

                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("New").clicked() {
                            // TODO: ask for confirmation if there are unsaved changes
                            self.replace_map(Map::default());
                        }
                        if ui.button("Open").clicked() {
                            let upload = map_io::MapUpload::start();
                            match upload {
                                Ok(upload) => self.map_upload = Some(upload),
                                Err(e) => log::error!("failed to start map upload: {:?}", e),
                            }
                        }
                        if ui.button("Save").clicked() {
                            self.map.round_all();
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
                    .inner_margin(egui::Margin::same(8.0))
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

        if self.last_save + 5000.0 < now() || self.full_save {
            self.last_save = now();
            let db = self.map_db.clone();
            let map = self.map.clone();

            let full_save = self.full_save;
            self.full_save = false;

            Promise::spawn_local(async move {
                let save_op = if full_save {
                    log::debug!("saving map with assets");
                    db.save_with_assets(map).await
                } else {
                    log::debug!("saving map");
                    db.save(map).await
                };

                match save_op {
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
