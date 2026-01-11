//! PropertyComponent trait definition for Unity-style Inspector components

use egui::Ui;
use urdf_core::Part;
use uuid::Uuid;

/// Context passed to property components for rendering
pub struct PropertyContext<'a> {
    /// The part being edited
    pub part: &'a mut Part,
    /// Part ID (for state updates)
    #[allow(dead_code)]
    pub part_id: Uuid,
    /// Selected joint point index (if any)
    pub selected_joint_point: Option<usize>,
}

/// Trait for property panel components (Unity-style Inspector sections)
pub trait PropertyComponent {
    /// Component display name shown in the header
    fn name(&self) -> &str;

    /// Render the component UI
    /// Returns true if any value was changed
    fn ui(&mut self, ui: &mut Ui, ctx: &mut PropertyContext) -> bool;

    /// Whether this component is collapsible (default: true)
    fn is_collapsible(&self) -> bool {
        true
    }

    /// Whether the component is open by default (default: true)
    fn default_open(&self) -> bool {
        true
    }
}
