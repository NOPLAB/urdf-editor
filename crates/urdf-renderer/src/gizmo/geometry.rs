//! Gizmo geometry generation
//!
//! This module contains the geometry generation functions for the transform gizmo.
//! It generates arrow meshes for the X, Y, and Z axes.

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};

use crate::constants::gizmo::{self, colors};

/// Gizmo vertex data
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct GizmoVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub axis_id: u32,
}

/// Generate translation gizmo geometry (3 arrows along X, Y, Z axes).
///
/// Returns (vertices, indices) for indexed triangle rendering.
pub fn generate_translation_gizmo() -> (Vec<GizmoVertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let arrow_length = gizmo::ARROW_LENGTH;
    let shaft_radius = gizmo::SHAFT_RADIUS;
    let head_radius = gizmo::HEAD_RADIUS;
    let head_length = gizmo::HEAD_LENGTH;
    let segments = gizmo::SEGMENTS;

    // Generate arrow for each axis
    for (axis_id, (color, direction)) in [
        (colors::X_AXIS, Vec3::X), // X = Red
        (colors::Y_AXIS, Vec3::Y), // Y = Green
        (colors::Z_AXIS, Vec3::Z), // Z = Blue
    ]
    .iter()
    .enumerate()
    {
        let base_index = vertices.len() as u32;

        // Create rotation to align arrow with axis direction
        let rotation = if *direction == Vec3::X {
            Mat4::from_rotation_z(-std::f32::consts::FRAC_PI_2)
        } else if *direction == Vec3::Z {
            Mat4::from_rotation_x(std::f32::consts::FRAC_PI_2)
        } else {
            Mat4::IDENTITY
        };

        // Shaft cylinder
        let shaft_end = arrow_length - head_length;
        for i in 0..=segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = angle.cos() * shaft_radius;
            let z = angle.sin() * shaft_radius;

            // Bottom of shaft
            let pos_bottom = rotation.transform_point3(Vec3::new(x, 0.0, z));
            vertices.push(GizmoVertex {
                position: pos_bottom.into(),
                color: *color,
                axis_id: axis_id as u32,
            });

            // Top of shaft
            let pos_top = rotation.transform_point3(Vec3::new(x, shaft_end, z));
            vertices.push(GizmoVertex {
                position: pos_top.into(),
                color: *color,
                axis_id: axis_id as u32,
            });
        }

        // Shaft indices
        for i in 0..segments {
            let i0 = base_index + i * 2;
            let i1 = base_index + i * 2 + 1;
            let i2 = base_index + (i + 1) * 2;
            let i3 = base_index + (i + 1) * 2 + 1;
            indices.extend_from_slice(&[i0, i2, i1, i1, i2, i3]);
        }

        // Cone head
        let cone_base_index = vertices.len() as u32;

        // Cone tip
        let tip_pos = rotation.transform_point3(Vec3::new(0.0, arrow_length, 0.0));
        vertices.push(GizmoVertex {
            position: tip_pos.into(),
            color: *color,
            axis_id: axis_id as u32,
        });

        // Cone base vertices
        for i in 0..=segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = angle.cos() * head_radius;
            let z = angle.sin() * head_radius;
            let pos = rotation.transform_point3(Vec3::new(x, shaft_end, z));
            vertices.push(GizmoVertex {
                position: pos.into(),
                color: *color,
                axis_id: axis_id as u32,
            });
        }

        // Cone side indices
        let tip_index = cone_base_index;
        for i in 0..segments {
            let i0 = cone_base_index + 1 + i;
            let i1 = cone_base_index + 1 + (i + 1);
            indices.extend_from_slice(&[tip_index, i1, i0]);
        }

        // Cone base cap (center point + ring)
        let center_index = vertices.len() as u32;
        let center_pos = rotation.transform_point3(Vec3::new(0.0, shaft_end, 0.0));
        vertices.push(GizmoVertex {
            position: center_pos.into(),
            color: *color,
            axis_id: axis_id as u32,
        });

        for i in 0..segments {
            let i0 = cone_base_index + 1 + i;
            let i1 = cone_base_index + 1 + (i + 1);
            indices.extend_from_slice(&[center_index, i0, i1]);
        }
    }

    (vertices, indices)
}
