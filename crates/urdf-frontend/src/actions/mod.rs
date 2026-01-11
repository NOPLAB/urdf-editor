//! Action handling module
//!
//! This module contains the action dispatch system for the URDF editor.
//! Actions are queued in AppState and processed each frame.

mod assembly;
#[cfg(not(target_arch = "wasm32"))]
mod file;
mod part;

use crate::state::{AppAction, SharedAppState, SharedViewportState};

pub use assembly::handle_assembly_action;
#[cfg(not(target_arch = "wasm32"))]
pub use file::handle_file_action;
pub use part::handle_part_action;

/// Context for action handlers
pub struct ActionContext<'a> {
    pub app_state: &'a SharedAppState,
    pub viewport_state: &'a Option<SharedViewportState>,
}

impl<'a> ActionContext<'a> {
    pub fn new(
        app_state: &'a SharedAppState,
        viewport_state: &'a Option<SharedViewportState>,
    ) -> Self {
        Self {
            app_state,
            viewport_state,
        }
    }
}

/// Dispatch an action to the appropriate handler
pub fn dispatch_action(action: AppAction, ctx: &ActionContext) {
    match action {
        // File actions (native only)
        #[cfg(not(target_arch = "wasm32"))]
        AppAction::ImportStl(_)
        | AppAction::ImportUrdf(_)
        | AppAction::SaveProject(_)
        | AppAction::LoadProject(_)
        | AppAction::ExportUrdf { .. }
        | AppAction::NewProject => {
            handle_file_action(action, ctx);
        }

        // File actions (WASM - ignore)
        #[cfg(target_arch = "wasm32")]
        AppAction::ImportStl(_)
        | AppAction::ImportUrdf(_)
        | AppAction::SaveProject(_)
        | AppAction::LoadProject(_)
        | AppAction::ExportUrdf { .. } => {
            tracing::warn!("File actions are not supported in WASM");
        }

        #[cfg(target_arch = "wasm32")]
        AppAction::NewProject => {
            ctx.app_state.lock().new_project();
            if let Some(viewport_state) = ctx.viewport_state {
                viewport_state.lock().clear_parts();
                viewport_state.lock().clear_overlays();
            }
        }

        // Part actions
        AppAction::CreatePrimitive { .. }
        | AppAction::CreateEmpty { .. }
        | AppAction::SelectPart(_)
        | AppAction::DeleteSelectedPart
        | AppAction::UpdatePartTransform { .. } => {
            handle_part_action(action, ctx);
        }

        // Assembly actions
        AppAction::AddJointPoint { .. }
        | AppAction::RemoveJointPoint { .. }
        | AppAction::ConnectParts { .. }
        | AppAction::DisconnectPart { .. }
        | AppAction::UpdateJointPosition { .. }
        | AppAction::ResetJointPosition { .. }
        | AppAction::ResetAllJointPositions => {
            handle_assembly_action(action, ctx);
        }
    }
}
