//! Plane selector sub-renderer for reference plane visualization.
//!
//! Renders three semi-transparent planes (XY, XZ, YZ) at the origin
//! for selecting a sketch plane. Supports hover highlighting.

use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use wgpu::util::DeviceExt;

use crate::pipeline::{PipelineConfig, create_camera_bind_group};

/// Vertex for plane selector rendering.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct PlaneSelectorVertex {
    /// Position in world space.
    pub position: [f32; 3],
    /// Normal vector.
    pub normal: [f32; 3],
    /// Vertex color (RGBA).
    pub color: [f32; 4],
    /// Plane ID (1 = XY, 2 = XZ, 3 = YZ).
    pub plane_id: u32,
}

impl PlaneSelectorVertex {
    /// Create a new vertex.
    pub fn new(position: Vec3, normal: Vec3, color: [f32; 4], plane_id: u32) -> Self {
        Self {
            position: position.to_array(),
            normal: normal.to_array(),
            color,
            plane_id,
        }
    }

    /// Returns the vertex buffer layout.
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // normal
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // color
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // plane_id
                wgpu::VertexAttribute {
                    offset: 40,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}

/// Uniform data for plane selector.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct PlaneUniform {
    /// Which plane is highlighted (0 = none, 1 = XY, 2 = XZ, 3 = YZ).
    highlighted_plane: u32,
    /// Plane size.
    plane_size: f32,
    /// Padding.
    _padding: [f32; 2],
}

/// Reference plane IDs for shader communication.
pub mod plane_ids {
    /// No plane highlighted.
    pub const NONE: u32 = 0;
    /// XY plane (Top view).
    pub const XY: u32 = 1;
    /// XZ plane (Front view).
    pub const XZ: u32 = 2;
    /// YZ plane (Side view).
    pub const YZ: u32 = 3;
}

/// Plane selector renderer for reference plane visualization.
pub struct PlaneSelectorRenderer {
    pipeline: wgpu::RenderPipeline,
    camera_bind_group: wgpu::BindGroup,
    #[allow(dead_code)]
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,

    /// Currently highlighted plane ID.
    highlighted_plane: u32,
    /// Size of the planes.
    plane_size: f32,
    /// Whether the renderer is visible.
    visible: bool,
}

impl PlaneSelectorRenderer {
    /// Default plane size.
    pub const DEFAULT_PLANE_SIZE: f32 = 2.0;

    /// Plane colors (RGBA).
    const XY_COLOR: [f32; 4] = [0.3, 0.5, 0.9, 0.25]; // Blue
    const XZ_COLOR: [f32; 4] = [0.3, 0.9, 0.5, 0.25]; // Green
    const YZ_COLOR: [f32; 4] = [0.9, 0.5, 0.3, 0.25]; // Red

