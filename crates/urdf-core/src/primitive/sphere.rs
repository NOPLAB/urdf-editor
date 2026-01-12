//! Sphere mesh generation (UV sphere)

use std::f32::consts::PI;

use super::MeshData;

/// Generate a UV sphere mesh
///
/// # Arguments
/// * `radius` - Sphere radius
///
/// # Returns
/// (vertices, normals, indices)
pub fn generate_sphere_mesh(radius: f32) -> MeshData {
    use crate::constants::{SPHERE_LAT_SEGMENTS, SPHERE_LON_SEGMENTS};
    generate_sphere_mesh_with_segments(radius, SPHERE_LAT_SEGMENTS, SPHERE_LON_SEGMENTS)
}

/// Generate a UV sphere mesh with custom resolution
///
/// # Arguments
/// * `radius` - Sphere radius
/// * `lat_segments` - Number of latitude bands (default: 16)
/// * `lon_segments` - Number of longitude segments (default: 32)
pub fn generate_sphere_mesh_with_segments(
    radius: f32,
    lat_segments: u32,
    lon_segments: u32,
) -> MeshData {
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    // Generate vertices
    for lat in 0..=lat_segments {
        let theta = (lat as f32 / lat_segments as f32) * PI; // 0 to PI
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        for lon in 0..=lon_segments {
            let phi = (lon as f32 / lon_segments as f32) * 2.0 * PI; // 0 to 2*PI
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            let x = sin_theta * cos_phi;
            let y = sin_theta * sin_phi;
            let z = cos_theta;

            vertices.push([radius * x, radius * y, radius * z]);
            normals.push([x, y, z]);
        }
    }

    // Generate indices
    for lat in 0..lat_segments {
        for lon in 0..lon_segments {
            let current = lat * (lon_segments + 1) + lon;
            let next = current + lon_segments + 1;

            // Triangle 1
            indices.push(current);
            indices.push(next);
            indices.push(current + 1);

            // Triangle 2
            indices.push(current + 1);
            indices.push(next);
            indices.push(next + 1);
        }
    }

    (vertices, normals, indices)
}
