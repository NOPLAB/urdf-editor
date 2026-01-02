//! Part list panel with hierarchical tree structure and drag-and-drop

use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use urdf_core::StlUnit;

use crate::app_state::{AppAction, SharedAppState};
use crate::panels::Panel;

/// Part list panel with drag-and-drop hierarchy
pub struct PartListPanel {
    /// Currently dragged part ID
    dragging_part: Option<Uuid>,
    /// Current drop target
    drop_target: Option<Uuid>,
}

impl PartListPanel {
    pub fn new() -> Self {
        Self {
            dragging_part: None,
            drop_target: None,
        }
    }
}

impl Default for PartListPanel {
    fn default() -> Self {
        Self::new()
    }
}

/// Actions collected during tree rendering
enum TreeAction {
    Select(Uuid),
    Delete(Uuid),
    Disconnect(Uuid),
    Connect { parent: Uuid, child: Uuid },
}

impl Panel for PartListPanel {
    fn name(&self) -> &str {
        "Parts"
    }

    fn ui(&mut self, ui: &mut egui::Ui, app_state: &SharedAppState) {
        // Import STL toolbar
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

        // Collect state data
        let state = app_state.lock();
        let selected_id = state.selected_part;

        // Build tree structure from Assembly
        let (root_parts, children_map, parts_with_parent, orphaned_parts) = self.build_tree_structure(&state);

        // Collect part names for display
        let part_names: HashMap<Uuid, String> = state.parts.iter()
            .map(|(id, p)| (*id, p.name.clone()))
            .collect();

        let is_empty = state.parts.is_empty();
        drop(state);

        // Reset drop target each frame
        self.drop_target = None;

        // Collect actions during rendering
        let mut actions: Vec<TreeAction> = Vec::new();

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Render connected parts (tree structure)
            if !root_parts.is_empty() {
                ui.label(egui::RichText::new("Connected").strong());
                ui.separator();

                for root_id in &root_parts {
                    self.render_part_tree(
                        ui,
                        *root_id,
                        &part_names,
                        &children_map,
                        &parts_with_parent,
                        selected_id,
                        0,
                        &mut actions,
                    );
                }
            }

            // Render orphaned parts (not in assembly)
            if !orphaned_parts.is_empty() {
                if !root_parts.is_empty() {
                    ui.add_space(10.0);
                }
                ui.label(egui::RichText::new("Unconnected").weak());
                ui.separator();

                for part_id in &orphaned_parts {
                    if let Some(name) = part_names.get(part_id) {
                        self.render_orphan_part(
                            ui,
                            *part_id,
                            name,
                            selected_id,
                            &mut actions,
                        );
                    }
                }
            }
        });

        // Handle drop on release
        if ui.input(|i| i.pointer.any_released()) {
            if let Some(dragged_id) = self.dragging_part {
                if let Some(target_id) = self.drop_target {
                    // Dropped on another part - connect
                    if dragged_id != target_id {
                        let state = app_state.lock();
                        let can_connect = self.can_connect(&state, target_id, dragged_id);
                        drop(state);

                        if can_connect {
                            actions.push(TreeAction::Connect {
                                parent: target_id,
                                child: dragged_id,
                            });
                        }
                    }
                } else {
                    // Dropped outside (no target) - disconnect if has parent
                    let state = app_state.lock();
                    let has_parent = state.project.assembly.links.iter()
                        .find(|(_, l)| l.part_id == dragged_id)
                        .and_then(|(link_id, _)| state.project.assembly.parent.get(link_id))
                        .is_some();
                    drop(state);

                    if has_parent {
                        actions.push(TreeAction::Disconnect(dragged_id));
                    }
                }
            }
            // Clear drag state
            self.dragging_part = None;
            self.drop_target = None;
        }

        // Process collected actions
        for action in actions {
            match action {
                TreeAction::Select(id) => {
                    app_state.lock().queue_action(AppAction::SelectPart(Some(id)));
                }
                TreeAction::Delete(id) => {
                    app_state.lock().queue_action(AppAction::SelectPart(Some(id)));
                    app_state.lock().queue_action(AppAction::DeleteSelectedPart);
                }
                TreeAction::Disconnect(id) => {
                    app_state.lock().queue_action(AppAction::DisconnectPart { child: id });
                }
                TreeAction::Connect { parent, child } => {
                    // ConnectParts handler will disconnect existing parent if needed
                    app_state.lock().queue_action(AppAction::ConnectParts { parent, child });
                }
            }
        }

        if is_empty {
            ui.weak("No parts loaded.\nClick 'Import STL...' to add parts.");
        }
    }
}

impl PartListPanel {
    /// Build tree structure from Assembly state
    fn build_tree_structure(
        &self,
        state: &crate::app_state::AppState,
    ) -> (Vec<Uuid>, HashMap<Uuid, Vec<Uuid>>, HashSet<Uuid>, Vec<Uuid>) {
        let assembly = &state.project.assembly;

        // Map link_id -> part_id
        let link_to_part: HashMap<Uuid, Uuid> = assembly.links
            .iter()
            .map(|(link_id, link)| (*link_id, link.part_id))
            .collect();

        // Map part_id -> link_id
        let part_to_link: HashMap<Uuid, Uuid> = assembly.links
            .iter()
            .map(|(link_id, link)| (link.part_id, *link_id))
            .collect();

        // Build children map (part_id -> [child_part_ids])
        let mut children_map: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        for (parent_link_id, children) in &assembly.children {
            if let Some(&parent_part_id) = link_to_part.get(parent_link_id) {
                let child_parts: Vec<Uuid> = children
                    .iter()
                    .filter_map(|(_, child_link_id)| link_to_part.get(child_link_id).copied())
                    .collect();
                if !child_parts.is_empty() {
                    children_map.insert(parent_part_id, child_parts);
                }
            }
        }

        // Identify parts with parents
        let parts_with_parent: HashSet<Uuid> = assembly.parent.keys()
            .filter_map(|link_id| link_to_part.get(link_id).copied())
            .collect();

        // Find root parts (in assembly but no parent)
        let mut root_parts: Vec<Uuid> = Vec::new();
        if let Some(root_link_id) = assembly.root_link {
            if let Some(&part_id) = link_to_part.get(&root_link_id) {
                root_parts.push(part_id);
            }
        }

        // Find orphaned parts (not in assembly)
        let orphaned_parts: Vec<Uuid> = state.parts.keys()
            .filter(|part_id| !part_to_link.contains_key(part_id))
            .copied()
            .collect();

        (root_parts, children_map, parts_with_parent, orphaned_parts)
    }

