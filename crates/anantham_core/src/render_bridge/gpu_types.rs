use crate::voxel::meshing::quad::{UnpackedQuad, VoxelFace};

/// The exact byte-layout that the Vulkan GLSL Mesh Shader will read.
/// Size: 8 bytes per Quad.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuQuad {
    // bits 0-7: min.x
    // bits 8-15: min.y
    // bits 16-23: min.z
    // bits 24-31: width
    pub geometry_data: u32,

    // bits 0-7: height
    // bits 8-10: face direction (0-5)
    // bits 11-15: padding/future use
    // bits 16-31: material_id
    pub material_data: u32,
}

impl From<&UnpackedQuad> for GpuQuad {
    fn from(quad: &UnpackedQuad) -> Self {
        let geometry_data = (quad.min[0] as u32)
            | ((quad.min[1] as u32) << 8)
            | ((quad.min[2] as u32) << 16)
            | ((quad.width as u32) << 24);

        let face_idx = match quad.face {
            VoxelFace::NegativeX => 0,
            VoxelFace::PositiveX => 1,
            VoxelFace::NegativeY => 2,
            VoxelFace::PositiveY => 3,
            VoxelFace::NegativeZ => 4,
            VoxelFace::PositiveZ => 5,
        };

        let material_data =
            (quad.height as u32) | ((face_idx as u32) << 8) | ((quad.material_id as u32) << 16);

        Self {
            geometry_data,
            material_data,
        }
    }
}

/// Represents the data for a single Threadgroup in the Vulkan Mesh Shader.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuMeshlet {
    pub chunk_x: f32,
    pub chunk_y: f32,
    pub chunk_z: f32,

    pub bounds_min_x: f32,
    pub bounds_min_y: f32,
    pub bounds_min_z: f32,

    pub bounds_max_x: f32,
    pub bounds_max_y: f32,
    pub bounds_max_z: f32,

    pub quad_offset: u32,
    pub quad_count: u32,
    pub padding: u32,
}
