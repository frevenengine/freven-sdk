//! Stable volumetric foundation SDK contracts.
//!
//! This crate owns public volumetric topology and coordinate helpers.
//! Storage encoding details remain engine-internal unless explicitly promoted later.

/// Chunk section edge length in cells.
///
/// Protocol v1 currently uses 32^3 sections.
pub const CHUNK_SECTION_DIM: usize = 32;

/// Total cells in a section.
pub const CHUNK_SECTION_VOLUME: usize = CHUNK_SECTION_DIM * CHUNK_SECTION_DIM * CHUNK_SECTION_DIM;

/// Canonical linearization order for a 32x32x32 section.
///
/// X-major order (x changes fastest), then Y, then Z:
/// index = x + DIM * (y + DIM * z)
#[inline]
pub fn section_index(x: usize, y: usize, z: usize) -> usize {
    debug_assert!(x < CHUNK_SECTION_DIM);
    debug_assert!(y < CHUNK_SECTION_DIM);
    debug_assert!(z < CHUNK_SECTION_DIM);
    x + CHUNK_SECTION_DIM * (y + CHUNK_SECTION_DIM * z)
}

/// Floor division and modulo for negative coordinates.
///
/// Returns (q, r) such that:
/// - x = q * d + r
/// - r in [0, d)
#[inline]
pub fn div_mod_floor_i32(x: i32, d: i32) -> (i32, i32) {
    debug_assert!(d > 0);
    let mut q = x / d;
    let mut r = x % d;
    if r < 0 {
        r += d;
        q -= 1;
    }
    (q, r)
}

/// Convert world-space cell coordinate to (section coordinate, local coordinate in 0..DIM).
#[inline]
pub fn world_to_section_and_local(w: i32) -> (i32, usize) {
    let (q, r) = div_mod_floor_i32(w, CHUNK_SECTION_DIM as i32);
    debug_assert!(r >= 0);
    (q, r as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_index_matches_expected_layout() {
        assert_eq!(section_index(0, 0, 0), 0);
        assert_eq!(section_index(1, 0, 0), 1);
        assert_eq!(section_index(0, 1, 0), CHUNK_SECTION_DIM);
        assert_eq!(
            section_index(0, 0, 1),
            CHUNK_SECTION_DIM * CHUNK_SECTION_DIM
        );
    }

    #[test]
    fn div_mod_floor_handles_negative() {
        let (q, r) = div_mod_floor_i32(-1, 32);
        assert_eq!(q, -1);
        assert_eq!(r, 31);

        let (q, r) = div_mod_floor_i32(-32, 32);
        assert_eq!(q, -1);
        assert_eq!(r, 0);

        let (q, r) = div_mod_floor_i32(-33, 32);
        assert_eq!(q, -2);
        assert_eq!(r, 31);
    }
}
