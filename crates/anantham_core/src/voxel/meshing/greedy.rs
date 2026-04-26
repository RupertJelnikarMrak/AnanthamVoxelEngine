use super::context::MeshingContext;
use super::quad::{UnpackedQuad, VoxelFace};
use super::registry::MeshingAttributes;
use crate::voxel::block::BlockState;
use crate::voxel::chunk::{CHUNK_SIZE, CHUNK_SIZE_I32};
use crate::voxel::meshing::MeshingRegistry;
use bevy::math::IVec3;

pub const MASK_SIZE: usize = CHUNK_SIZE * CHUNK_SIZE;

#[inline(always)]
fn get_face(axis: usize, is_positive_dir: bool) -> VoxelFace {
    match (axis, is_positive_dir) {
        (0, true) => VoxelFace::PositiveX,
        (0, false) => VoxelFace::NegativeX,
        (1, true) => VoxelFace::PositiveY,
        (1, false) => VoxelFace::NegativeY,
        (2, true) => VoxelFace::PositiveZ,
        (2, false) => VoxelFace::NegativeZ,
        _ => unreachable!(),
    }
}

pub fn generate_greedy_quads(
    ctx: &MeshingContext,
    registry: &MeshingRegistry,
) -> Vec<UnpackedQuad> {
    let mut quads = Vec::new();

    let mut mask: [Option<(BlockState, VoxelFace)>; MASK_SIZE] = [None; MASK_SIZE];

    let registry_lock = registry.data.read().unwrap();
    let registry_slice: &[MeshingAttributes] = registry_lock.as_slice();

    for axis in 0..3 {
        let u = (axis + 1) % 3;
        let v = (axis + 2) % 3;

        let mut x = [0i32; 3];
        let mut q = [0i32; 3];
        q[axis] = 1;

        mask.fill(None);

        x[axis] = -1;
        while x[axis] < CHUNK_SIZE_I32 {
            let mut n = 0;
            x[v] = 0;
            while x[v] < CHUNK_SIZE_I32 {
                x[u] = 0;
                while x[u] < CHUNK_SIZE_I32 {
                    let current_local = IVec3::new(x[0], x[1], x[2]);
                    let next_local = IVec3::new(x[0] + q[0], x[1] + q[1], x[2] + q[2]);

                    let state_current = ctx.get_block_extended(current_local);
                    let state_next = ctx.get_block_extended(next_local);

                    // SAFETY: The mesher assumes that all BlockStates present in a chunk
                    // have been properly registered and exist in the MeshingRegistry.
                    unsafe {
                        let current_attr = *registry_slice.get_unchecked(state_current.0 as usize);
                        let next_attr = *registry_slice.get_unchecked(state_next.0 as usize);

                        let identical_blocks = state_current == state_next;

                        if current_attr.fractional_mask() != 0 || next_attr.fractional_mask() != 0 {
                            mask[n] = None;
                        } else if !identical_blocks {
                            let current_is_inside = x[axis] >= 0;
                            let next_is_inside = (x[axis] + 1) < CHUNK_SIZE_I32;

                            if current_is_inside
                                && current_attr.is_visible()
                                && next_attr.is_transparent()
                            {
                                mask[n] = Some((state_current, get_face(axis, true)));
                            } else if next_is_inside
                                && next_attr.is_visible()
                                && current_attr.is_transparent()
                            {
                                mask[n] = Some((state_next, get_face(axis, false)));
                            } else {
                                mask[n] = None;
                            }
                        } else {
                            mask[n] = None;
                        }
                    }

                    x[u] += 1;
                    n += 1;
                }
                x[v] += 1;
            }

            x[axis] += 1;

            n = 0;
            let mut j = 0;
            while j < CHUNK_SIZE_I32 {
                let mut i = 0;
                while i < CHUNK_SIZE_I32 {
                    if let Some((target_state, target_face)) = mask[n] {
                        let mut width = 1;
                        while i + width < CHUNK_SIZE_I32
                            && mask[n + width as usize] == Some((target_state, target_face))
                        {
                            width += 1;
                        }

                        let mut height = 1;
                        let mut done = false;
                        while j + height < CHUNK_SIZE_I32 {
                            for k in 0..width {
                                if mask[n + k as usize + (height * CHUNK_SIZE_I32) as usize]
                                    != Some((target_state, target_face))
                                {
                                    done = true;
                                    break;
                                }
                            }
                            if done {
                                break;
                            }
                            height += 1;
                        }

                        let mut min = [0u8; 3];
                        min[u] = i as u8;
                        min[v] = j as u8;

                        // x[axis] was already incremented earlier in the loop.
                        // If it's a positive face, it belongs to the block behind the boundary (x - 1).
                        // If it's a negative face, it belongs to the block in front of the boundary (x).
                        let is_positive_face = target_face == VoxelFace::PositiveX
                            || target_face == VoxelFace::PositiveY
                            || target_face == VoxelFace::PositiveZ;

                        min[axis] = if is_positive_face {
                            (x[axis] - 1) as u8
                        } else {
                            x[axis] as u8
                        };

                        // SAFETY: target_state was pulled directly from the chunk data
                        let material_id = unsafe {
                            registry_lock
                                .get_unchecked(target_state.0 as usize)
                                .material_id()
                        };

                        quads.push(UnpackedQuad {
                            min,
                            width: width as u8,
                            height: height as u8,
                            face: target_face,
                            state: target_state,
                            material_id,
                        });

                        for l in 0..height {
                            for k in 0..width {
                                mask[n + k as usize + (l * CHUNK_SIZE_I32) as usize] = None;
                            }
                        }

                        i += width;
                        n += width as usize;
                    } else {
                        i += 1;
                        n += 1;
                    }
                }
                j += 1;
            }
        }
    }

    quads
}
