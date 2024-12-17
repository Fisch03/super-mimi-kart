use common::map::Map;
use common::types::*;
use egui::{
    epaint::CircleShape, pos2, vec2, Color32, DragValue, Grid, Rect, Sense, Shape, Spinner,
    TextureFilter, TextureOptions, Window,
};

pub struct View {
    zoom: f32,
    pan: egui::Pos2,
    map_image: String,
    selection: Selection,
    dragging_selection: bool,
}

#[derive(Debug, Eq, PartialEq)]
enum Selection {
    None,
    Point(usize),
    Segment(usize),
}

impl core::fmt::Display for Selection {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Selection::None => write!(f, "None"),
            Selection::Point(_) => write!(f, "Point"),
            Selection::Segment(_) => write!(f, "Segment"),
        }
    }
}

impl Default for View {
    fn default() -> Self {
        let location = web_sys::window().unwrap().location();
        let map_image = format!(
            "http://{}/assets/maps/mcircuit1/map.png",
            location.host().unwrap()
        );

        Self {
            zoom: 1.0,
            pan: egui::Pos2::ZERO,
            map_image,
            selection: Selection::None,
            dragging_selection: false,
        }
    }
}

impl View {
    pub fn show(&mut self, ui: &mut egui::Ui, map: &mut Map) {
        let (rect, res) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());

        // let tex_res = Image::new("https://files.cibo-online.net/xBVyOuvwP3u0.png")
        //     .load_for_size(ui.ctx(), rect.size());
        let tex_res = ui.ctx().try_load_texture(
            &self.map_image,
            TextureOptions {
                magnification: TextureFilter::Nearest,
                minification: TextureFilter::Nearest,
                ..Default::default()
            },
            egui::SizeHint::Size(rect.width() as u32, rect.height() as u32),
        );

