use super::{Selection, View};
use common::map::Map;
use egui::{vec2, DragValue, Grid, Window};

impl View {
    pub(super) fn show_selection_window(&self, ui: &egui::Ui, map: &mut Map, rect: egui::Rect) {
        Window::new(self.selection.to_string())
            .id(egui::Id::new("selection"))
            .constrain_to(rect)
            .anchor(egui::Align2::RIGHT_BOTTOM, vec2(-20.0, -20.0))
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
                        Selection::ColliderPoint(c_i, p_i) => {
                            let collider = &mut map.colliders[c_i];
                            collider.shape.exterior_mut(|ext| {
                                let point = &mut ext.0[p_i];

                                ui.label("X");
                                ui.add(DragValue::new(&mut point.x).fixed_decimals(0));
                                ui.end_row();

                                ui.label("Y");
                                ui.add(DragValue::new(&mut point.y).fixed_decimals(0));
                                ui.end_row();
                            });
                        }
                        Selection::ColliderSegment(c_i, s_i) => {
                            let collider = &mut map.colliders[c_i];
                            collider.shape.exterior_mut(|ext| {
                                let p_1 = &mut ext.0[s_i];
                                ui.strong("Start");
                                ui.end_row();

                                ui.label("X");
                                ui.add(DragValue::new(&mut p_1.x).fixed_decimals(0));
                                ui.end_row();

                                ui.label("Y");
                                ui.add(DragValue::new(&mut p_1.y).fixed_decimals(0));
                                ui.end_row();
                                let p_1 = *p_1;

                                let p_2 = &mut ext.0[s_i - 1];
                                ui.strong("End");
                                ui.end_row();

                                ui.label("X");
                                ui.add(DragValue::new(&mut p_2.x).fixed_decimals(0));
                                ui.end_row();

                                ui.label("Y");
                                ui.add(DragValue::new(&mut p_2.y).fixed_decimals(0));
                                ui.end_row();
                                let p_2 = *p_2;

                                let len = ext.0.len();
                                if s_i == len - 1 {
                                    ext.0[0] = p_1;
                                } else if s_i == 1 {
                                    ext.0[len - 1] = p_2;
                                }
                            });
                        }
                        Selection::None => {}
                    })
            });
    }
}
