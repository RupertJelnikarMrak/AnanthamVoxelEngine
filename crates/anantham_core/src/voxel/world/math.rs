use crate::voxel::chunk::{CHUNK_SIZE, CHUNK_SIZE_I32};
use bevy::math::IVec3;

#[inline(always)]
pub fn global_to_local(global: IVec3) -> (IVec3, IVec3) {
    let size = CHUNK_SIZE as i32;

    let chunk_coord = IVec3::new(
        global.x.div_euclid(size) * size,
        global.y.div_euclid(size) * size,
        global.z.div_euclid(size) * size,
    );

    let local_coord = IVec3::new(
        global.x.rem_euclid(size),
        global.y.rem_euclid(size),
        global.z.rem_euclid(size),
    );

    (chunk_coord, local_coord)
}

#[inline(always)]
pub fn local_to_index(local: IVec3) -> usize {
    (local.x + local.y * CHUNK_SIZE_I32 + local.z * CHUNK_SIZE_I32 * CHUNK_SIZE_I32) as usize
}
