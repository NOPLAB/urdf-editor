//! Visual component - color and material editing

use egui::Ui;

use crate::panels::properties::{PropertyComponent, PropertyContext};

/// Visual properties component (color, material)
pub struct VisualComponent;

impl VisualComponent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for VisualComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyComponent for VisualComponent {
    fn name(&self) -> &str {
        "Visual"
    }

    fn ui(&mut self, ui: &mut Ui, ctx: &mut PropertyContext) -> bool {
        let mut changed = false;
        let part = &mut ctx.part;

        // Color picker
        ui.horizontal(|ui| {
            ui.label("Color:");
            let mut color = egui::Color32::from_rgba_unmultiplied(
                (part.color[0] * 255.0) as u8,
                (part.color[1] * 255.0) as u8,
                (part.color[2] * 255.0) as u8,
                (part.color[3] * 255.0) as u8,
            );
            if ui.color_edit_button_srgba(&mut color).changed() {
                part.color = [
                    color.r() as f32 / 255.0,
                    color.g() as f32 / 255.0,
                    color.b() as f32 / 255.0,
                    color.a() as f32 / 255.0,
                ];
                changed = true;
            }
        });

        // Material name
        ui.horizontal(|ui| {
            ui.label("Material:");
            let mut material_name = part.material_name.clone().unwrap_or_default();
            if ui.text_edit_singleline(&mut material_name).changed() {
                part.material_name = if material_name.is_empty() {
                    None
                } else {
                    Some(material_name)
                };
                changed = true;
            }
        });

        changed
    }
}
