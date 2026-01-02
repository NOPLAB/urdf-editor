//! Transform gizmo renderer

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

/// Gizmo mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GizmoMode {
    #[default]
    Translate,
    Rotate,
}

/// Which axis is being manipulated
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GizmoAxis {
    #[default]
    None,
    X,
    Y,
    Z,
}

impl GizmoAxis {
    pub fn to_index(&self) -> i32 {
        match self {
            GizmoAxis::None => -1,
            GizmoAxis::X => 0,
            GizmoAxis::Y => 1,
            GizmoAxis::Z => 2,
        }
    }

    pub fn direction(&self) -> Vec3 {
        match self {
            GizmoAxis::None => Vec3::ZERO,
            GizmoAxis::X => Vec3::X,
            GizmoAxis::Y => Vec3::Y,
            GizmoAxis::Z => Vec3::Z,
        }
    }
}

/// Gizmo instance data
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct GizmoInstance {
    pub transform: [[f32; 4]; 4],
    pub scale: f32,
    pub highlighted_axis: f32, // -1=none, 0=X, 1=Y, 2=Z
    pub _padding: [f32; 2],
}

impl Default for GizmoInstance {
    fn default() -> Self {
        Self {
            transform: Mat4::IDENTITY.to_cols_array_2d(),
            scale: 1.0,
            highlighted_axis: -1.0,
            _padding: [0.0; 2],
        }
    }
}

/// Gizmo vertex
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct GizmoVertex {
    position: [f32; 3],
    color: [f32; 4],
    axis_id: u32,
}

/// Gizmo renderer
pub struct GizmoRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    instance_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pub visible: bool,
    pub mode: GizmoMode,
    pub highlighted_axis: GizmoAxis,
    instance: GizmoInstance,
}

impl GizmoRenderer {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
        camera_buffer: &wgpu::Buffer,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Gizmo Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/gizmo.wgsl").into()),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Gizmo Camera Bind Group"),
            layout: camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Gizmo Pipeline Layout"),
            bind_group_layouts: &[camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Gizmo Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    // Vertex buffer
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<GizmoVertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float32x3,
                            },
                            wgpu::VertexAttribute {
                                offset: 12,
                                shader_location: 1,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            wgpu::VertexAttribute {
                                offset: 28,
                                shader_location: 2,
                                format: wgpu::VertexFormat::Uint32,
                            },
                        ],
                    },
                    // Instance buffer
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<GizmoInstance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 3,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            wgpu::VertexAttribute {
                                offset: 16,
                                shader_location: 4,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            wgpu::VertexAttribute {
                                offset: 32,
                                shader_location: 5,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            wgpu::VertexAttribute {
                                offset: 48,
                                shader_location: 6,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            wgpu::VertexAttribute {
                                offset: 64,
                                shader_location: 7,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                        ],
                    },
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: false, // Gizmo always on top
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let (vertices, indices) = generate_translation_gizmo();
        let index_count = indices.len() as u32;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Gizmo Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Gizmo Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Gizmo Instance Buffer"),
            contents: bytemuck::cast_slice(&[GizmoInstance::default()]),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            index_count,
            instance_buffer,
            bind_group,
            visible: false,
            mode: GizmoMode::Translate,
            highlighted_axis: GizmoAxis::None,
            instance: GizmoInstance::default(),
        }
    }

    /// Set gizmo position and scale
    pub fn set_transform(&mut self, queue: &wgpu::Queue, position: Vec3, scale: f32) {
        self.instance.transform = Mat4::from_translation(position).to_cols_array_2d();
        self.instance.scale = scale;
        self.update_buffer(queue);
    }

    /// Set highlighted axis
    pub fn set_highlighted(&mut self, queue: &wgpu::Queue, axis: GizmoAxis) {
        self.highlighted_axis = axis;
        self.instance.highlighted_axis = axis.to_index() as f32;
        self.update_buffer(queue);
    }

    fn update_buffer(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&[self.instance]));
    }

    /// Show the gizmo at the given position
    pub fn show(&mut self, queue: &wgpu::Queue, position: Vec3, scale: f32) {
        self.visible = true;
        self.set_transform(queue, position, scale);
    }

    /// Hide the gizmo
    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if !self.visible {
            return;
        }

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }

    /// Check if a ray intersects with a gizmo axis handle
    /// Returns the closest intersecting axis
    pub fn hit_test(&self, ray_origin: Vec3, ray_dir: Vec3, gizmo_pos: Vec3, scale: f32) -> GizmoAxis {
        let handle_radius = 0.08 * scale;
        let handle_length = 1.0 * scale;

        let mut closest_axis = GizmoAxis::None;
        let mut closest_dist = f32::MAX;

        for (axis, dir) in [
            (GizmoAxis::X, Vec3::X),
            (GizmoAxis::Y, Vec3::Y),
            (GizmoAxis::Z, Vec3::Z),
        ] {
            // Test intersection with cylinder along axis
            let axis_start = gizmo_pos;
            let axis_end = gizmo_pos + dir * handle_length;

            if let Some(dist) = ray_cylinder_intersection(
                ray_origin,
                ray_dir,
                axis_start,
                axis_end,
                handle_radius,
            ) {
                if dist < closest_dist {
                    closest_dist = dist;
                    closest_axis = axis;
                }
            }
        }

        closest_axis
    }
}

/// Generate translation gizmo geometry (3 arrows)
fn generate_translation_gizmo() -> (Vec<GizmoVertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let arrow_length = 1.0f32;
    let shaft_radius = 0.02f32;
    let head_radius = 0.06f32;
    let head_length = 0.15f32;
    let segments = 8u32;

    // Generate arrow for each axis
    for (axis_id, (color, direction)) in [
        ([1.0, 0.2, 0.2, 1.0], Vec3::X), // X = Red
        ([0.2, 1.0, 0.2, 1.0], Vec3::Y), // Y = Green
        ([0.2, 0.2, 1.0, 1.0], Vec3::Z), // Z = Blue
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

/// Ray-cylinder intersection test
fn ray_cylinder_intersection(
    ray_origin: Vec3,
    ray_dir: Vec3,
    cylinder_start: Vec3,
    cylinder_end: Vec3,
    radius: f32,
) -> Option<f32> {
    let cylinder_axis = (cylinder_end - cylinder_start).normalize();
    let cylinder_length = (cylinder_end - cylinder_start).length();

    let d = ray_dir - cylinder_axis * ray_dir.dot(cylinder_axis);
    let o = (ray_origin - cylinder_start) - cylinder_axis * (ray_origin - cylinder_start).dot(cylinder_axis);

    let a = d.dot(d);
    let b = 2.0 * d.dot(o);
    let c = o.dot(o) - radius * radius;

    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return None;
    }

    let t = (-b - discriminant.sqrt()) / (2.0 * a);
    if t < 0.0 {
        return None;
    }

    // Check if hit point is within cylinder length
    let hit_point = ray_origin + ray_dir * t;
    let projection = (hit_point - cylinder_start).dot(cylinder_axis);
    if projection < 0.0 || projection > cylinder_length {
        return None;
    }

    Some(t)
}