    /// Check if connecting parent to child would be valid
    fn can_connect(&self, state: &crate::app_state::AppState, parent_part: Uuid, child_part: Uuid) -> bool {
        if parent_part == child_part {
            return false;
        }

        let assembly = &state.project.assembly;

        // Find link IDs
        let parent_link = assembly.links.iter()
            .find(|(_, l)| l.part_id == parent_part)
            .map(|(id, _)| *id);
        let child_link = assembly.links.iter()
            .find(|(_, l)| l.part_id == child_part)
            .map(|(id, _)| *id);

        match (parent_link, child_link) {
            (Some(p), Some(c)) => {
                // Check if connecting would create a cycle
                // by checking if child is an ancestor of parent
                let mut current = Some(p);
                while let Some(id) = current {
                    if id == c {
                        return false;
                    }
                    current = assembly.parent.get(&id).map(|(_, parent_id)| *parent_id);
                }
                true
            }
            _ => true, // If either isn't in assembly yet, no cycle possible
        }
    }

    /// Render a draggable/droppable part item
    fn render_part_item(
        &mut self,
        ui: &mut egui::Ui,
        part_id: Uuid,
        label_text: &str,
        is_selected: bool,
        has_parent: bool,
        actions: &mut Vec<TreeAction>,
    ) {
        let is_being_dragged = self.dragging_part == Some(part_id);
        let is_drop_target = self.drop_target == Some(part_id);

        // Visual style based on drag state
        let text = if is_being_dragged {
            egui::RichText::new(label_text).italics().weak()
        } else if is_drop_target {
            egui::RichText::new(label_text).strong().color(egui::Color32::GREEN)
        } else {
            egui::RichText::new(label_text)
        };

        // Create interactive area with click and drag
        let response = ui.add(
            egui::Label::new(text)
                .selectable(false)
                .sense(egui::Sense::click_and_drag())
        );

        // Draw selection background
        if is_selected {
            ui.painter().rect_filled(
                response.rect,
                2.0,
                ui.visuals().selection.bg_fill,
            );
        }

        // Context menu
        response.context_menu(|ui| {
            if has_parent {
                if ui.button("Disconnect").clicked() {
                    actions.push(TreeAction::Disconnect(part_id));
                    ui.close_menu();
                }
            }
            if ui.button("Delete").clicked() {
                actions.push(TreeAction::Delete(part_id));
                ui.close_menu();
            }
        });

        // Handle drag start
        if response.drag_started() {
            self.dragging_part = Some(part_id);
        }

        // Handle as drop target when something else is being dragged
        if let Some(dragging) = self.dragging_part {
            if dragging != part_id && response.hovered() {
                self.drop_target = Some(part_id);
            }
        }

        // Selection on click
        if response.clicked() {
            actions.push(TreeAction::Select(part_id));
        }
    }

    /// Render a part in the tree with its children
    fn render_part_tree(
        &mut self,
        ui: &mut egui::Ui,
        part_id: Uuid,
        part_names: &HashMap<Uuid, String>,
        children_map: &HashMap<Uuid, Vec<Uuid>>,
        parts_with_parent: &HashSet<Uuid>,
        selected_id: Option<Uuid>,
        depth: usize,
        actions: &mut Vec<TreeAction>,
    ) {
        let Some(name) = part_names.get(&part_id) else { return };
        let children = children_map.get(&part_id);
        let has_children = children.is_some_and(|c| !c.is_empty());
        let has_parent = parts_with_parent.contains(&part_id);
        let is_selected = selected_id == Some(part_id);

        ui.push_id(part_id, |ui| {
            let indent = depth as f32 * 16.0;

            // Tree icon
            let icon = if has_children { "▼" } else { "●" };
            let label_text = format!("{} {}", icon, name);

            ui.horizontal(|ui| {
                ui.add_space(indent);
                self.render_part_item(ui, part_id, &label_text, is_selected, has_parent, actions);
            });

            // Render children
            if let Some(children) = children {
                for child_id in children {
                    self.render_part_tree(
                        ui,
                        *child_id,
                        part_names,
                        children_map,
                        parts_with_parent,
                        selected_id,
                        depth + 1,
                        actions,
                    );
                }
            }
        });
    }

    /// Render an orphaned (unconnected) part
    fn render_orphan_part(
        &mut self,
        ui: &mut egui::Ui,
        part_id: Uuid,
        name: &str,
        selected_id: Option<Uuid>,
        actions: &mut Vec<TreeAction>,
    ) {
        let is_selected = selected_id == Some(part_id);
        let label_text = format!("○ {}", name);

        ui.push_id(part_id, |ui| {
            ui.horizontal(|ui| {
                self.render_part_item(ui, part_id, &label_text, is_selected, false, actions);
            });
        });
    }
}
