//! Part list panel

use uuid::Uuid;

use urdf_core::StlUnit;

use crate::app_state::{AppAction, SharedAppState};
use crate::panels::Panel;

/// Part list panel
pub struct PartListPanel;

impl PartListPanel {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PartListPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Panel for PartListPanel {
    fn name(&self) -> &str {
        "Parts"
    }

    fn ui(&mut self, ui: &mut egui::Ui, app_state: &SharedAppState) {
        ui.horizontal(|ui| {
            if ui.button("Import STL...").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("STL files", &["stl", "STL"])
                    .pick_file()
                {
                    app_state.lock().queue_action(AppAction::ImportStl(path));
                }
            }

            ui.separator();

            ui.label("Unit:");
            let mut state = app_state.lock();
            let current_unit = state.stl_import_unit;
            egui::ComboBox::from_id_salt("stl_unit")
                .selected_text(current_unit.name())
                .show_ui(ui, |ui| {
                    for unit in StlUnit::ALL {
                        ui.selectable_value(&mut state.stl_import_unit, *unit, unit.name());
                    }
                });
        });

        ui.separator();

        // Collect parts data with joint points
        let state = app_state.lock();
        let selected_id = state.selected_part;
        let selected_joint_point = state.selected_joint_point;
        let parts: Vec<_> = state
            .parts
            .values()
            .map(|p| {
                let joints: Vec<_> = p
                    .joint_points
                    .iter()
                    .enumerate()
                    .map(|(i, jp)| (i, jp.name.clone()))
                    .collect();
                (p.id, p.name.clone(), joints)
            })
            .collect();
        let is_empty = parts.is_empty();
        drop(state);

        // Tree view of parts and joint points
        let mut part_to_select: Option<Uuid> = None;
        let mut joint_to_select: Option<(Uuid, usize)> = None;

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (id, name, joints) in &parts {
                let is_part_selected =
                    selected_id == Some(*id) && selected_joint_point.is_none();
                let has_children = !joints.is_empty();

                ui.push_id(*id, |ui| {
                    // Part node with collapsing header
                    let header = egui::CollapsingHeader::new(name.as_str())
                        .default_open(selected_id == Some(*id))
                        .icon(move |ui, openness, response| {
                            // Custom icon: folder or triangle
                            let rect = response.rect;
                            let center = rect.center();
                            let stroke = ui.visuals().widgets.inactive.fg_stroke;

                            if has_children {
                                // Draw triangle
                                let size = 4.0;
                                let rotation = std::f32::consts::PI / 2.0 * openness;
                                let points = [
                                    egui::pos2(
                                        center.x + size * rotation.cos(),
                                        center.y + size * rotation.sin(),
                                    ),
                                    egui::pos2(
                                        center.x + size * (rotation + 2.094).cos(),
                                        center.y + size * (rotation + 2.094).sin(),
                                    ),
                                    egui::pos2(
                                        center.x + size * (rotation + 4.189).cos(),
                                        center.y + size * (rotation + 4.189).sin(),
                                    ),
                                ];
                                ui.painter()
                                    .add(egui::Shape::convex_polygon(points.to_vec(), stroke.color, stroke));
                            } else {
                                // Draw bullet for leaf
                                ui.painter().circle_filled(center, 3.0, stroke.color);
                            }
                        })
                        .show(ui, |ui| {
                            // Child nodes: joint points
                            for (idx, jp_name) in joints {
                                let is_jp_selected =
                                    selected_joint_point == Some((*id, *idx));

                                ui.horizontal(|ui| {
                                    ui.add_space(8.0);
                                    let icon = "○";
                                    if ui
                                        .selectable_label(
                                            is_jp_selected,
                                            format!("{} {}", icon, jp_name),
                                        )
                                        .clicked()
                                    {
                                        joint_to_select = Some((*id, *idx));
                                    }
                                });
                            }
                        });

                    // Part selection and context menu
                    let header_response = &header.header_response;

                    if header_response.clicked() && !header_response.double_clicked() {
                        part_to_select = Some(*id);
                    }

                    header_response.context_menu(|ui| {
                        if ui.button("Delete").clicked() {
                            app_state.lock().queue_action(AppAction::SelectPart(Some(*id)));
                            app_state.lock().queue_action(AppAction::DeleteSelectedPart);
                            ui.close_menu();
                        }
                    });

                    // Highlight selected part
                    if is_part_selected {
                        let rect = header_response.rect;
                        ui.painter().rect_stroke(
                            rect,
                            2.0,
                            egui::Stroke::new(1.0, ui.visuals().selection.stroke.color),
                        );
                    }
                });
            }
        });

        // Handle selections
        if let Some(id) = part_to_select {
            app_state.lock().queue_action(AppAction::SelectPart(Some(id)));
        }
        if let Some((part_id, idx)) = joint_to_select {
            app_state.lock().select_joint_point(part_id, idx);
        }

        if is_empty {
            ui.weak("No parts loaded.\nClick 'Import STL...' to add parts.");
        }
    }
}