        let texture_poll = match tex_res {
            Ok(texture_poll) => texture_poll,
            Err(err) => {
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("⚠ Failed to load map image!"),
                    egui::FontId::proportional(32.0),
                    egui::Color32::RED,
                );
                log::error!("Failed to load map image '{}': {}", self.map_image, err);
                return;
            }
        };

        let texture = match texture_poll {
            egui::load::TexturePoll::Pending { .. } => {
                Spinner::new()
                    .paint_at(ui, Rect::from_center_size(rect.center(), vec2(50.0, 50.0)));
                ui.ctx().request_repaint();
                return;
            }
            egui::load::TexturePoll::Ready { texture } => texture,
        };

        if res.hovered() {
            let zoom_target = res.hover_pos().unwrap_or(rect.center());
            ui.ctx().input(|i| {
                let old_zoom = self.zoom;
                self.zoom *= 1.0 + i.smooth_scroll_delta.y * 0.001;

                let old_size = texture.size * old_zoom;
                let new_size = texture.size * self.zoom;
                let old_center = rect.center() - self.pan;
                let new_center = old_center
                    + (old_size - new_size) * (zoom_target - old_center).to_vec2() / old_size;

                self.pan -= new_center - old_center;
            })
        }

        let zoomed_size = texture.size * self.zoom;
        let image_center_screen = rect.center() - self.pan;
        let image_rect = Rect::from_center_size(
            pos2(image_center_screen.x, image_center_screen.y),
            zoomed_size,
        );

        if res.drag_started() {
            if let Some(pos) = res.interact_pointer_pos() {
                let pos = (pos - image_center_screen) / self.zoom;

                let maybe_selection = self.try_select(pos, map);
                if maybe_selection != Selection::None {
                    self.selection = maybe_selection;
                    self.dragging_selection = true;
                } else {
                    self.dragging_selection = false;
                }
            } else {
                self.dragging_selection = false;
            }
        }

        if res.dragged() {
            if self.dragging_selection {
                let delta = res.drag_delta() / self.zoom;
                match self.selection {
                    Selection::Point(i) => {
                        let prev_i = (i + map.track.path.len() - 1) % map.track.path.len();

                        let prev = &mut map.track.path[prev_i];
                        prev.line.end.x += delta.x;
                        prev.line.end.y += delta.y;
                        let segment = &mut map.track.path[i];
                        segment.line.start.x += delta.x;
                        segment.line.start.y += delta.y;
                    }
                    Selection::Segment(i) => {
                        let prev_i = (i + map.track.path.len() - 1) % map.track.path.len();
                        let next_i = (i + 1) % map.track.path.len();

                        let prev = &mut map.track.path[prev_i];
                        prev.line.end.x += delta.x;
                        prev.line.end.y += delta.y;
                        let segment = &mut map.track.path[i];
                        segment.line.start.x += delta.x;
                        segment.line.start.y += delta.y;
                        segment.line.end.x += delta.x;
                        segment.line.end.y += delta.y;
                        let next = &mut map.track.path[next_i];
                        next.line.start.x += delta.x;
                        next.line.start.y += delta.y;
                    }
                    _ => {}
                }
            } else {
                self.pan -= res.drag_delta();
            }
        }

        if res.double_clicked() {
            let click_pos = res.interact_pointer_pos().unwrap_or_default();
            let click_pos = (click_pos - image_center_screen) / self.zoom;

            match self.selection {
                Selection::Segment(i) => {
                    let closest =
                        map.track.path[i].closest_point(Vec2::new(click_pos.x, click_pos.y));

                    let end = map.track.path[i].line.end;
                    map.track.path[i].line.end = (closest.x, closest.y).into();
                    map.track.path.insert(
                        i + 1,
                        common::map::Segment::new(
                            Vec2::new(closest.x, closest.y),
                            Vec2::new(end.x, end.y),
                        ),
                    );

                    self.selection = Selection::Point(i + 1);
                }
                _ => {}
            }
        } else if res.clicked() {
            let click_pos = res.interact_pointer_pos().unwrap_or_default();
            let click_pos = (click_pos - image_center_screen) / self.zoom;

            self.selection = self.try_select(click_pos, map);
        }

        ui.painter().image(
            texture.id,
            image_rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        for start in map.track.iter_starts().take(10) {
            let start = pos2(start.x, start.y) * self.zoom + image_center_screen;
            ui.painter().circle_filled(start, 5.0, Color32::GREEN);
        }

        let mut lines = Vec::with_capacity(map.track.path.len());
        let mut circles = Vec::with_capacity(map.track.path.len());
        map.track.path.iter().enumerate().for_each(|(i, segment)| {
            let start =
                pos2(segment.line.start.x, segment.line.start.y) * self.zoom + image_center_screen;
            let end =
                pos2(segment.line.end.x, segment.line.end.y) * self.zoom + image_center_screen;

            let line_color = if self.selection == Selection::Segment(i) {
                Color32::RED
            } else {
                Color32::WHITE
            };
            let cp_color = if self.selection == Selection::Point(i) {
                Color32::RED
            } else {
                if i == 0 {
                    Color32::GREEN
                } else {
                    Color32::WHITE
                }
            };

            lines.push(Shape::LineSegment {
                points: [start, end],
                stroke: (3.0, line_color).into(),
            });
            circles.push(Shape::Circle(CircleShape {
                center: start,
                radius: 5.0,
                fill: cp_color,
                stroke: (1.0, Color32::BLACK).into(),
            }));
        });

        ui.painter().extend(lines);
        ui.painter().extend(circles);

        if self.selection != Selection::None {
            Window::new(self.selection.to_string())
                .id(egui::Id::new("selection"))
                .anchor(egui::Align2::RIGHT_BOTTOM, vec2(-30.0, -30.0))
                .resizable(false)
                .collapsible(false)
                .movable(false)
                .show(ui.ctx(), |ui| {
                    Grid::new("metadata_grid")
                        .num_columns(2)
                        .show(ui, |ui| match self.selection {
                            Selection::Point(i) => {
                                let segment = &mut map.track.path[i];

                                ui.label("X");
                                ui.add(DragValue::new(&mut segment.line.start.x));
                                ui.end_row();

                                ui.label("Y");
                                ui.add(DragValue::new(&mut segment.line.start.y));
                                ui.end_row();

                                let start = segment.line.start;
                                let prev_i = (i + map.track.path.len() - 1) % map.track.path.len();
                                map.track.path[prev_i].line.end = start;
                            }
                            Selection::Segment(i) => {
                                let segment = &mut map.track.path[i];
                                ui.strong("Start");
                                ui.end_row();

                                ui.label("X");
                                ui.add(DragValue::new(&mut segment.line.start.x));
                                ui.end_row();

                                ui.label("Y");
                                ui.add(DragValue::new(&mut segment.line.start.y));
                                ui.end_row();

                                ui.strong("End");
                                ui.end_row();

                                ui.label("X");
                                ui.add(DragValue::new(&mut segment.line.end.x));
                                ui.end_row();

                                ui.label("Y");
                                ui.add(DragValue::new(&mut segment.line.end.y));
                                ui.end_row();

                                let start = segment.line.start;
                                let end = segment.line.end;
                                let prev_i = (i + map.track.path.len() - 1) % map.track.path.len();
                                map.track.path[prev_i].line.end = start;
                                let next_i = (i + 1) % map.track.path.len();
                                map.track.path[next_i].line.start = end;
                            }
                            _ => {}
                        })
                });
        }
    }

    fn try_select(&mut self, pos: egui::Pos2, map: &Map) -> Selection {
        for (i, segment) in map.track.path.iter().enumerate() {
            let start = pos2(segment.line.start.x, segment.line.start.y);
            let end = pos2(segment.line.end.x, segment.line.end.y);
            let start_dist = (start - pos).length();
            let end_dist = (end - pos).length();
            let segment_dist = segment.distance(common::types::Vec2::new(pos.x, pos.y));

            let tolerance = 15.0 / self.zoom;
            if start_dist < tolerance && start_dist < segment_dist + tolerance * 5.0 {
                return Selection::Point(i);
            }
            if end_dist < tolerance && end_dist < segment_dist + tolerance * 5.0 {
                return Selection::Point((i + 1) % map.track.path.len());
            }
            if segment_dist < tolerance {
                return Selection::Segment(i);
            }
        }

        return Selection::None;
    }
}
