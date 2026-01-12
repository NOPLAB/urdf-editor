//! Project file serialization

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::assembly::Assembly;
use crate::part::Part;

/// Serialization format for backward compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectData {
    version: u32,
    name: String,
    parts: Vec<Part>,
    assembly: Assembly,
    materials: Vec<MaterialDef>,
}

/// Project file containing all editor state
#[derive(Debug, Clone)]
pub struct Project {
    /// File format version
    pub version: u32,
    /// Project name
    pub name: String,
    /// All parts in the project (keyed by ID for O(1) lookup)
    parts: HashMap<Uuid, Part>,
    /// Robot assembly
    pub assembly: Assembly,
    /// Material definitions
    pub materials: Vec<MaterialDef>,
}

impl From<Project> for ProjectData {
    fn from(project: Project) -> Self {
        Self {
            version: project.version,
            name: project.name,
            parts: project.parts.into_values().collect(),
            assembly: project.assembly,
            materials: project.materials,
        }
    }
}

impl From<ProjectData> for Project {
    fn from(data: ProjectData) -> Self {
        let parts = data.parts.into_iter().map(|p| (p.id, p)).collect();
        Self {
            version: data.version,
            name: data.name,
            parts,
            assembly: data.assembly,
            materials: data.materials,
        }
    }
}

impl Serialize for Project {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let data = ProjectData {
            version: self.version,
            name: self.name.clone(),
            parts: self.parts.values().cloned().collect(),
            assembly: self.assembly.clone(),
            materials: self.materials.clone(),
        };
        data.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Project {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let data = ProjectData::deserialize(deserializer)?;
        Ok(Project::from(data))
    }
}

impl Default for Project {
    fn default() -> Self {
        Self::new("New Project")
    }
}

impl Project {
    /// Create a new empty project
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            version: 1,
            name: name.into(),
            parts: HashMap::new(),
            assembly: Assembly::default(),
            materials: Vec::new(),
        }
    }

    /// Create a project with all fields specified (used by import)
    pub fn with_parts(
        name: impl Into<String>,
        parts: HashMap<Uuid, Part>,
        assembly: Assembly,
        materials: Vec<MaterialDef>,
    ) -> Self {
        Self {
            version: 1,
            name: name.into(),
            parts,
            assembly,
            materials,
        }
    }

    /// Save project to a file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ProjectError> {
        let path = path.as_ref();
        let content = self.to_bytes()?;
        std::fs::write(path, content).map_err(|e| ProjectError::Io(e.to_string()))?;
        Ok(())
    }

    /// Serialize project to bytes (for WASM support)
    pub fn to_bytes(&self) -> Result<Vec<u8>, ProjectError> {
        let content = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(|e| ProjectError::Serialize(e.to_string()))?;
        Ok(content.into_bytes())
    }

    /// Load project from a file
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ProjectError> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|e| ProjectError::Io(e.to_string()))?;
        let project: Project =
            ron::from_str(&content).map_err(|e| ProjectError::Deserialize(e.to_string()))?;
        Ok(project)
    }

    /// Load project from bytes (for WASM support)
    pub fn load_from_bytes(data: &[u8]) -> Result<Self, ProjectError> {
        let content =
            std::str::from_utf8(data).map_err(|e| ProjectError::Deserialize(e.to_string()))?;
        let project: Project =
            ron::from_str(content).map_err(|e| ProjectError::Deserialize(e.to_string()))?;
        Ok(project)
    }

    // ============== Part Accessors ==============

    /// Get a reference to the parts map
    pub fn parts(&self) -> &HashMap<Uuid, Part> {
        &self.parts
    }

    /// Get a mutable reference to the parts map
    pub fn parts_mut(&mut self) -> &mut HashMap<Uuid, Part> {
        &mut self.parts
    }

    /// Iterate over all parts
    pub fn parts_iter(&self) -> impl Iterator<Item = &Part> {
        self.parts.values()
    }

    /// Add a part to the project, returns the part ID
    pub fn add_part(&mut self, part: Part) -> Uuid {
        let id = part.id;
        self.parts.insert(id, part);
        id
    }

    /// Get a part by ID
    pub fn get_part(&self, id: Uuid) -> Option<&Part> {
        self.parts.get(&id)
    }

    /// Get a mutable part by ID
    pub fn get_part_mut(&mut self, id: Uuid) -> Option<&mut Part> {
        self.parts.get_mut(&id)
    }

    /// Remove a part by ID
    pub fn remove_part(&mut self, id: Uuid) -> Option<Part> {
        self.parts.remove(&id)
    }
}

/// Material definition for URDF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialDef {
    pub name: String,
    pub color: [f32; 4],
    pub texture: Option<String>,
}

impl MaterialDef {
    pub fn new(name: impl Into<String>, color: [f32; 4]) -> Self {
        Self {
            name: name.into(),
            color,
            texture: None,
        }
    }
}

/// Project-related errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum ProjectError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("Serialization error: {0}")]
    Serialize(String),
    #[error("Deserialization error: {0}")]
    Deserialize(String),
}
