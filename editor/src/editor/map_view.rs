use common::map::{AssetId, Collider, Map};
use common::types::*;
use earcut::Earcut;
use egui::{
    Color32, Grid, Rect, Sense, Shape, Spinner, TextureFilter, TextureOptions, Window,
    epaint::{CircleShape, PathShape, RectShape},
    load::{SizedTexture, TexturePoll},
    pos2, vec2,
};

pub mod selection;
pub use selection::{PointSelect, SegmentSelect, Select, Selection};
// mod selection_window;
mod tools;
mod view_settings;

pub struct View {
    zoom: f32,
    pan: egui::Pos2,
    selection: Selection,
    tool: tools::Tool,
    dragging_selection: bool,

    start_viz_amt: usize,
}

impl Default for View {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: egui::Pos2::ZERO,
            tool: tools::Tool::Move,
            selection: Selection::None,
            dragging_selection: false,
            start_viz_amt: 10,
        }
    }
}

#[derive(Debug)]
enum LoadError {
    Pending,
    LoadError,
}

fn load_asset(ui: &mut egui::Ui, id: Option<AssetId>) -> Result<SizedTexture, LoadError> {
    let uri = id
        .map(|id| format!("smk://asset/{}", id.as_usize()))
        .unwrap_or_else(|| String::from("smk://asset/default"));

    let res = ui.ctx().try_load_texture(
        &uri,
        TextureOptions {
            magnification: TextureFilter::Nearest,
            // minification: TextureFilter::Nearest,
            ..Default::default()
        },
        Default::default(),
    );

    match res {
        Ok(TexturePoll::Ready { texture }) => Ok(texture),
        Ok(TexturePoll::Pending { .. }) => Err(LoadError::Pending),
        Err(err) => {
            log::error!("Failed to load asset {}: {}", uri, err);
            return Err(LoadError::LoadError);
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

        let bg_texture = match load_asset(ui, map.background) {
            Ok(texture) => texture,
            Err(LoadError::Pending) => {
                Spinner::new()
                    .paint_at(ui, Rect::from_center_size(rect.center(), vec2(50.0, 50.0)));
                ui.ctx().request_repaint();
                return;
            }
            Err(LoadError::LoadError) => {
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("⚠ Failed to load map image!"),
                    egui::FontId::proportional(32.0),
                    egui::Color32::RED,
                );
                return;
            }
        };

        if res.hovered() {
            let zoom_target = res.hover_pos().unwrap_or(rect.center());
            ui.ctx().input(|i| {
                let old_zoom = self.zoom;
                self.zoom *= 1.0 + i.smooth_scroll_delta.y * 0.001;

                let old_size = bg_texture.size * old_zoom;
                let new_size = bg_texture.size * self.zoom;
                let old_center = rect.center() - self.pan;
                let new_center = old_center
                    + (old_size - new_size) * (zoom_target - old_center).to_vec2() / old_size;

                self.pan -= new_center - old_center;
            })
        }

        let zoomed_size = bg_texture.size * self.zoom;
        let image_center_screen = rect.center() - self.pan;
        let image_rect = Rect::from_center_size(
            pos2(image_center_screen.x, image_center_screen.y),
            zoomed_size,
        );

        if res.drag_started() && ui.ctx().input(|i| !i.pointer.middle_down()) {
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
            if self.dragging_selection && ui.ctx().input(|i| !i.pointer.middle_down()) {
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
        } else if res.clicked_by(egui::PointerButton::Primary) {
            let click_pos = res.interact_pointer_pos().unwrap_or_default();
            let click_pos = (click_pos - image_center_screen) / self.zoom;

            if self.tool == tools::Tool::Move {
                self.selection = self.try_select(Vec2::new(click_pos.x, click_pos.y), map);
            }
            self.use_tool(map, Vec2::new(click_pos.x, click_pos.y));
        }

        ui.painter().image(
            bg_texture.id,
            image_rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        for (start, _) in map.track.iter_starts().take(self.start_viz_amt) {
            let start = pos2(start.x, start.y) * self.zoom + image_center_screen;
            ui.painter().circle_filled(start, 5.0, Color32::GREEN);
        }

        let mut circles = Vec::with_capacity(map.track.path.len() + map.colliders.len() * 4);
        map.offroad.iter().enumerate().for_each(|offroad| {
            self.triangulate(
                offroad,
                ui,
                map,
                Color32::from_rgba_premultiplied(50, 50, 0, 30),
                image_center_screen,
                &mut circles,
                match self.selection {
                    Selection::OffroadSegment(s) if s.offroad.0 == offroad.0 => {
                        Some(s.segment(map))
                    }
                    _ => None,
                },
                match self.selection {
                    Selection::OffroadPoint(p) if p.offroad.0 == offroad.0 => Some(p.point(map)),
                    _ => None,
                },
            );
        });

        map.colliders.iter().enumerate().for_each(|collider| {
            self.triangulate(
                collider,
                ui,
                map,
                Color32::from_rgba_premultiplied(255, 0, 0, 100),
                image_center_screen,
                &mut circles,
                match self.selection {
                    Selection::ColliderSegment(s) if s.collider.0 == collider.0 => {
                        Some(s.segment(map))
                    }

                    _ => None,
                },
                match self.selection {
                    Selection::ColliderPoint(p) if p.collider.0 == collider.0 => Some(p.point(map)),
                    _ => None,
                },
            );
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
                let end = end.to_rounded();
                let (cp_left, cp_right) = end.checkpoint_positions();
                let cp_left = pos2(cp_left.x, cp_left.y) * self.zoom + image_center_screen;
                let cp_right = pos2(cp_right.x, cp_right.y) * self.zoom + image_center_screen;

                let start =
                    pos2(start.pos.x, start.pos.y).round() * self.zoom + image_center_screen;
                let end = pos2(end.pos.x, end.pos.y).round() * self.zoom + image_center_screen;

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
                ui.painter().add(Shape::LineSegment {
                    points: [cp_left, cp_right],
                    stroke: (3.0, Color32::ORANGE).into(),
                });

                circles.push(Shape::Circle(CircleShape {
                    center: start,
                    radius: 5.0,
                    fill: cp_color,
                    stroke: (1.0, Color32::BLACK).into(),
                }));
            });

        ui.painter().extend(circles);

        for (i, item_box) in map.item_spawns.iter().enumerate() {
            let item_box =
                pos2(item_box.x.round(), item_box.y.round()) * self.zoom + image_center_screen;

            ui.painter().add(Shape::Rect(RectShape::new(
                Rect::from_center_size(item_box, vec2(15.0, 15.0)),
                0.0,
                Color32::ORANGE,
                if self.selection == Selection::item_box(i) {
                    (3.0, Color32::RED)
                } else {
                    (1.0, Color32::BLACK)
                },
            )));
        }

        for (i, coin) in map.coins.iter().enumerate() {
            let coin =
                pos2(coin.x.round() + 0.5, coin.y.round() + 0.5) * self.zoom + image_center_screen;

            ui.painter().add(Shape::Circle(CircleShape {
                center: coin,
                radius: 6.0,
                fill: Color32::YELLOW,
                stroke: if self.selection == Selection::coin(i) {
                    (3.0, Color32::RED).into()
                } else {
                    (1.0, Color32::BLACK).into()
                },
            }));
        }

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

        for (i, item_box) in map.item_spawns.iter().enumerate() {
            if item_box.distance(pos) < tolerance {
                return Selection::item_box(i);
            }
        }

        for (i, coin) in map.coins.iter().enumerate() {
            if coin.distance(pos) < tolerance {
                return Selection::coin(i);
            }
        }

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
            if segment_dist < tolerance / 2.0 {
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
                if segment_dist < tolerance / 2.0 {
                    return Selection::collider_segment(c_i, p_i);
                }
            }
        }

        for (o_i, offroad) in map.offroad.iter().enumerate() {
            for (p_i, (start, end)) in offroad
                .shape
                .windows(2)
                .map(|points| (points[0], points[1]))
                .chain(std::iter::once((
                    *offroad.shape.last().unwrap(),
                    *offroad.shape.first().unwrap(),
                )))
                .enumerate()
            {
                let segment = Segment::new(start, end);
                let start_dist = start.distance(pos);
                let end_dist = end.distance(pos);
                let segment_dist = segment.distance(pos);
                if start_dist < tolerance && start_dist < segment_dist + tolerance * 5.0 {
                    return Selection::offroad_point(o_i, p_i);
                }
                if end_dist < tolerance && end_dist < segment_dist + tolerance * 5.0 {
                    return Selection::offroad_point(o_i, (p_i + 1) % offroad.shape.len());
                }
                if segment_dist < tolerance / 2.0 {
                    return Selection::offroad_segment(o_i, p_i);
                }
            }
        }

        return Selection::None;
    }

    fn triangulate(
        &mut self,
        (i, collider): (usize, &Collider),
        ui: &mut egui::Ui,
        map: &Map,
        fill: Color32,
        image_center_screen: egui::Vec2,
        circles: &mut Vec<Shape>,
        is_segment_selected: Option<Segment>,
        is_point_selected: Option<Vec2>,
    ) {
        // TODO: reuse earcut instance
        let mut earcut = Earcut::new();
        let mut tris: Vec<usize> = Vec::new();

        //TODO: only do this when colliders are changed
        earcut.earcut(collider.shape.iter().map(|p| [p.x, p.y]), &[], &mut tris);
        for tri in tris.chunks(3) {
            ui.painter().add(Shape::convex_polygon(
                tri.iter()
                    .map(|&i| {
                        let p = collider.shape[i];
                        pos2(p.x, p.y).round() * self.zoom + image_center_screen
                    })
                    .collect(),
                fill,
                egui::Stroke::NONE,
            ));
        }

        let outline = Shape::Path(PathShape::convex_polygon(
            collider
                .shape
                .iter()
                .enumerate()
                .map(|(p_i, p)| {
                    let p = pos2(p.x, p.y).round() * self.zoom + image_center_screen;

                    circles.push(Shape::Circle(CircleShape {
                        center: p,
                        radius: 5.0,
                        fill: Color32::WHITE,
                        stroke: (1.0, Color32::BLACK).into(),
                    }));

                    p
                })
                .collect(),
            Color32::from_rgba_premultiplied(0, 0, 0, 0),
            (1.0, Color32::BLACK),
        ));

        ui.painter().add(outline);

        // match self.selection {
        if let Some(s) = is_segment_selected {
            let p_1 = pos2(s.start.x, s.start.y).round() * self.zoom + image_center_screen;
            let p_2 = pos2(s.end.x, s.end.y).round() * self.zoom + image_center_screen;

            ui.painter().add(Shape::LineSegment {
                points: [p_1, p_2],
                stroke: (3.0, Color32::RED).into(),
            });
        }

        if let Some(p) = is_point_selected {
            let p = pos2(p.x, p.y).round() * self.zoom + image_center_screen;
            circles.push(Shape::Circle(CircleShape {
                center: p,
                radius: 5.0,
                fill: Color32::RED,
                stroke: (1.0, Color32::BLACK).into(),
            }));
        }
    }
}
