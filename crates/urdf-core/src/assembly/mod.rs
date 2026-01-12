//! Assembly (scene graph) for robot structure

mod joint;
mod queries;
mod tree_cache;
mod types;

use std::cell::RefCell;
use std::collections::HashMap;

use glam::{Mat4, Quat};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::part::{JointPoint, JointType};

pub use joint::{Joint, JointBuilder, JointDynamics, JointMimic};
pub use types::{CollisionElement, InertialProperties, Link, Pose, VisualElement};

use tree_cache::TreeCache;

/// Raw assembly data for deserialization (used internally)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AssemblyData {
    name: String,
    links: HashMap<Uuid, Link>,
    joints: HashMap<Uuid, Joint>,
    joint_points: HashMap<Uuid, JointPoint>,
    children: HashMap<Uuid, Vec<(Uuid, Uuid)>>,
    parent: HashMap<Uuid, (Uuid, Uuid)>,
}

/// Robot assembly (scene graph)
#[derive(Debug, Clone, Serialize)]
#[serde(into = "AssemblyData")]
pub struct Assembly {
    pub name: String,
    /// All links
    pub links: HashMap<Uuid, Link>,
    /// All joints
    pub joints: HashMap<Uuid, Joint>,
    /// All joint points (managed by assembly, not by parts)
    pub joint_points: HashMap<Uuid, JointPoint>,
    /// Children mapping: parent_link -> [(joint_id, child_link)]
    pub children: HashMap<Uuid, Vec<(Uuid, Uuid)>>,
    /// Parent mapping: child_link -> (joint_id, parent_link)
    pub parent: HashMap<Uuid, (Uuid, Uuid)>,
    /// Name to ID index for links (O(1) lookup)
    pub(crate) link_name_index: HashMap<String, Uuid>,
    /// Name to ID index for joints (O(1) lookup)
    pub(crate) joint_name_index: HashMap<String, Uuid>,
    /// Cached tree structure for efficient traversal (interior mutability for lazy evaluation)
    cache: RefCell<TreeCache>,
}

impl From<Assembly> for AssemblyData {
    fn from(assembly: Assembly) -> Self {
        Self {
            name: assembly.name,
            links: assembly.links,
            joints: assembly.joints,
            joint_points: assembly.joint_points,
            children: assembly.children,
            parent: assembly.parent,
        }
    }
}

impl From<AssemblyData> for Assembly {
    fn from(data: AssemblyData) -> Self {
        let mut assembly = Self {
            name: data.name,
            links: data.links,
            joints: data.joints,
            joint_points: data.joint_points,
            children: data.children,
            parent: data.parent,
            link_name_index: HashMap::new(),
            joint_name_index: HashMap::new(),
            cache: RefCell::new(TreeCache::default()),
        };
        assembly.rebuild_indices();
        assembly.update_world_transforms();
        assembly
    }
}

impl<'de> Deserialize<'de> for Assembly {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let data = AssemblyData::deserialize(deserializer)?;
        Ok(Assembly::from(data))
    }
}

impl Default for Assembly {
    fn default() -> Self {
        Self::new("robot")
    }
}

