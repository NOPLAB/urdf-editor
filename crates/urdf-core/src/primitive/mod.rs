//! Primitive mesh generation for URDF geometry types
//!
//! Generates vertices, normals, and indices for basic shapes:
//! - Box (rectangular prism)
//! - Cylinder (with end caps)
//! - Sphere (UV sphere)

mod box_mesh;
mod cylinder;
mod sphere;

pub use box_mesh::generate_box_mesh;
pub use cylinder::{generate_cylinder_mesh, generate_cylinder_mesh_with_segments};
pub use sphere::{generate_sphere_mesh, generate_sphere_mesh_with_segments};

/// Mesh data: vertices, normals, and triangle indices
pub type MeshData = (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_mesh() {
        let (vertices, normals, indices) = generate_box_mesh([1.0, 1.0, 1.0]);
        assert_eq!(vertices.len(), 24); // 6 faces * 4 vertices
        assert_eq!(normals.len(), 24);
        assert_eq!(indices.len(), 36); // 6 faces * 2 triangles * 3 indices
    }

    #[test]
    fn test_cylinder_mesh() {
        let (vertices, normals, indices) = generate_cylinder_mesh(0.5, 1.0);
        assert!(!vertices.is_empty());
        assert_eq!(vertices.len(), normals.len());
        assert!(!indices.is_empty());
        assert!(indices.len() % 3 == 0); // Valid triangles
    }

    #[test]
    fn test_sphere_mesh() {
        let (vertices, normals, indices) = generate_sphere_mesh(1.0);
        assert!(!vertices.is_empty());
        assert_eq!(vertices.len(), normals.len());
        assert!(!indices.is_empty());
        assert!(indices.len() % 3 == 0);
    }

    #[test]
    fn test_box_dimensions() {
        let (vertices, _, _) = generate_box_mesh([2.0, 4.0, 6.0]);
        let mut min = [f32::MAX; 3];
        let mut max = [f32::MIN; 3];
        for v in &vertices {
            for i in 0..3 {
                min[i] = min[i].min(v[i]);
                max[i] = max[i].max(v[i]);
            }
        }
        assert!((max[0] - min[0] - 2.0).abs() < 0.001);
        assert!((max[1] - min[1] - 4.0).abs() < 0.001);
        assert!((max[2] - min[2] - 6.0).abs() < 0.001);
    }
}