    /// Creates a new plane selector renderer.
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
        camera_buffer: &wgpu::Buffer,
    ) -> Self {
        let camera_bind_group = create_camera_bind_group(
            device,
            camera_bind_group_layout,
            camera_buffer,
            "PlaneSelector",
        );

        // Create uniform bind group layout
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("PlaneSelector Uniform Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        // Create uniform buffer
        let uniform = PlaneUniform {
            highlighted_plane: plane_ids::NONE,
            plane_size: Self::DEFAULT_PLANE_SIZE,
            _padding: [0.0; 2],
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("PlaneSelector Uniform Buffer"),
            contents: bytemuck::bytes_of(&uniform),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create uniform bind group
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("PlaneSelector Uniform Bind Group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Create pipeline with alpha blending
        let pipeline = PipelineConfig::new(
            "PlaneSelector",
            include_str!("../shaders/plane_selector.wgsl"),
            format,
            depth_format,
            &[camera_bind_group_layout, &uniform_bind_group_layout],
        )
        .with_vertex_layouts(vec![PlaneSelectorVertex::layout()])
        .with_cull_mode(None) // Double-sided
        .build(device);

        // Generate plane vertices
        let (vertices, indices) = Self::generate_plane_geometry();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("PlaneSelector Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("PlaneSelector Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            pipeline,
            camera_bind_group,
            uniform_bind_group_layout,
            uniform_buffer,
            uniform_bind_group,
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            highlighted_plane: plane_ids::NONE,
            plane_size: Self::DEFAULT_PLANE_SIZE,
            visible: false,
        }
    }

    /// Generate vertices and indices for the three reference planes.
    fn generate_plane_geometry() -> (Vec<PlaneSelectorVertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // XY plane (Z = 0, normal = +Z)
        let xy_base = vertices.len() as u32;
        let xy_color = Self::XY_COLOR;
        let xy_normal = Vec3::Z;
        vertices.push(PlaneSelectorVertex::new(
            Vec3::new(-1.0, -1.0, 0.0),
            xy_normal,
            xy_color,
            plane_ids::XY,
        ));
        vertices.push(PlaneSelectorVertex::new(
            Vec3::new(1.0, -1.0, 0.0),
            xy_normal,
            xy_color,
            plane_ids::XY,
        ));
        vertices.push(PlaneSelectorVertex::new(
            Vec3::new(1.0, 1.0, 0.0),
            xy_normal,
            xy_color,
            plane_ids::XY,
        ));
        vertices.push(PlaneSelectorVertex::new(
            Vec3::new(-1.0, 1.0, 0.0),
            xy_normal,
            xy_color,
            plane_ids::XY,
        ));
        indices.extend_from_slice(&[
            xy_base,
            xy_base + 1,
            xy_base + 2,
            xy_base,
            xy_base + 2,
            xy_base + 3,
        ]);

        // XZ plane (Y = 0, normal = +Y)
        let xz_base = vertices.len() as u32;
        let xz_color = Self::XZ_COLOR;
        let xz_normal = Vec3::Y;
        vertices.push(PlaneSelectorVertex::new(
            Vec3::new(-1.0, 0.0, -1.0),
            xz_normal,
            xz_color,
            plane_ids::XZ,
        ));
        vertices.push(PlaneSelectorVertex::new(
            Vec3::new(1.0, 0.0, -1.0),
            xz_normal,
            xz_color,
            plane_ids::XZ,
        ));
        vertices.push(PlaneSelectorVertex::new(
            Vec3::new(1.0, 0.0, 1.0),
            xz_normal,
            xz_color,
            plane_ids::XZ,
        ));
        vertices.push(PlaneSelectorVertex::new(
            Vec3::new(-1.0, 0.0, 1.0),
            xz_normal,
            xz_color,
            plane_ids::XZ,
        ));
        indices.extend_from_slice(&[
            xz_base,
            xz_base + 1,
            xz_base + 2,
            xz_base,
            xz_base + 2,
            xz_base + 3,
        ]);

        // YZ plane (X = 0, normal = +X)
        let yz_base = vertices.len() as u32;
        let yz_color = Self::YZ_COLOR;
        let yz_normal = Vec3::X;
        vertices.push(PlaneSelectorVertex::new(
            Vec3::new(0.0, -1.0, -1.0),
            yz_normal,
            yz_color,
            plane_ids::YZ,
        ));
        vertices.push(PlaneSelectorVertex::new(
            Vec3::new(0.0, 1.0, -1.0),
            yz_normal,
            yz_color,
            plane_ids::YZ,
        ));
        vertices.push(PlaneSelectorVertex::new(
            Vec3::new(0.0, 1.0, 1.0),
            yz_normal,
            yz_color,
            plane_ids::YZ,
        ));
        vertices.push(PlaneSelectorVertex::new(
            Vec3::new(0.0, -1.0, 1.0),
            yz_normal,
            yz_color,
            plane_ids::YZ,
        ));
        indices.extend_from_slice(&[
            yz_base,
            yz_base + 1,
            yz_base + 2,
            yz_base,
            yz_base + 2,
            yz_base + 3,
        ]);

        (vertices, indices)
    }

    /// Set visibility.
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Check if visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Set the highlighted plane.
    pub fn set_highlighted(&mut self, queue: &wgpu::Queue, plane_id: u32) {
        if self.highlighted_plane != plane_id {
            self.highlighted_plane = plane_id;
            self.update_uniform(queue);
        }
    }

    /// Get the currently highlighted plane ID.
    pub fn highlighted(&self) -> u32 {
        self.highlighted_plane
    }

    /// Set the plane size.
    pub fn set_plane_size(&mut self, queue: &wgpu::Queue, size: f32) {
        if (self.plane_size - size).abs() > f32::EPSILON {
            self.plane_size = size;
            self.update_uniform(queue);
        }
    }

    /// Update the uniform buffer.
    fn update_uniform(&self, queue: &wgpu::Queue) {
        let uniform = PlaneUniform {
            highlighted_plane: self.highlighted_plane,
            plane_size: self.plane_size,
            _padding: [0.0; 2],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniform));
    }

    /// Render the plane selector.
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if !self.visible {
            return;
        }

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}
