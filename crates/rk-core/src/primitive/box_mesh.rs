//! Box (rectangular prism) mesh generation

use super::MeshData;

/// Generate a box mesh with the given dimensions
///
/// # Arguments
/// * `size` - [width (x), depth (y), height (z)]
///
/// # Returns
/// (vertices, normals, indices) - 24 vertices (4 per face), 12 triangles
pub fn generate_box_mesh(size: [f32; 3]) -> MeshData {
    let hx = size[0] / 2.0;
    let hy = size[1] / 2.0;
    let hz = size[2] / 2.0;

    // 6 faces, 4 vertices each (for proper normals)
    let mut vertices = Vec::with_capacity(24);
    let mut normals = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);

    // Helper to add a face
    let mut add_face = |corners: [[f32; 3]; 4], normal: [f32; 3]| {
        let base = vertices.len() as u32;
        for corner in corners {
            vertices.push(corner);
            normals.push(normal);
        }
        // Two triangles per face
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    };

    // +X face
    add_face(
        [[hx, -hy, -hz], [hx, hy, -hz], [hx, hy, hz], [hx, -hy, hz]],
        [1.0, 0.0, 0.0],
    );

    // -X face
    add_face(
        [
            [-hx, hy, -hz],
            [-hx, -hy, -hz],
            [-hx, -hy, hz],
            [-hx, hy, hz],
        ],
        [-1.0, 0.0, 0.0],
    );

    // +Y face
    add_face(
        [[hx, hy, -hz], [-hx, hy, -hz], [-hx, hy, hz], [hx, hy, hz]],
        [0.0, 1.0, 0.0],
    );

    // -Y face
    add_face(
        [
            [-hx, -hy, -hz],
            [hx, -hy, -hz],
            [hx, -hy, hz],
            [-hx, -hy, hz],
        ],
        [0.0, -1.0, 0.0],
    );

    // +Z face (top)
    add_face(
        [[-hx, -hy, hz], [hx, -hy, hz], [hx, hy, hz], [-hx, hy, hz]],
        [0.0, 0.0, 1.0],
    );

    // -Z face (bottom)
    add_face(
        [
            [-hx, hy, -hz],
            [hx, hy, -hz],
            [hx, -hy, -hz],
            [-hx, -hy, -hz],
        ],
        [0.0, 0.0, -1.0],
    );

    (vertices, normals, indices)
}
