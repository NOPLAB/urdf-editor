//! Normal calculation utilities for mesh data

/// Calculate normal for a single triangle
pub fn calculate_triangle_normal(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> [f32; 3] {
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

    let cross = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];

    let len = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
    if len > 0.0 {
        [cross[0] / len, cross[1] / len, cross[2] / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}

/// Calculate face normals from vertices and indices
pub fn calculate_face_normals(vertices: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let mut normals = Vec::new();

    for chunk in indices.chunks(3) {
        if chunk.len() != 3 {
            continue;
        }

        let v0 = vertices[chunk[0] as usize];
        let v1 = vertices[chunk[1] as usize];
        let v2 = vertices[chunk[2] as usize];

        // Calculate edge vectors
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

        // Cross product
        let cross = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];

        // Normalize
        let len = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
        let normal = if len > 0.0 {
            [cross[0] / len, cross[1] / len, cross[2] / len]
        } else {
            [0.0, 0.0, 1.0]
        };

        normals.push(normal);
    }

    normals
}
