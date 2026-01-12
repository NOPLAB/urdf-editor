//! Tree structure building for part hierarchy

use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::state::AppState;

/// Actions collected during tree rendering
pub enum TreeAction {
    Select(Uuid),
    Delete(Uuid),
    Disconnect(Uuid),
    Connect { parent: Uuid, child: Uuid },
}

/// Build tree structure from Assembly state
///
/// Returns:
/// - root_parts: Parts whose links have no parent (top-level parts in hierarchy)
/// - children_map: Map of part_id -> child part_ids
/// - parts_with_parent: Set of parts that have a parent
/// - unconnected_parts: Parts not in assembly at all
#[allow(clippy::type_complexity)]
pub fn build_tree_structure(
    state: &AppState,
) -> (
    Vec<Uuid>,
    HashMap<Uuid, Vec<Uuid>>,
    HashSet<Uuid>,
    Vec<Uuid>,
) {
    let assembly = &state.project.assembly;

    // Map link_id -> part_id (only for links with parts)
    let link_to_part: HashMap<Uuid, Uuid> = assembly
        .links
        .iter()
        .filter_map(|(link_id, link)| link.part_id.map(|pid| (*link_id, pid)))
        .collect();

    // Map part_id -> link_id
    let part_to_link: HashMap<Uuid, Uuid> = assembly
        .links
        .iter()
        .filter_map(|(link_id, link)| link.part_id.map(|pid| (pid, *link_id)))
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

    // Identify parts with parents (their links are in assembly.parent)
    let parts_with_parent: HashSet<Uuid> = assembly
        .parent
        .keys()
        .filter_map(|link_id| link_to_part.get(link_id).copied())
        .collect();

    // Root parts: parts with a link but no parent (top of their hierarchy)
    let root_parts: Vec<Uuid> = state
        .project
        .parts()
        .keys()
        .filter(|part_id| {
            if let Some(&link_id) = part_to_link.get(part_id) {
                // Part has a link - it's a root if it has no parent
                !assembly.parent.contains_key(&link_id)
            } else {
                false // No link = not a root (it's unconnected)
            }
        })
        .copied()
        .collect();

    // Unconnected parts: parts not in assembly at all (no link)
    let unconnected_parts: Vec<Uuid> = state
        .project
        .parts()
        .keys()
        .filter(|part_id| !part_to_link.contains_key(part_id))
        .copied()
        .collect();

    (
        root_parts,
        children_map,
        parts_with_parent,
        unconnected_parts,
    )
}

/// Check if connecting parent to child would be valid (no cycle)
pub fn can_connect(state: &AppState, parent_part: Uuid, child_part: Uuid) -> bool {
    if parent_part == child_part {
        return false;
    }

    let assembly = &state.project.assembly;

    // Find link IDs
    let parent_link = assembly
        .links
        .iter()
        .find(|(_, l)| l.part_id == Some(parent_part))
        .map(|(id, _)| *id);
    let child_link = assembly
        .links
        .iter()
        .find(|(_, l)| l.part_id == Some(child_part))
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
