pub type ShapeMask = u8;

/// We reverse the shape mask so a Block's base_id defaults to full.
pub const SHAPE_FULL: ShapeMask = 0b0000_0000;
pub const SHAPE_EMPTY: ShapeMask = 0b1111_1111;

/// The general archtype, one has multiple block states based on orientation...
/// Made for user understanding and simplicty.
#[derive(Debug, PartialEq, Eq)]
pub enum Archetype {
    Full,
    /// Empty isn't really an allowed state in world, it simply defaults back to BlockState::AIR
    Empty,
    Quarter,
    Slab,
    VerticalSlab,
    Stair,
    /// Represents custom carving. Holds the *remaining* volume (1 to 7)
    Irregular(u8),
}

/// Hardcoded translator that evaluates the bitmask regardless of rotation
pub fn identify_archetype(mask: ShapeMask) -> Archetype {
    let missing_volume = mask.count_ones() as u8;
    let remaining_volume = 8 - missing_volume;

    match missing_volume {
        0 => Archetype::Full,
        8 => Archetype::Empty,
        7 => Archetype::Quarter,
        4 => {
            // SLAB CHECKS
            // 0b0000_1111 (Top 4 missing) or 0b1111_0000 (Bottom 4 missing)
            if mask == 0x0F || mask == 0xF0 {
                return Archetype::Slab;
            }
            // Vertical Slabs (Left/Right or Front/Back missing)
            if mask == 0xCC || mask == 0x33 || mask == 0xAA || mask == 0x55 {
                return Archetype::VerticalSlab;
            }
            Archetype::Irregular(remaining_volume)
        }
        2 => {
            // STAIR CHECKS (Exactly 2 missing sub-voxels forming a corner column)
            // Example: Top-Right-Front and Top-Right-Back missing
            const STAIR_MASKS: [ShapeMask; 8] = [
                0b0000_0011,
                0b0000_1100,
                0b0011_0000,
                0b1100_0000,
                0b0000_0101,
                0b0000_1010,
                0b0101_0000,
                0b1010_0000,
            ];

            if STAIR_MASKS.contains(&mask) {
                Archetype::Stair
            } else {
                Archetype::Irregular(remaining_volume)
            }
        }
        _ => Archetype::Irregular(remaining_volume),
    }
}
