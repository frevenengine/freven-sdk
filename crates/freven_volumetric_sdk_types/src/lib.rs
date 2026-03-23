//! Stable volumetric foundation SDK contracts.
//!
//! This crate owns public volumetric topology, section/column addressing,
//! and coordinate helpers.
//!
//! It does not own block gameplay semantics.

mod addressing;
mod topology;

pub use addressing::{ColumnCoord, SectionCoord, SectionY, WorldCellPos};
pub use topology::{
    CHUNK_SECTION_DIM, CHUNK_SECTION_VOLUME, div_mod_floor_i32, section_index,
    world_to_section_and_local,
};
