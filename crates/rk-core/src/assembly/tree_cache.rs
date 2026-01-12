//! Cached tree structure for efficient traversal

use std::collections::HashMap;
use uuid::Uuid;

use super::types::Link;

/// Cached tree structure for efficient traversal (computed on demand)
#[derive(Debug, Clone, Default)]
pub(super) struct TreeCache {
    /// Depth of each link (root = 0)
    pub depths: HashMap<Uuid, usize>,
    /// Pre-computed chain to root for each link
    pub ancestors: HashMap<Uuid, Vec<Uuid>>,
    /// Pre-computed descendants for each link
    pub descendants: HashMap<Uuid, Vec<Uuid>>,
    /// Root link IDs
    pub roots: Vec<Uuid>,
    /// Whether cache is valid
    pub valid: bool,
}

impl TreeCache {
    pub fn invalidate(&mut self) {
        self.valid = false;
    }

    pub fn rebuild(
        &mut self,
        links: &HashMap<Uuid, Link>,
        parent: &HashMap<Uuid, (Uuid, Uuid)>,
        children: &HashMap<Uuid, Vec<(Uuid, Uuid)>>,
    ) {
        self.depths.clear();
        self.ancestors.clear();
        self.descendants.clear();
        self.roots.clear();

        // Find all root links
        self.roots = links
            .keys()
            .filter(|id| !parent.contains_key(id))
            .copied()
            .collect();

        // Build depths and ancestors using DFS from each root
        for &root_id in &self.roots.clone() {
            self.depths.insert(root_id, 0);
            self.ancestors.insert(root_id, vec![root_id]);
            Self::build_recursive(
                &mut self.depths,
                &mut self.ancestors,
                children,
                root_id,
                0,
                vec![root_id],
            );
        }

        // Build descendants (reverse of ancestors)
        for (link_id, ancestor_chain) in &self.ancestors.clone() {
            for &ancestor_id in ancestor_chain {
                if ancestor_id != *link_id {
                    self.descendants
                        .entry(ancestor_id)
                        .or_default()
                        .push(*link_id);
                }
            }
        }

        self.valid = true;
    }

    fn build_recursive(
        depths: &mut HashMap<Uuid, usize>,
        ancestors: &mut HashMap<Uuid, Vec<Uuid>>,
        children: &HashMap<Uuid, Vec<(Uuid, Uuid)>>,
        link_id: Uuid,
        depth: usize,
        ancestor_chain: Vec<Uuid>,
    ) {
        if let Some(child_list) = children.get(&link_id) {
            for (_, child_id) in child_list {
                let child_depth = depth + 1;
                let mut child_ancestors = ancestor_chain.clone();
                child_ancestors.push(*child_id);

                depths.insert(*child_id, child_depth);
                ancestors.insert(*child_id, child_ancestors.clone());
                Self::build_recursive(
                    depths,
                    ancestors,
                    children,
                    *child_id,
                    child_depth,
                    child_ancestors,
                );
            }
        }
    }
}
