use super::quad::UnpackedQuad;
use bevy::math::Vec3;

/// 1 Quad = 4 Vertices. Max 64 vertices per Vulkan Mesh Shader threadgroup.
pub const MAX_QUADS_PER_MESHLET: usize = 16;

#[derive(Debug, Clone)]
pub struct Meshlet {
    pub quads: Vec<UnpackedQuad>,
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
}

impl Default for Meshlet {
    fn default() -> Self {
        Self {
            quads: Vec::with_capacity(MAX_QUADS_PER_MESHLET),
            bounds_min: Vec3::splat(f32::MAX),
            bounds_max: Vec3::splat(f32::MIN),
        }
    }
}

impl Meshlet {
    pub fn add_quad(&mut self, quad: &UnpackedQuad) {
        self.quads.push(quad.clone());

        let min = Vec3::new(quad.min[0] as f32, quad.min[1] as f32, quad.min[2] as f32);
        let mut max = min;

        match quad.face {
            super::quad::VoxelFace::NegativeZ | super::quad::VoxelFace::PositiveZ => {
                max.x += quad.width as f32;
                max.y += quad.height as f32;
            }
            super::quad::VoxelFace::NegativeX | super::quad::VoxelFace::PositiveX => {
                max.z += quad.width as f32;
                max.y += quad.height as f32;
            }
            super::quad::VoxelFace::NegativeY | super::quad::VoxelFace::PositiveY => {
                max.x += quad.width as f32;
                max.z += quad.height as f32;
            }
        }

        self.bounds_min = self.bounds_min.min(min);
        self.bounds_max = self.bounds_max.max(max);
    }
}

pub fn build_meshlets(raw_quads: Vec<UnpackedQuad>) -> Vec<Meshlet> {
    let mut meshlets = Vec::new();
    let mut current_meshlet = Meshlet::default();

    for quad in raw_quads {
        if current_meshlet.quads.len() >= MAX_QUADS_PER_MESHLET {
            meshlets.push(current_meshlet);
            current_meshlet = Meshlet::default();
        }
        current_meshlet.add_quad(&quad);
    }

    if !current_meshlet.quads.is_empty() {
        meshlets.push(current_meshlet);
    }

    meshlets
}
