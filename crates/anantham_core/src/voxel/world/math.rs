use bevy::math::IVec3;

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_SIZE_I32: i32 = 32;

#[inline(always)]
pub fn global_to_local(global: IVec3) -> (IVec3, IVec3) {
    let chunk_coord = global.div_euclid(IVec3::splat(CHUNK_SIZE_I32));
    let local_coord = global.rem_euclid(IVec3::splat(CHUNK_SIZE_I32));

    (chunk_coord, local_coord)
}

#[inline(always)]
pub fn local_to_index(local: IVec3) -> usize {
    (local.x + local.y * CHUNK_SIZE_I32 + local.z * CHUNK_SIZE_I32 * CHUNK_SIZE_I32) as usize
}
