use super::gpu_types::{GpuMeshlet, GpuQuad};
use crate::{
    spatial::camera::PlayerCamera,
    voxel::meshing::{ChunkCoord, ChunkMesh},
};
use bevy::{prelude::*, window::PrimaryWindow};
use std::collections::HashMap;

/// A standard Bevy Resource that buffers geometry updates for the Vulkan backend.
#[derive(Resource, Default)]
pub struct ExtractedVoxelData {
    /// A list of updated chunks: (Global Coordinate, Meshlets, Quads)
    pub active_chunks: HashMap<IVec3, (Vec<GpuMeshlet>, Vec<GpuQuad>)>,
    pub requires_upload: bool,
}

/// Runs in the ExtractSchedule.
/// Since we aren't using `bevy::render`, this just reads from the main world
/// and buffers the data into a Resource for the RenderSchedule to consume.
pub fn extract_voxel_geometry(
    mut extracted_data: ResMut<ExtractedVoxelData>,
    query: Query<(&ChunkCoord, &ChunkMesh), Changed<ChunkMesh>>,
) {
    let mut changed = false;

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
                chunk_x: coord.0.x as f32,
                chunk_y: coord.0.y as f32,
                chunk_z: coord.0.z as f32,

                bounds_min_x: meshlet.bounds_min.x,
                bounds_min_y: meshlet.bounds_min.y,
                bounds_min_z: meshlet.bounds_min.z,

                bounds_max_x: meshlet.bounds_max.x,
                bounds_max_y: meshlet.bounds_max.y,
                bounds_max_z: meshlet.bounds_max.z,

                quad_offset,
                quad_count,
                padding: 0,
            });
        }

        extracted_data
            .active_chunks
            .insert(coord.0, (gpu_meshlets, gpu_quads));
        changed = true;
    }

    if changed {
        extracted_data.requires_upload = true;
    }
}

#[derive(Resource, Default, Clone)]
pub struct ExtractedCamera {
    pub view_proj: [f32; 16],
    pub position: Vec3,
}

pub fn extract_camera(
    mut extracted: ResMut<ExtractedCamera>,
    query: Query<(&Transform, &PlayerCamera)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok((transform, camera)) = query.single() else {
        return;
    };
    let Ok(window) = window_query.single() else {
        return;
    };

    let aspect = window.resolution.width() / window.resolution.height().max(1.0);

    let mut proj = Mat4::perspective_infinite_reverse_rh(camera.fov, aspect, 0.1);
    proj.y_axis.y *= -1.0;
    let view = transform.to_matrix().inverse();

    extracted.view_proj = (proj * view).to_cols_array();
    extracted.position = transform.translation;
}
