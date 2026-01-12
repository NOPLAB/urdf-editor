//! Cylinder mesh generation (with end caps)

use std::f32::consts::PI;

use super::MeshData;

/// Generate a cylinder mesh along the Z axis
///
/// # Arguments
/// * `radius` - Cylinder radius
/// * `length` - Cylinder length (height along Z)
///
/// # Returns
/// (vertices, normals, indices)
pub fn generate_cylinder_mesh(radius: f32, length: f32) -> MeshData {
    use crate::constants::CYLINDER_SEGMENTS;
    generate_cylinder_mesh_with_segments(radius, length, CYLINDER_SEGMENTS)
}

/// Generate a cylinder mesh with custom segment count
pub fn generate_cylinder_mesh_with_segments(radius: f32, length: f32, segments: u32) -> MeshData {
    let half_length = length / 2.0;
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    // Generate side vertices
    for i in 0..=segments {
        let theta = (i as f32 / segments as f32) * 2.0 * PI;
        let x = radius * theta.cos();
        let y = radius * theta.sin();
        let nx = theta.cos();
        let ny = theta.sin();

        // Bottom vertex
        vertices.push([x, y, -half_length]);
        normals.push([nx, ny, 0.0]);

        // Top vertex
        vertices.push([x, y, half_length]);
        normals.push([nx, ny, 0.0]);
    }

    // Side triangles
    for i in 0..segments {
        let base = i * 2;
        // Triangle 1
        indices.push(base);
        indices.push(base + 2);
        indices.push(base + 1);
        // Triangle 2
        indices.push(base + 1);
        indices.push(base + 2);
        indices.push(base + 3);
    }

    // Top cap center
    let top_center_idx = vertices.len() as u32;
    vertices.push([0.0, 0.0, half_length]);
    normals.push([0.0, 0.0, 1.0]);

    // Top cap rim vertices
    let top_rim_start = vertices.len() as u32;
    for i in 0..=segments {
        let theta = (i as f32 / segments as f32) * 2.0 * PI;
        let x = radius * theta.cos();
        let y = radius * theta.sin();
        vertices.push([x, y, half_length]);
        normals.push([0.0, 0.0, 1.0]);
    }

    // Top cap triangles
    for i in 0..segments {
        indices.push(top_center_idx);
        indices.push(top_rim_start + i);
        indices.push(top_rim_start + i + 1);
    }

    // Bottom cap center
    let bottom_center_idx = vertices.len() as u32;
    vertices.push([0.0, 0.0, -half_length]);
    normals.push([0.0, 0.0, -1.0]);

    // Bottom cap rim vertices
    let bottom_rim_start = vertices.len() as u32;
    for i in 0..=segments {
        let theta = (i as f32 / segments as f32) * 2.0 * PI;
        let x = radius * theta.cos();
        let y = radius * theta.sin();
        vertices.push([x, y, -half_length]);
        normals.push([0.0, 0.0, -1.0]);
    }

    // Bottom cap triangles (reversed winding)
    for i in 0..segments {
        indices.push(bottom_center_idx);
        indices.push(bottom_rim_start + i + 1);
        indices.push(bottom_rim_start + i);
    }

    (vertices, normals, indices)
}
