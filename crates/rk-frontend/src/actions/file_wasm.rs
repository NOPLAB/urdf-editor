//! WASM file I/O action handlers

use rk_core::{
    Part, Project, StlUnit, load_dae_from_bytes, load_obj_from_bytes, load_stl_from_bytes,
};

use crate::state::AppAction;

use super::ActionContext;

/// Handle WASM file-related actions (bytes-based)
pub fn handle_file_action_wasm(action: AppAction, ctx: &ActionContext) {
    match action {
        AppAction::ImportMeshBytes { name, data } => handle_import_mesh_bytes(&name, &data, ctx),
        AppAction::LoadProjectBytes { name, data } => handle_load_project_bytes(&name, &data, ctx),
        _ => {}
    }
}

/// Detect mesh format from filename extension
fn detect_mesh_format(filename: &str) -> Option<&'static str> {
    let lower = filename.to_lowercase();
    if lower.ends_with(".stl") {
        Some("stl")
    } else if lower.ends_with(".obj") {
        Some("obj")
    } else if lower.ends_with(".dae") {
        Some("dae")
    } else {
        None
    }
}

/// Extract part name from filename (removes extension)
fn extract_part_name(filename: &str) -> String {
    // Find the last dot and remove extension
    if let Some(pos) = filename.rfind('.') {
        filename[..pos].to_string()
    } else {
        filename.to_string()
    }
}

/// Load mesh from bytes based on format
fn load_mesh_from_bytes(filename: &str, data: &[u8], unit: StlUnit) -> Result<Part, String> {
    let part_name = extract_part_name(filename);
    let format = detect_mesh_format(filename);

    match format {
        Some("stl") => load_stl_from_bytes(&part_name, data, unit)
            .map_err(|e| format!("STL load error: {}", e)),
        Some("obj") => load_obj_from_bytes(&part_name, data, unit)
            .map_err(|e| format!("OBJ load error: {}", e)),
        Some("dae") => load_dae_from_bytes(&part_name, data, unit)
            .map_err(|e| format!("DAE load error: {}", e)),
        _ => Err(format!("Unsupported mesh format: {}", filename)),
    }
}

fn handle_import_mesh_bytes(filename: &str, data: &[u8], ctx: &ActionContext) {
    let unit = ctx.app_state.lock().stl_import_unit;
    match load_mesh_from_bytes(filename, data, unit) {
        Ok(part) => {
            tracing::info!(
                "Loaded mesh from bytes: {} ({} vertices, unit={:?})",
                part.name,
                part.vertices.len(),
                unit
            );

            // Calculate bounding sphere for camera fit
            let center = glam::Vec3::new(
                (part.bbox_min[0] + part.bbox_max[0]) / 2.0,
                (part.bbox_min[1] + part.bbox_max[1]) / 2.0,
                (part.bbox_min[2] + part.bbox_max[2]) / 2.0,
            );
            let extent = glam::Vec3::new(
                part.bbox_max[0] - part.bbox_min[0],
                part.bbox_max[1] - part.bbox_min[1],
                part.bbox_max[2] - part.bbox_min[2],
            );
            let radius = extent.length() / 2.0;

            // Add to viewport
            if let Some(viewport_state) = ctx.viewport_state {
                let mut vp = viewport_state.lock();
                vp.add_part(&part);
                vp.renderer.camera_mut().fit_all(center, radius);
            }

            // Add to app state
            ctx.app_state.lock().add_part(part);
        }
        Err(e) => {
            tracing::error!("Failed to load mesh from bytes: {}", e);
        }
    }
}

fn handle_load_project_bytes(_name: &str, data: &[u8], ctx: &ActionContext) {
    match Project::load_from_bytes(data) {
        Ok(project) => {
            tracing::info!("Loaded project from bytes: {}", project.name);

            // Clear viewport
            if let Some(viewport_state) = ctx.viewport_state {
                viewport_state.lock().clear_parts();
                viewport_state.lock().clear_overlays();
            }

            // Load parts into viewport
            if let Some(viewport_state) = ctx.viewport_state {
                for part in project.parts_iter() {
                    viewport_state.lock().add_part(part);
                }
            }

            // Load into app state (without file path for WASM)
            // No manual sync needed - parts are stored directly in project
            let mut state = ctx.app_state.lock();
            state.project = project;
            state.project_path = None;
            state.selected_part = None;
            state.modified = false;
        }
        Err(e) => {
            tracing::error!("Failed to load project from bytes: {}", e);
        }
    }
}
