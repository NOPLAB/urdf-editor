//! Transform component - position and rotation editing

use egui::Ui;
use glam::{EulerRot, Mat4, Quat, Vec3};

use crate::panels::properties::helpers::{rotation_row, vector3_row};
use crate::panels::properties::{PropertyComponent, PropertyContext};

/// Transform component (position, rotation)
pub struct TransformComponent;

impl TransformComponent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TransformComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyComponent for TransformComponent {
    fn name(&self) -> &str {
        "Transform"
    }

    fn ui(&mut self, ui: &mut Ui, ctx: &mut PropertyContext) -> bool {
        let part = &mut ctx.part;
        let parent_transform = ctx.parent_world_transform;

        // Extract position and rotation from the transform matrix (world coordinates)
        let (scale, rotation, translation) = part.origin_transform.to_scale_rotation_translation();
        let euler = rotation.to_euler(EulerRot::XYZ);

        let mut pos = [translation.x, translation.y, translation.z];
        let mut rot_deg = [
            euler.0.to_degrees(),
            euler.1.to_degrees(),
            euler.2.to_degrees(),
        ];

        // World Position (editable)
        let pos_changed = vector3_row(ui, "Position", &mut pos, 0.01);

        // Local Position (relative to parent, editable) - only show if parent exists
        let mut local_pos_changed = false;
        let mut local_pos = [0.0f32; 3];
        let mut local_rot_deg = [0.0f32; 3];
        let mut local_rot_changed = false;

        if let Some(parent) = parent_transform {
            // Calculate local transform: parent_inverse * world_transform
            let parent_inverse = parent.inverse();
            let local = parent_inverse * part.origin_transform;
            let (_, local_rotation, local_translation) = local.to_scale_rotation_translation();
            let local_euler = local_rotation.to_euler(EulerRot::XYZ);

            local_pos = [
                local_translation.x,
                local_translation.y,
                local_translation.z,
            ];
            local_rot_deg = [
                local_euler.0.to_degrees(),
                local_euler.1.to_degrees(),
                local_euler.2.to_degrees(),
            ];

            local_pos_changed = vector3_row(ui, "Local Position", &mut local_pos, 0.01);
        }

        // Rotation
        let rot_changed = rotation_row(ui, "Rotation", &mut rot_deg, 1.0);

        // Local Rotation (relative to parent, editable) - only show if parent exists
        if parent_transform.is_some() {
            local_rot_changed = rotation_row(ui, "Local Rotation", &mut local_rot_deg, 1.0);
        }

        // Update transform if changed
        if local_pos_changed || local_rot_changed {
            // Local transform was edited - compute new world transform
            let parent = parent_transform.unwrap();
            let new_local_rotation = Quat::from_euler(
                EulerRot::XYZ,
                local_rot_deg[0].to_radians(),
                local_rot_deg[1].to_radians(),
                local_rot_deg[2].to_radians(),
            );
            let new_local_translation = Vec3::new(local_pos[0], local_pos[1], local_pos[2]);
            let new_local_transform = Mat4::from_scale_rotation_translation(
                scale,
                new_local_rotation,
                new_local_translation,
            );
            // world = parent * local
            part.origin_transform = parent * new_local_transform;
            true
        } else if pos_changed || rot_changed {
            // World transform was edited
            let new_rotation = Quat::from_euler(
                EulerRot::XYZ,
                rot_deg[0].to_radians(),
                rot_deg[1].to_radians(),
                rot_deg[2].to_radians(),
            );
            let new_translation = Vec3::new(pos[0], pos[1], pos[2]);
            part.origin_transform =
                Mat4::from_scale_rotation_translation(scale, new_rotation, new_translation);
            true
        } else {
            false
        }
    }
}
