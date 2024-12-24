use common::map::Map;
use common::types::*;
use egui::{
    epaint::{CircleShape, PathShape},
    pos2, vec2, Color32, Grid, Rect, Sense, Shape, Spinner, TextureFilter, TextureOptions, Window,
};

pub mod selection;
pub use selection::{SegmentSelect, Select, Selection};
// mod selection_window;
mod tools;
mod view_settings;

pub struct View {
    zoom: f32,
    pan: egui::Pos2,
    map_image: String,
    selection: Selection,
    dragging_selection: bool,

    start_viz_amt: usize,
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
            start_viz_amt: 10,
        }
    }
}

impl View {
    pub fn selection(&self) -> Selection {
        self.selection
    }

    pub fn select(&mut self, selection: Selection) {
        self.selection = selection;
        self.dragging_selection = false;
    }

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

                let maybe_selection = self.try_select(Vec2::new(pos.x, pos.y), map);
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
                self.selection.translate(map, Vec2::new(delta.x, delta.y));
            } else {
                self.pan -= res.drag_delta();
            }
        }

        if res.double_clicked() {
            let click_pos = res.interact_pointer_pos().unwrap_or_default();
            let click_pos = (click_pos - image_center_screen) / self.zoom;

            if let Some(s) = self.selection.as_segment() {
                let closest = s
                    .segment(map)
                    .closest_point(Vec2::new(click_pos.x, click_pos.y));

                s.insert_point(map, closest);
            }
        } else if res.clicked() {
            let click_pos = res.interact_pointer_pos().unwrap_or_default();
            let click_pos = (click_pos - image_center_screen) / self.zoom;

            self.selection = self.try_select(Vec2::new(click_pos.x, click_pos.y), map);
        }

        ui.painter().image(
            texture.id,
            image_rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        for start in map.track.iter_starts().take(self.start_viz_amt) {
            let start = pos2(start.x, start.y) * self.zoom + image_center_screen;
            ui.painter().circle_filled(start, 5.0, Color32::GREEN);
        }

        let mut circles = Vec::with_capacity(map.track.path.len() + map.colliders.len() * 4);
        map.colliders
            .iter()
            .enumerate()
            .for_each(|(c_i, collider)| {
                let polygon = Shape::Path(PathShape::convex_polygon(
                    collider
                        .shape
                        .iter()
                        .enumerate()
                        .map(|(p_i, p)| {
                            let p = pos2(p.x, p.y) * self.zoom + image_center_screen;

                            let color = if self.selection == Selection::collider_point(c_i, p_i) {
                                Color32::RED
                            } else {
                                Color32::WHITE
                            };

                            circles.push(Shape::Circle(CircleShape {
                                center: p,
                                radius: 5.0,
                                fill: color,
                                stroke: (1.0, Color32::BLACK).into(),
                            }));

                            p
                        })
                        .collect(),
                    Color32::from_rgba_premultiplied(255, 0, 0, 50),
                    (1.0, Color32::BLACK),
                ));

                ui.painter().add(polygon);

                match self.selection {
                    Selection::ColliderSegment(s) if s.collider.0 == c_i => {
                        let segment = s.segment(map);
                        let p_1 = pos2(segment.start.x, segment.start.y) * self.zoom
                            + image_center_screen;
                        let p_2 =
                            pos2(segment.end.x, segment.end.y) * self.zoom + image_center_screen;

                        ui.painter().add(Shape::LineSegment {
                            points: [p_1, p_2],
                            stroke: (3.0, Color32::RED).into(),
                        });
                    }
                    _ => {}
                }
            });

        map.track
            .path
            .windows(2)
            .map(|points| (&points[0], &points[1]))
            .chain(std::iter::once((
                map.track.path.last().unwrap(),
                map.track.path.first().unwrap(),
            )))
            .enumerate()
            .for_each(|(i, (start, end))| {
                let start = pos2(start.pos.x, start.pos.y) * self.zoom + image_center_screen;
                let end = pos2(end.pos.x, end.pos.y) * self.zoom + image_center_screen;

                let line_color = if self.selection == Selection::track_segment(i) {
                    Color32::RED
                } else {
                    Color32::WHITE
                };
                let cp_color = if self.selection == Selection::track_point(i) {
                    Color32::RED
                } else {
                    if i == 0 {
                        Color32::GREEN
                    } else {
                        Color32::WHITE
                    }
                };

                ui.painter().add(Shape::LineSegment {
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

        ui.painter().extend(circles);

        if self.selection != Selection::None {
            Window::new(self.selection.to_string())
                .id(egui::Id::new("selection"))
                .constrain_to(rect)
                .anchor(egui::Align2::RIGHT_BOTTOM, vec2(-20.0, -20.0))
                .resizable(false)
                .collapsible(false)
                .movable(false)
                .show(ui.ctx(), |ui| {
                    Grid::new("metadata_grid").num_columns(2).show(ui, |ui| {
                        self.selection.edit_ui(map, ui);
                    });
                });
        }
    }

    fn try_select(&mut self, pos: Vec2, map: &Map) -> Selection {
        let tolerance = 15.0 / self.zoom;

        for (i, (start, end)) in map
            .track
            .path
            .windows(2)
            .map(|points| (&points[0], &points[1]))
            .chain(std::iter::once((
                map.track.path.last().unwrap(),
                map.track.path.first().unwrap(),
            )))
            .enumerate()
        {
            let segment = Segment::new(start.pos, end.pos);

            let start_dist = start.pos.distance(pos);
            let end_dist = end.pos.distance(pos);
            let segment_dist = segment.distance(pos);

            if start_dist < tolerance && start_dist < segment_dist + tolerance * 5.0 {
                return Selection::track_point(i);
            }
            if end_dist < tolerance && end_dist < segment_dist + tolerance * 5.0 {
                return Selection::track_point((i + 1) % map.track.path.len());
            }
            if segment_dist < tolerance {
                return Selection::track_segment(i);
            }
        }

        for (c_i, collider) in map.colliders.iter().enumerate() {
            for (p_i, (start, end)) in collider
                .shape
                .windows(2)
                .map(|points| (points[0], points[1]))
                .chain(std::iter::once((
                    *collider.shape.last().unwrap(),
                    *collider.shape.first().unwrap(),
                )))
                .enumerate()
            {
                let segment = Segment::new(start, end);

                let start_dist = start.distance(pos);
                let end_dist = end.distance(pos);
                let segment_dist = segment.distance(pos);

                if start_dist < tolerance && start_dist < segment_dist + tolerance * 5.0 {
                    return Selection::collider_point(c_i, p_i);
                }
                if end_dist < tolerance && end_dist < segment_dist + tolerance * 5.0 {
                    return Selection::collider_point(c_i, (p_i + 1) % collider.shape.len());
                }
                if segment_dist < tolerance {
                    return Selection::collider_segment(c_i, p_i);
                }
            }
        }

        return Selection::None;
    }
}