impl Assembly {
    /// Create a new empty assembly
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            links: HashMap::new(),
            joints: HashMap::new(),
            joint_points: HashMap::new(),
            children: HashMap::new(),
            parent: HashMap::new(),
            link_name_index: HashMap::new(),
            joint_name_index: HashMap::new(),
            cache: RefCell::new(TreeCache::default()),
        }
    }

    /// Get all root links (links without parents)
    pub fn get_root_links(&self) -> Vec<Uuid> {
        self.ensure_cache_valid();
        self.cache.borrow().roots.clone()
    }

    /// Rebuild name indices (call after deserialization)
    pub fn rebuild_indices(&mut self) {
        self.link_name_index.clear();
        self.joint_name_index.clear();
        for (id, link) in &self.links {
            self.link_name_index.insert(link.name.clone(), *id);
        }
        for (id, joint) in &self.joints {
            self.joint_name_index.insert(joint.name.clone(), *id);
        }
        self.invalidate_cache();
    }

    /// Invalidate the tree cache (call after any structural change)
    pub(crate) fn invalidate_cache(&self) {
        self.cache.borrow_mut().invalidate();
    }

    /// Ensure the cache is valid, rebuilding if necessary
    pub(crate) fn ensure_cache_valid(&self) {
        let mut cache = self.cache.borrow_mut();
        if !cache.valid {
            cache.rebuild(&self.links, &self.parent, &self.children);
        }
    }

    /// Add a link to the assembly (does not automatically set as root)
    pub fn add_link(&mut self, link: Link) -> Uuid {
        let id = link.id;
        self.link_name_index.insert(link.name.clone(), id);
        self.links.insert(id, link);
        self.invalidate_cache();
        id
    }

    /// Remove a link and all its children
    pub fn remove_link(&mut self, id: Uuid) -> Result<(), AssemblyError> {
        if !self.links.contains_key(&id) {
            return Err(AssemblyError::LinkNotFound(id));
        }

        // Collect all descendants
        let mut to_remove = vec![id];
        let mut i = 0;
        while i < to_remove.len() {
            let link_id = to_remove[i];
            if let Some(children) = self.children.get(&link_id) {
                for (_, child_id) in children {
                    to_remove.push(*child_id);
                }
            }
            i += 1;
        }

        // Collect part IDs for joint point cleanup
        let part_ids: Vec<Uuid> = to_remove
            .iter()
            .filter_map(|link_id| self.links.get(link_id).and_then(|l| l.part_id))
            .collect();

        // Remove all collected links and their joints
        for link_id in &to_remove {
            if let Some(link) = self.links.remove(link_id) {
                self.link_name_index.remove(&link.name);
            }
            self.children.remove(link_id);
            if let Some((joint_id, _)) = self.parent.remove(link_id)
                && let Some(joint) = self.joints.remove(&joint_id)
            {
                self.joint_name_index.remove(&joint.name);
            }
        }

        // Remove associated joint points
        self.joint_points
            .retain(|_, jp| !part_ids.contains(&jp.part_id));

        // Clean up children references
        for children in self.children.values_mut() {
            children.retain(|(_, child_id)| !to_remove.contains(child_id));
        }

        self.invalidate_cache();
        Ok(())
    }

    /// Connect two links with a joint
    pub fn connect(
        &mut self,
        parent_id: Uuid,
        child_id: Uuid,
        joint: Joint,
    ) -> Result<Uuid, AssemblyError> {
        // Validate links exist
        if !self.links.contains_key(&parent_id) {
            return Err(AssemblyError::LinkNotFound(parent_id));
        }
        if !self.links.contains_key(&child_id) {
            return Err(AssemblyError::LinkNotFound(child_id));
        }

        // Check for cycles
        if self.would_create_cycle(parent_id, child_id) {
            return Err(AssemblyError::WouldCreateCycle);
        }

        // Check if child already has a parent
        if self.parent.contains_key(&child_id) {
            return Err(AssemblyError::AlreadyHasParent(child_id));
        }

        let joint_id = joint.id;

        // Add joint and update name index
        self.joint_name_index.insert(joint.name.clone(), joint_id);
        self.joints.insert(joint_id, joint);

        // Update mappings
        self.children
            .entry(parent_id)
            .or_default()
            .push((joint_id, child_id));
        self.parent.insert(child_id, (joint_id, parent_id));

        self.invalidate_cache();
        Ok(joint_id)
    }

    /// Disconnect a link from its parent
    pub fn disconnect(&mut self, child_id: Uuid) -> Result<Joint, AssemblyError> {
        let (joint_id, parent_id) = self
            .parent
            .remove(&child_id)
            .ok_or(AssemblyError::NoParent(child_id))?;

        // Remove from children
        if let Some(children) = self.children.get_mut(&parent_id) {
            children.retain(|(_, cid)| *cid != child_id);
        }

        // Remove joint and update name index
        let joint = self
            .joints
            .remove(&joint_id)
            .ok_or(AssemblyError::JointNotFound(joint_id))?;
        self.joint_name_index.remove(&joint.name);

        self.invalidate_cache();
        Ok(joint)
    }

    /// Check if connecting parent to child would create a cycle
    fn would_create_cycle(&self, parent_id: Uuid, child_id: Uuid) -> bool {
        // Check if child is an ancestor of parent
        let mut current = Some(parent_id);
        while let Some(id) = current {
            if id == child_id {
                return true;
            }
            current = self.parent.get(&id).map(|(_, p)| *p);
        }
        false
    }

    /// Get the world transform of a link
    pub fn get_world_transform(&self, link_id: Uuid) -> Mat4 {
        let mut transform = Mat4::IDENTITY;
        let mut current = Some(link_id);

        // Build transform chain from root to link
        let mut chain = Vec::new();
        while let Some(id) = current {
            chain.push(id);
            current = self.parent.get(&id).map(|(_, p)| *p);
        }

        // Apply transforms from root to link
        for id in chain.into_iter().rev() {
            if let Some((joint_id, _)) = self.parent.get(&id)
                && let Some(joint) = self.joints.get(joint_id)
            {
                transform *= joint.origin.to_mat4();
            }
        }

        transform
    }

    /// Update all world transforms
    pub fn update_world_transforms(&mut self) {
        let roots = self.get_root_links();
        for root_id in roots {
            self.update_transform_recursive(root_id, Mat4::IDENTITY);
        }
    }

    fn update_transform_recursive(&mut self, link_id: Uuid, parent_transform: Mat4) {
        let transform = if let Some((joint_id, _)) = self.parent.get(&link_id) {
            if let Some(joint) = self.joints.get(joint_id) {
                parent_transform * joint.origin.to_mat4()
            } else {
                parent_transform
            }
        } else {
            parent_transform
        };

        if let Some(link) = self.links.get_mut(&link_id) {
            link.world_transform = transform;
        }

        // Get children IDs first to avoid borrow issues
        let children: Vec<Uuid> = self
            .children
            .get(&link_id)
            .map(|c| c.iter().map(|(_, child_id)| *child_id).collect())
            .unwrap_or_default();

        for child_id in children {
            self.update_transform_recursive(child_id, transform);
        }
    }

    /// Update all world transforms with joint positions applied
    pub fn update_world_transforms_with_positions(&mut self, joint_positions: &HashMap<Uuid, f32>) {
        let roots = self.get_root_links();
        for root_id in roots {
            self.update_transform_recursive_with_positions(
                root_id,
                Mat4::IDENTITY,
                joint_positions,
            );
        }
    }

    fn update_transform_recursive_with_positions(
        &mut self,
        link_id: Uuid,
        parent_transform: Mat4,
        joint_positions: &HashMap<Uuid, f32>,
    ) {
        let transform = if let Some((joint_id, _)) = self.parent.get(&link_id) {
            if let Some(joint) = self.joints.get(joint_id) {
                // Get joint position (defaults to 0)
                let position = joint_positions.get(joint_id).copied().unwrap_or(0.0);
                // Compute joint transform with position
                let joint_transform =
                    Self::compute_joint_transform(&joint.joint_type, joint.axis, position);
                parent_transform * joint.origin.to_mat4() * joint_transform
            } else {
                parent_transform
            }
        } else {
            parent_transform
        };

        if let Some(link) = self.links.get_mut(&link_id) {
            link.world_transform = transform;
        }

        // Get children IDs first to avoid borrow issues
        let children: Vec<Uuid> = self
            .children
            .get(&link_id)
            .map(|c| c.iter().map(|(_, child_id)| *child_id).collect())
            .unwrap_or_default();

        for child_id in children {
            self.update_transform_recursive_with_positions(child_id, transform, joint_positions);
        }
    }

    /// Compute the transform for a joint at a given position
    pub fn compute_joint_transform(
        joint_type: &JointType,
        axis: glam::Vec3,
        position: f32,
    ) -> Mat4 {
        match joint_type {
            JointType::Revolute | JointType::Continuous => {
                // Rotation around the joint axis
                let rotation = Quat::from_axis_angle(axis, position);
                Mat4::from_quat(rotation)
            }
            JointType::Prismatic => {
                // Translation along the joint axis
                let translation = axis * position;
                Mat4::from_translation(translation)
            }
            JointType::Fixed | JointType::Floating | JointType::Planar => {
                // No transform for fixed joints, floating/planar would need more DOFs
                Mat4::IDENTITY
            }
        }
    }

    /// Validate the assembly
    pub fn validate(&self) -> Result<(), Vec<AssemblyError>> {
        let mut errors = Vec::new();

        // All links are reachable from their respective roots (no orphans possible with multiple roots)

        // Check joint references
        for joint in self.joints.values() {
            if !self.links.contains_key(&joint.parent_link) {
                errors.push(AssemblyError::InvalidJointReference(
                    joint.id,
                    joint.parent_link,
                ));
            }
            if !self.links.contains_key(&joint.child_link) {
                errors.push(AssemblyError::InvalidJointReference(
                    joint.id,
                    joint.child_link,
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get all links in depth-first order from all roots
    pub fn links_depth_first(&self) -> Vec<Uuid> {
        let mut result = Vec::new();
        for root_id in self.get_root_links() {
            self.collect_depth_first(root_id, &mut result);
        }
        result
    }

    fn collect_depth_first(&self, link_id: Uuid, result: &mut Vec<Uuid>) {
        result.push(link_id);
        if let Some(children) = self.children.get(&link_id) {
            for (_, child_id) in children {
                self.collect_depth_first(*child_id, result);
            }
        }
    }

    // ============== Joint Point Management ==============

    /// Add a joint point to the assembly
    pub fn add_joint_point(&mut self, joint_point: JointPoint) -> Uuid {
        let id = joint_point.id;
        self.joint_points.insert(id, joint_point);
        id
    }

    /// Remove a joint point by ID
    pub fn remove_joint_point(&mut self, id: Uuid) -> Option<JointPoint> {
        self.joint_points.remove(&id)
    }

    /// Get a joint point by ID
    pub fn get_joint_point(&self, id: Uuid) -> Option<&JointPoint> {
        self.joint_points.get(&id)
    }

    /// Get a mutable joint point by ID
    pub fn get_joint_point_mut(&mut self, id: Uuid) -> Option<&mut JointPoint> {
        self.joint_points.get_mut(&id)
    }

    /// Get all joint points for a specific part
    pub fn get_joint_points_for_part(&self, part_id: Uuid) -> Vec<&JointPoint> {
        self.joint_points
            .values()
            .filter(|jp| jp.part_id == part_id)
            .collect()
    }

    /// Count total number of joint points
    pub fn joint_point_count(&self) -> usize {
        self.joint_points.len()
    }
}

/// Assembly-related errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum AssemblyError {
    #[error("Link not found: {0}")]
    LinkNotFound(Uuid),
    #[error("Joint not found: {0}")]
    JointNotFound(Uuid),
    #[error("Joint point not found: {0}")]
    JointPointNotFound(Uuid),
    #[error("Part not found: {0}")]
    PartNotFound(Uuid),
    #[error("Connection would create a cycle")]
    WouldCreateCycle,
    #[error("Link already has a parent: {0}")]
    AlreadyHasParent(Uuid),
    #[error("Link has no parent: {0}")]
    NoParent(Uuid),
    #[error("No root link defined")]
    NoRoot,
    #[error("Orphaned link: {0}")]
    OrphanedLink(Uuid),
    #[error("Invalid joint reference: joint {0} references non-existent link {1}")]
    InvalidJointReference(Uuid, Uuid),
}
