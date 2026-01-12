//! Export options for URDF generation

use std::path::PathBuf;

/// Export options for URDF generation
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Output directory
    pub output_dir: PathBuf,
    /// Robot name (for URDF root element)
    pub robot_name: String,
    /// Mesh package prefix (e.g., "package://robot_description")
    pub mesh_prefix: String,
    /// Whether to use package:// URIs or relative paths
    pub use_package_uri: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("."),
            robot_name: "robot".to_string(),
            mesh_prefix: "meshes".to_string(),
            use_package_uri: false,
        }
    }
}
