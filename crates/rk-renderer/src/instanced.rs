//! Instance buffer management utilities
//!
//! This module provides utilities for managing instance buffers used
//! in instanced rendering (axis, marker, gizmo renderers).

use bytemuck::Pod;
use std::marker::PhantomData;

/// Manages an instance buffer with automatic capacity tracking.
///
/// This struct handles the common pattern of:
/// - Pre-allocating a buffer with maximum capacity
/// - Tracking current instance count
/// - Warning when instances exceed capacity
///
/// # Type Parameters
///
/// * `T` - The instance data type. Must implement `Pod` for zero-copy GPU upload.
pub struct InstanceBuffer<T: Pod> {
    buffer: wgpu::Buffer,
    count: u32,
    max_instances: u32,
    _marker: PhantomData<T>,
}

impl<T: Pod> InstanceBuffer<T> {
    /// Create a new instance buffer with the given capacity.
    ///
    /// # Arguments
    ///
    /// * `device` - The wgpu device.
    /// * `label` - Buffer label for debugging.
    /// * `max_instances` - Maximum number of instances this buffer can hold.
    pub fn new(device: &wgpu::Device, label: &str, max_instances: u32) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("{} Instance Buffer", label)),
            size: (max_instances as usize * std::mem::size_of::<T>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            count: 0,
            max_instances,
            _marker: PhantomData,
        }
    }

    /// Create a new instance buffer with initial data.
    ///
    /// The buffer is created with exactly enough space for the initial data.
    /// Use `new()` if you need a pre-allocated buffer with larger capacity.
    pub fn with_data(device: &wgpu::Device, label: &str, data: &[T]) -> Self {
        use wgpu::util::DeviceExt;

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{} Instance Buffer", label)),
            contents: bytemuck::cast_slice(data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            buffer,
            count: data.len() as u32,
            max_instances: data.len() as u32,
            _marker: PhantomData,
        }
    }

    /// Update the instance buffer with new data.
    ///
    /// If `instances` exceeds the maximum capacity, a warning is logged
    /// and the data is truncated.
    ///
    /// # Arguments
    ///
    /// * `queue` - The wgpu queue for buffer writes.
    /// * `instances` - Slice of instance data to upload.
    pub fn update(&mut self, queue: &wgpu::Queue, instances: &[T]) {
        let count = instances.len();

        if count > self.max_instances as usize {
            tracing::warn!(
                "Instance count {} exceeds maximum {}, truncating",
                count,
                self.max_instances
            );
        }

        let count = count.min(self.max_instances as usize);
        self.count = count as u32;

        if count > 0 {
            queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&instances[..count]));
        }
    }

    /// Update a single instance at the given index.
    ///
    /// # Panics
    ///
    /// Panics if `index >= count`.
    pub fn update_single(&self, queue: &wgpu::Queue, index: u32, instance: &T) {
        debug_assert!(index < self.count, "Instance index out of bounds");
        let offset = (index as usize * std::mem::size_of::<T>()) as u64;
        queue.write_buffer(&self.buffer, offset, bytemuck::bytes_of(instance));
    }

    /// Clear all instances.
    pub fn clear(&mut self) {
        self.count = 0;
    }

    /// Get the current instance count.
    pub fn count(&self) -> u32 {
        self.count
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Get the maximum capacity.
    pub fn max_instances(&self) -> u32 {
        self.max_instances
    }

    /// Get a reference to the underlying buffer.
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    /// Get a buffer slice for use in render passes.
    pub fn slice(&self) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(..)
    }
}

/// Single instance buffer for renderers that only need one instance.
///
/// Simpler interface than `InstanceBuffer` when you always have exactly
/// one instance (like the gizmo renderer).
pub struct SingleInstanceBuffer<T: Pod> {
    buffer: wgpu::Buffer,
    data: T,
    _marker: PhantomData<T>,
}

impl<T: Pod + Default> SingleInstanceBuffer<T> {
    /// Create a new single instance buffer with default data.
    pub fn new(device: &wgpu::Device, label: &str) -> Self {
        use wgpu::util::DeviceExt;

        let data = T::default();
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{} Instance Buffer", label)),
            contents: bytemuck::bytes_of(&data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            buffer,
            data,
            _marker: PhantomData,
        }
    }
}

impl<T: Pod> SingleInstanceBuffer<T> {
    /// Get a reference to the current data.
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Get a mutable reference to the data.
    ///
    /// After modifying, call `upload()` to sync changes to GPU.
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Upload the current data to the GPU buffer.
    pub fn upload(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&self.data));
    }

    /// Set new data and upload to GPU.
    pub fn set(&mut self, queue: &wgpu::Queue, data: T) {
        self.data = data;
        self.upload(queue);
    }

    /// Get a reference to the underlying buffer.
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    /// Get a buffer slice for use in render passes.
    pub fn slice(&self) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(..)
    }
}
