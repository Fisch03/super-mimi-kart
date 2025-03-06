use super::View;
use common::{
    map::{Collider, Map},
    types::*,
};
use egui::{vec2, RichText, TextStyle};
use egui_phosphor::bold;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Move,
    Collider,
    Coin,
    ItemBox,
}

impl View {
    pub fn show_tools(&mut self, ui: &mut egui::Ui, map: &mut Map) {
        ui.scope(|ui| {
            macro_rules! tool_button {
                ($tool:expr, $icon:expr, $text:expr) => {
                    if ui
                        .selectable_label(
                            self.tool == $tool,
                            RichText::new(format!("{}", $icon)).text_style(TextStyle::Heading),
                        )
                        .on_hover_text($text)
                        .clicked()
                    {
                        self.tool = $tool;
                    }

                    ui.separator();
                };
            }

            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            tool_button!(Tool::Move, bold::HAND, "Move");
            tool_button!(Tool::Collider, bold::PLUS_SQUARE, "Add a collider");
            tool_button!(Tool::ItemBox, bold::CUBE, "Add an item box");
            tool_button!(Tool::Coin, bold::COINS, "Add a coin");
        });
    }

    pub fn use_tool(&self, map: &mut Map, pos: Vec2) {
        match self.tool {
            Tool::Move => {}
            Tool::Collider => {
                let mut collider = Collider::default();
                collider.translate(pos);
                map.colliders.push(collider)
            }
            Tool::ItemBox => {
                map.item_spawns.push(pos);
            }
            Tool::Coin => {
                map.coins.push(pos);
            }
        }
    }
}
