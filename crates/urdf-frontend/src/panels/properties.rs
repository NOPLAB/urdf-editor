//! Properties panel

use glam::Vec3;
use uuid::Uuid;

use urdf_core::{JointLimits, JointPoint, JointType, MAX_JOINT_POINTS};

use crate::panels::Panel;
use crate::state::SharedAppState;

/// Properties panel for editing selected part
pub struct PropertiesPanel;

impl PropertiesPanel {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PropertiesPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Panel for PropertiesPanel {
    fn name(&self) -> &str {
        "Properties"
    }

    fn ui(&mut self, ui: &mut egui::Ui, app_state: &SharedAppState) {
        let mut state = app_state.lock();

        let Some(selected_id) = state.selected_part else {
            ui.weak("No part selected");
            return;
        };

        // Extract selection state before mutable borrow
        let selected_point = state.selected_joint_point.map(|(_, idx)| idx);

        let Some(part) = state.parts.get_mut(&selected_id) else {
            ui.weak("Selected part not found");
            return;
        };

        ui.heading("Part Properties");
        ui.separator();

        // Name
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut part.name);
        });

        ui.separator();

        // Physical properties
        ui.heading("Physical");

        ui.horizontal(|ui| {
            ui.label("Mass (kg):");
            ui.add(egui::DragValue::new(&mut part.mass).speed(0.01).range(0.001..=1000.0));
        });

        ui.collapsing("Inertia", |ui| {
            ui.horizontal(|ui| {
                ui.label("Ixx:");
                ui.add(egui::DragValue::new(&mut part.inertia.ixx).speed(0.0001));
            });
            ui.horizontal(|ui| {
                ui.label("Ixy:");
                ui.add(egui::DragValue::new(&mut part.inertia.ixy).speed(0.0001));
            });
            ui.horizontal(|ui| {
                ui.label("Ixz:");
                ui.add(egui::DragValue::new(&mut part.inertia.ixz).speed(0.0001));
            });
            ui.horizontal(|ui| {
                ui.label("Iyy:");
                ui.add(egui::DragValue::new(&mut part.inertia.iyy).speed(0.0001));
            });
            ui.horizontal(|ui| {
                ui.label("Iyz:");
                ui.add(egui::DragValue::new(&mut part.inertia.iyz).speed(0.0001));
            });
            ui.horizontal(|ui| {
                ui.label("Izz:");
                ui.add(egui::DragValue::new(&mut part.inertia.izz).speed(0.0001));
            });

            if ui.button("Auto-calculate from mesh").clicked() {
                part.inertia = urdf_core::InertiaMatrix::from_bounding_box(
                    part.mass,
                    part.bbox_min,
                    part.bbox_max,
                );
            }
        });

        ui.separator();

        // Visual properties
        ui.heading("Visual");

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
            }
        });

        ui.horizontal(|ui| {
            ui.label("Material:");
            let mut material_name = part.material_name.clone().unwrap_or_default();
            if ui.text_edit_singleline(&mut material_name).changed() {
                part.material_name = if material_name.is_empty() {
                    None
                } else {
                    Some(material_name)
                };
            }
        });

        ui.separator();

        // Geometry info
        ui.heading("Geometry");
        ui.label(format!("Vertices: {}", part.vertices.len()));
        ui.label(format!("Triangles: {}", part.indices.len() / 3));
        ui.label(format!(
            "Bounding Box: [{:.3}, {:.3}, {:.3}] to [{:.3}, {:.3}, {:.3}]",
            part.bbox_min[0], part.bbox_min[1], part.bbox_min[2],
            part.bbox_max[0], part.bbox_max[1], part.bbox_max[2]
        ));

        let size = part.size();
        ui.label(format!("Size: {:.3} x {:.3} x {:.3}", size.x, size.y, size.z));

        if let Some(ref path) = part.stl_path {
            ui.label(format!("STL: {}", path));
        }

        ui.separator();

        // Joint Points section
        ui.horizontal(|ui| {
            ui.heading("Joint Points");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("{}/{}", part.joint_points.len(), MAX_JOINT_POINTS));
            });
        });

        // Add button
        let can_add = part.joint_points.len() < MAX_JOINT_POINTS;
        ui.add_enabled_ui(can_add, |ui| {
            if ui.button("Add Joint Point").clicked() {
                let center = part.center();
                let point = JointPoint::new(
                    format!("point_{}", part.joint_points.len() + 1),
                    center,
                );
                let _ = part.add_joint_point(point);
            }
        });

        ui.separator();

        // List of joint points
        let mut point_to_remove: Option<Uuid> = None;
        let mut point_to_select: Option<usize> = None;

        egui::ScrollArea::vertical()
            .id_salt("joint_points_scroll")
            .show(ui, |ui| {
                for (idx, point) in part.joint_points.iter_mut().enumerate() {
                    let is_selected = selected_point == Some(idx);

                    ui.push_id(point.id, |ui| {
                        let header = egui::CollapsingHeader::new(&point.name)
                            .default_open(is_selected)
                            .show(ui, |ui| {
                                // Name
                                ui.horizontal(|ui| {
                                    ui.label("Name:");
                                    ui.text_edit_singleline(&mut point.name);
                                });

                                // Position
                                ui.horizontal(|ui| {
                                    ui.label("Position:");
                                });
                                ui.horizontal(|ui| {
                                    ui.label("X:");
                                    ui.add(egui::DragValue::new(&mut point.position.x).speed(0.01));
                                    ui.label("Y:");
                                    ui.add(egui::DragValue::new(&mut point.position.y).speed(0.01));
                                    ui.label("Z:");
                                    ui.add(egui::DragValue::new(&mut point.position.z).speed(0.01));
                                });

                                // Joint type
                                ui.horizontal(|ui| {
                                    ui.label("Type:");
                                    egui::ComboBox::from_id_salt(format!("joint_type_{}", point.id))
                                        .selected_text(point.joint_type.display_name())
                                        .show_ui(ui, |ui| {
                                            for jt in JointType::all() {
                                                ui.selectable_value(
                                                    &mut point.joint_type,
                                                    *jt,
                                                    jt.display_name(),
                                                );
                                            }
                                        });
                                });

                                // Axis (for revolute/continuous/prismatic)
                                if point.joint_type.has_axis() {
                                    ui.horizontal(|ui| {
                                        ui.label("Axis:");
                                        if ui.selectable_label(point.axis == Vec3::X, "X").clicked() {
                                            point.axis = Vec3::X;
                                        }
                                        if ui.selectable_label(point.axis == Vec3::Y, "Y").clicked() {
                                            point.axis = Vec3::Y;
                                        }
                                        if ui.selectable_label(point.axis == Vec3::Z, "Z").clicked() {
                                            point.axis = Vec3::Z;
                                        }
                                    });
                                }

                                // Limits (for revolute/prismatic)
                                if point.joint_type.has_limits() {
                                    let limits = point.limits.get_or_insert(JointLimits::default());
                                    ui.collapsing("Limits", |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label("Lower:");
                                            ui.add(egui::DragValue::new(&mut limits.lower).speed(0.01));
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Upper:");
                                            ui.add(egui::DragValue::new(&mut limits.upper).speed(0.01));
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Effort:");
                                            ui.add(egui::DragValue::new(&mut limits.effort).speed(1.0));
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Velocity:");
                                            ui.add(egui::DragValue::new(&mut limits.velocity).speed(0.1));
                                        });
                                    });
                                } else {
                                    point.limits = None;
                                }

                                // Delete button
                                ui.separator();
                                if ui.button("Delete").clicked() {
                                    point_to_remove = Some(point.id);
                                }
                            });

                        if header.header_response.clicked() {
                            point_to_select = Some(idx);
                        }
                    });
                }
            });

        // Handle actions
        if let Some(point_id) = point_to_remove {
            part.remove_joint_point(point_id);
        }

        let is_empty = part.joint_points.is_empty();

        // Update selection - drop state first
        drop(state);
        if let Some(idx) = point_to_select {
            app_state.lock().select_joint_point(selected_id, idx);
        }

        if is_empty {
            ui.weak("No joint points defined.\nClick 'Add Joint Point' to create one.");
        }
    }
}
