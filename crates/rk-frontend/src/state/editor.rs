//! Editor state types

/// Editor tool mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorTool {
    #[default]
    Select,
    Move,
    Rotate,
    AddJointPoint,
}

impl EditorTool {
    pub fn name(&self) -> &'static str {
        match self {
            EditorTool::Select => "Select",
            EditorTool::Move => "Move",
            EditorTool::Rotate => "Rotate",
            EditorTool::AddJointPoint => "Add Joint Point",
        }
    }
}

/// Primitive type for creating geometric shapes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveType {
    Box,
    Cylinder,
    Sphere,
}

impl PrimitiveType {
    pub fn name(&self) -> &'static str {
        match self {
            PrimitiveType::Box => "Box",
            PrimitiveType::Cylinder => "Cylinder",
            PrimitiveType::Sphere => "Sphere",
        }
    }
}
