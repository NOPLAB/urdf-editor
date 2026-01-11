//! Dock layout and tab viewer

use egui_dock::{DockState, NodeIndex, TabViewer};

use crate::panels::{Panel, PartListPanel, PropertiesPanel, ViewportPanel};
use crate::state::{SharedAppState, SharedViewportState};

/// Panel types for the dock system
pub enum PanelType {
    Viewport(ViewportPanel),
    PartList(PartListPanel),
    Properties(PropertiesPanel),
}

impl PanelType {
    pub fn name(&self) -> &str {
        match self {
            PanelType::Viewport(p) => p.name(),
            PanelType::PartList(p) => p.name(),
            PanelType::Properties(p) => p.name(),
        }
    }
}

/// Tab viewer for dock area
pub struct UrdfTabViewer<'a> {
    pub app_state: &'a SharedAppState,
    pub render_state: Option<&'a egui_wgpu::RenderState>,
    pub viewport_state: &'a Option<SharedViewportState>,
}

impl TabViewer for UrdfTabViewer<'_> {
    type Tab = PanelType;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.name().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            PanelType::Viewport(panel) => {
                if let (Some(render_state), Some(viewport_state)) =
                    (self.render_state, self.viewport_state)
                {
                    panel.ui_with_render_context(ui, self.app_state, render_state, viewport_state);
                } else {
                    panel.ui(ui, self.app_state);
                }
            }
            PanelType::PartList(panel) => panel.ui(ui, self.app_state),
            PanelType::Properties(panel) => {
                if let (Some(render_state), Some(viewport_state)) =
                    (self.render_state, self.viewport_state)
                {
                    panel.ui_with_render_context(ui, self.app_state, render_state, viewport_state);
                } else {
                    panel.ui(ui, self.app_state);
                }
            }
        }
    }
}

/// Create the default dock layout
pub fn create_dock_layout() -> DockState<PanelType> {
    let mut dock_state = DockState::new(vec![PanelType::Viewport(ViewportPanel::new())]);

    // Get the main surface
    let surface = dock_state.main_surface_mut();

    // Split right for properties
    let [_viewport, _right] = surface.split_right(
        NodeIndex::root(),
        0.75,
        vec![PanelType::Properties(PropertiesPanel::new())],
    );

    // Split left for parts list (now includes hierarchy via tree structure)
    let [_left, _viewport] = surface.split_left(
        NodeIndex::root(),
        0.2,
        vec![PanelType::PartList(PartListPanel::new())],
    );

    dock_state
}
