//! Vertex types for 3D rendering

use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;

/// Standard 3D vertex with position, normal, and color
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct Vertex3D {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 4],
}

impl Vertex3D {
    /// Create a new vertex
    pub fn new(position: [f32; 3], normal: [f32; 3], color: [f32; 4]) -> Self {
        Self {
            position,
            normal,
            color,
        }
    }

    /// Create a vertex with default white color
    pub fn with_pos_normal(position: [f32; 3], normal: [f32; 3]) -> Self {
        Self {
            position,
            normal,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }

    /// Vulkano vertex buffer description
    pub fn per_vertex() -> vulkano::pipeline::graphics::vertex_input::VertexBufferDescription {
        vulkano::pipeline::graphics::vertex_input::VertexBufferDescription {
            stride: std::mem::size_of::<Self>() as u32,
            input_rate: vulkano::pipeline::graphics::vertex_input::VertexInputRate::Vertex,
            members: HashMap::from([
                (
                    "position".to_string(),
                    vulkano::pipeline::graphics::vertex_input::VertexMemberInfo {
                        offset: 0,
                        format: vulkano::format::Format::R32G32B32_SFLOAT,
                        num_elements: 1,
                        stride: std::mem::size_of::<Self>() as u32,
                    },
                ),
                (
                    "normal".to_string(),
                    vulkano::pipeline::graphics::vertex_input::VertexMemberInfo {
                        offset: 12,
                        format: vulkano::format::Format::R32G32B32_SFLOAT,
                        num_elements: 1,
                        stride: std::mem::size_of::<Self>() as u32,
                    },
                ),
                (
                    "color".to_string(),
                    vulkano::pipeline::graphics::vertex_input::VertexMemberInfo {
                        offset: 24,
                        format: vulkano::format::Format::R32G32B32A32_SFLOAT,
                        num_elements: 1,
                        stride: std::mem::size_of::<Self>() as u32,
                    },
                ),
            ]),
        }
    }
}

/// Simple sky vertex with just position
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct SkyVertex {
    pub position: [f32; 3],
}

impl SkyVertex {
    pub fn new(position: [f32; 3]) -> Self {
        Self { position }
    }

    pub fn per_vertex() -> vulkano::pipeline::graphics::vertex_input::VertexBufferDescription {
        vulkano::pipeline::graphics::vertex_input::VertexBufferDescription {
            stride: std::mem::size_of::<Self>() as u32,
            input_rate: vulkano::pipeline::graphics::vertex_input::VertexInputRate::Vertex,
            members: HashMap::from([(
                "position".to_string(),
                vulkano::pipeline::graphics::vertex_input::VertexMemberInfo {
                    offset: 0,
                    format: vulkano::format::Format::R32G32B32_SFLOAT,
                    num_elements: 1,
                    stride: std::mem::size_of::<Self>() as u32,
                },
            )]),
        }
    }
}
