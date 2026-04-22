use super::gpu_types::{GpuMeshlet, GpuQuad};
use crate::voxel::meshing::{ChunkCoord, ChunkMesh};
use bevy::prelude::*;

/// A standard Bevy Resource that buffers geometry updates for the Vulkan backend.
#[derive(Resource, Default)]
pub struct ExtractedVoxelData {
    /// A list of updated chunks: (Global Coordinate, Meshlets, Quads)
    pub updates: Vec<(IVec3, Vec<GpuMeshlet>, Vec<GpuQuad>)>,
}

/// Runs in the ExtractSchedule.
/// Since we aren't using `bevy::render`, this just reads from the main world
/// and buffers the data into a Resource for the RenderSchedule to consume.
pub fn extract_voxel_geometry(
    mut extracted_data: ResMut<ExtractedVoxelData>,
    query: Query<(&ChunkCoord, &ChunkMesh), Changed<ChunkMesh>>,
) {
    // Clear out last frame's uploads
    extracted_data.updates.clear();

    for (coord, chunk_mesh) in query.iter() {
        let mut gpu_meshlets = Vec::with_capacity(chunk_mesh.meshlets.len());

        let total_quads: usize = chunk_mesh.meshlets.iter().map(|m| m.quads.len()).sum();
        let mut gpu_quads = Vec::with_capacity(total_quads);

        for meshlet in &chunk_mesh.meshlets {
            let quad_offset = gpu_quads.len() as u32;
            let quad_count = meshlet.quads.len() as u32;

            for quad in &meshlet.quads {
                gpu_quads.push(GpuQuad::from(quad));
            }

            gpu_meshlets.push(GpuMeshlet {
                bounds_min: meshlet.bounds_min,
                bounds_max: meshlet.bounds_max,
                quad_offset,
                quad_count,
            });
        }

        extracted_data
            .updates
            .push((coord.0, gpu_meshlets, gpu_quads));
    }
}
