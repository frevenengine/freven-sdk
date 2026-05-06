use std::ops::Range;

use freven_block_sdk_types::BlockRuntimeId;
use freven_volumetric_sdk_types::{
    CHUNK_SECTION_DIM, CHUNK_SECTION_VOLUME, ColumnCoord, SectionY, WorldCellPos, section_index,
};

use crate::{InitialWorldSpawnHint, WorldGenError, WorldGenOutput, WorldTerrainWrite};

const SECTION_DIM_I32: i32 = CHUNK_SECTION_DIM as i32;

/// Error returned by SDK-side worldgen output builders.
///
/// These errors are authoring/validation helpers only. They do not change the
/// canonical worldgen wire contract.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum WorldGenOutputBuildError {
    #[error(
        "invalid FillBox bounds: min={min:?} max={max:?}; expected half-open bounds with min < max on every axis"
    )]
    InvalidBox {
        min: WorldCellPos,
        max: WorldCellPos,
    },

    #[error("column-local coordinate out of range: axis={axis} value={value}; expected {expected}")]
    LocalCoordOutOfRange {
        axis: &'static str,
        value: i32,
        expected: &'static str,
    },

    #[error("invalid section cell buffer length: len={len}, expected {expected}")]
    InvalidSectionCellBufferLen { len: usize, expected: usize },
}

impl From<WorldGenOutputBuildError> for WorldGenError {
    fn from(value: WorldGenOutputBuildError) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

/// A position inside one requested worldgen column.
///
/// `x` and `z` are column-local cell coordinates. `y` is still an absolute
/// world-cell Y coordinate because Freven columns are vertically sectioned.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColumnLocalCellPos {
    pub x: u8,
    pub y: i32,
    pub z: u8,
}

impl ColumnLocalCellPos {
    #[must_use]
    pub const fn new(x: u8, y: i32, z: u8) -> Self {
        Self { x, y, z }
    }

    #[must_use]
    pub const fn tuple(self) -> (u8, i32, u8) {
        (self.x, self.y, self.z)
    }
}

/// Builder for `WorldGenOutput` using absolute world-cell coordinates.
///
/// This is an SDK-side authoring helper. It does not change the serialized
/// `WorldGenOutput` / `WorldTerrainWrite` contract consumed by hosts.
#[derive(Debug, Clone, Default)]
pub struct WorldGenOutputBuilder {
    output: WorldGenOutput,
}

impl WorldGenOutputBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn from_output(output: WorldGenOutput) -> Self {
        Self { output }
    }

    pub fn push_write(&mut self, write: WorldTerrainWrite) -> &mut Self {
        self.output.writes.push(write);
        self
    }

    pub fn set_block(&mut self, pos: WorldCellPos, block_id: BlockRuntimeId) -> &mut Self {
        self.output
            .writes
            .push(WorldTerrainWrite::SetBlock { pos, block_id });
        self
    }

    pub fn try_fill_box(
        &mut self,
        min: WorldCellPos,
        max: WorldCellPos,
        block_id: BlockRuntimeId,
    ) -> Result<&mut Self, WorldGenOutputBuildError> {
        validate_box(min, max)?;

        if is_single_cell_box(min, max) {
            self.set_block(min, block_id);
        } else {
            self.output
                .writes
                .push(WorldTerrainWrite::FillBox { min, max, block_id });
        }

        Ok(self)
    }

    pub fn fill_section(&mut self, sy: SectionY, block_id: BlockRuntimeId) -> &mut Self {
        self.output
            .writes
            .push(WorldTerrainWrite::FillSection { sy, block_id });
        self
    }

    pub fn set_initial_world_spawn_hint(&mut self, feet_position: [f32; 3]) -> &mut Self {
        self.output.bootstrap.initial_world_spawn_hint =
            Some(InitialWorldSpawnHint { feet_position });
        self
    }

    #[must_use]
    pub fn finish(self) -> WorldGenOutput {
        self.output
    }
}

/// Builder for `WorldGenOutput` scoped to one requested column.
///
/// This helper accepts column-local X/Z coordinates and emits the same canonical
/// `WorldTerrainWrite` values as builtin providers.
#[derive(Debug, Clone)]
pub struct WorldGenColumnBuilder {
    column: ColumnCoord,
    inner: WorldGenOutputBuilder,
}

impl WorldGenColumnBuilder {
    #[must_use]
    pub fn new(column: ColumnCoord) -> Self {
        Self {
            column,
            inner: WorldGenOutputBuilder::new(),
        }
    }

    #[must_use]
    pub fn for_request(request: &crate::WorldGenRequest) -> Self {
        Self::new(request.column)
    }

    #[must_use]
    pub const fn column(&self) -> ColumnCoord {
        self.column
    }

    pub fn set_block_local(
        &mut self,
        pos: ColumnLocalCellPos,
        block_id: BlockRuntimeId,
    ) -> Result<&mut Self, WorldGenOutputBuildError> {
        self.validate_local_cell(pos)?;
        let pos = self.world_pos(pos);
        self.inner.set_block(pos, block_id);
        Ok(self)
    }

    pub fn fill_vertical_run_local(
        &mut self,
        local_x: u8,
        local_z: u8,
        y: Range<i32>,
        block_id: BlockRuntimeId,
    ) -> Result<&mut Self, WorldGenOutputBuildError> {
        validate_local_cell_coord("x", local_x)?;
        validate_local_cell_coord("z", local_z)?;

        let min = ColumnLocalCellPos::new(local_x, y.start, local_z);
        let max = ColumnLocalCellPos::new(local_x + 1, y.end, local_z + 1);

        self.try_fill_box_local(min, max, block_id)
    }

    pub fn try_fill_box_local(
        &mut self,
        min: ColumnLocalCellPos,
        max: ColumnLocalCellPos,
        block_id: BlockRuntimeId,
    ) -> Result<&mut Self, WorldGenOutputBuildError> {
        self.validate_local_cell(min)?;
        validate_local_exclusive_coord("x", max.x)?;
        validate_local_exclusive_coord("z", max.z)?;

        if let Some(sy) = full_section_y(min, max) {
            self.inner.fill_section(sy, block_id);
            return Ok(self);
        }

        let min = self.world_pos(min);
        let max = self.world_pos(max);

        self.inner.try_fill_box(min, max, block_id)?;
        Ok(self)
    }

    pub fn fill_section(&mut self, sy: SectionY, block_id: BlockRuntimeId) -> &mut Self {
        self.inner.fill_section(sy, block_id);
        self
    }

    pub fn emit_section_cells(
        &mut self,
        sy: SectionY,
        cells: &[BlockRuntimeId],
    ) -> Result<&mut Self, WorldGenOutputBuildError> {
        if cells.len() != CHUNK_SECTION_VOLUME {
            return Err(WorldGenOutputBuildError::InvalidSectionCellBufferLen {
                len: cells.len(),
                expected: CHUNK_SECTION_VOLUME,
            });
        }

        let first = cells[0];
        if cells.iter().copied().all(|block_id| block_id == first) {
            self.fill_section(sy, first);
            return Ok(self);
        }

        let section_min_y = section_y_min(sy);

        for z in 0..CHUNK_SECTION_DIM {
            for x in 0..CHUNK_SECTION_DIM {
                let mut y = 0;

                while y < CHUNK_SECTION_DIM {
                    let run_start = y;
                    let block_id = cells[section_index(x, y, z)];

                    y += 1;
                    while y < CHUNK_SECTION_DIM && cells[section_index(x, y, z)] == block_id {
                        y += 1;
                    }

                    let run_start_y =
                        section_min_y + i32::try_from(run_start).expect("section y fits i32");
                    let run_end_y = section_min_y + i32::try_from(y).expect("section y fits i32");
                    let local_x = u8::try_from(x).expect("section x fits u8");
                    let local_z = u8::try_from(z).expect("section z fits u8");

                    if y - run_start == 1 {
                        self.set_block_local(
                            ColumnLocalCellPos::new(local_x, run_start_y, local_z),
                            block_id,
                        )?;
                    } else {
                        self.fill_vertical_run_local(
                            local_x,
                            local_z,
                            run_start_y..run_end_y,
                            block_id,
                        )?;
                    }
                }
            }
        }

        Ok(self)
    }

    pub fn set_initial_world_spawn_hint(&mut self, feet_position: [f32; 3]) -> &mut Self {
        self.inner.set_initial_world_spawn_hint(feet_position);
        self
    }

    #[must_use]
    pub fn finish(self) -> WorldGenOutput {
        self.inner.finish()
    }

    fn validate_local_cell(&self, pos: ColumnLocalCellPos) -> Result<(), WorldGenOutputBuildError> {
        validate_local_cell_coord("x", pos.x)?;
        validate_local_cell_coord("z", pos.z)?;
        Ok(())
    }

    fn world_pos(&self, pos: ColumnLocalCellPos) -> WorldCellPos {
        WorldCellPos::new(
            self.column.cx * SECTION_DIM_I32 + i32::from(pos.x),
            pos.y,
            self.column.cz * SECTION_DIM_I32 + i32::from(pos.z),
        )
    }
}

fn validate_box(min: WorldCellPos, max: WorldCellPos) -> Result<(), WorldGenOutputBuildError> {
    if min.x < max.x && min.y < max.y && min.z < max.z {
        Ok(())
    } else {
        Err(WorldGenOutputBuildError::InvalidBox { min, max })
    }
}

fn is_single_cell_box(min: WorldCellPos, max: WorldCellPos) -> bool {
    min.x.checked_add(1) == Some(max.x)
        && min.y.checked_add(1) == Some(max.y)
        && min.z.checked_add(1) == Some(max.z)
}

fn validate_local_cell_coord(
    axis: &'static str,
    value: u8,
) -> Result<(), WorldGenOutputBuildError> {
    if usize::from(value) < CHUNK_SECTION_DIM {
        Ok(())
    } else {
        Err(WorldGenOutputBuildError::LocalCoordOutOfRange {
            axis,
            value: i32::from(value),
            expected: "0..32",
        })
    }
}

fn validate_local_exclusive_coord(
    axis: &'static str,
    value: u8,
) -> Result<(), WorldGenOutputBuildError> {
    if value > 0 && usize::from(value) <= CHUNK_SECTION_DIM {
        Ok(())
    } else {
        Err(WorldGenOutputBuildError::LocalCoordOutOfRange {
            axis,
            value: i32::from(value),
            expected: "1..=32",
        })
    }
}

fn full_section_y(min: ColumnLocalCellPos, max: ColumnLocalCellPos) -> Option<SectionY> {
    let full_local_xz = min.x == 0
        && usize::from(max.x) == CHUNK_SECTION_DIM
        && min.z == 0
        && usize::from(max.z) == CHUNK_SECTION_DIM;

    if !full_local_xz {
        return None;
    }

    if min.y.rem_euclid(SECTION_DIM_I32) != 0 {
        return None;
    }

    if min.y.checked_add(SECTION_DIM_I32) != Some(max.y) {
        return None;
    }

    i8::try_from(min.y.div_euclid(SECTION_DIM_I32))
        .ok()
        .map(SectionY::new)
}

fn section_y_min(sy: SectionY) -> i32 {
    i32::from(sy.raw()) * SECTION_DIM_I32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_builder_set_block_emits_set_block() {
        let mut builder = WorldGenOutputBuilder::new();

        builder.set_block(WorldCellPos::new(1, 2, 3), BlockRuntimeId(7));

        assert_eq!(
            builder.finish().writes,
            vec![WorldTerrainWrite::SetBlock {
                pos: WorldCellPos::new(1, 2, 3),
                block_id: BlockRuntimeId(7),
            }]
        );
    }

    #[test]
    fn output_builder_rejects_zero_volume_boxes() {
        let mut builder = WorldGenOutputBuilder::new();

        let err = builder
            .try_fill_box(
                WorldCellPos::new(0, 0, 0),
                WorldCellPos::new(0, 1, 1),
                BlockRuntimeId(1),
            )
            .unwrap_err();

        assert!(matches!(err, WorldGenOutputBuildError::InvalidBox { .. }));
    }

    #[test]
    fn output_builder_rejects_inverted_boxes() {
        let mut builder = WorldGenOutputBuilder::new();

        let err = builder
            .try_fill_box(
                WorldCellPos::new(4, 0, 0),
                WorldCellPos::new(3, 1, 1),
                BlockRuntimeId(1),
            )
            .unwrap_err();

        assert!(matches!(err, WorldGenOutputBuildError::InvalidBox { .. }));
    }

    #[test]
    fn output_builder_coalesces_single_cell_box_to_set_block() {
        let mut builder = WorldGenOutputBuilder::new();

        builder
            .try_fill_box(
                WorldCellPos::new(5, 6, 7),
                WorldCellPos::new(6, 7, 8),
                BlockRuntimeId(9),
            )
            .expect("valid single-cell box");

        assert_eq!(
            builder.finish().writes,
            vec![WorldTerrainWrite::SetBlock {
                pos: WorldCellPos::new(5, 6, 7),
                block_id: BlockRuntimeId(9),
            }]
        );
    }

    #[test]
    fn output_builder_emits_multi_cell_fill_box() {
        let mut builder = WorldGenOutputBuilder::new();

        builder
            .try_fill_box(
                WorldCellPos::new(5, 6, 7),
                WorldCellPos::new(6, 10, 8),
                BlockRuntimeId(9),
            )
            .expect("valid vertical run");

        assert_eq!(
            builder.finish().writes,
            vec![WorldTerrainWrite::FillBox {
                min: WorldCellPos::new(5, 6, 7),
                max: WorldCellPos::new(6, 10, 8),
                block_id: BlockRuntimeId(9),
            }]
        );
    }

    #[test]
    fn column_builder_converts_local_coords_to_absolute_coords_for_negative_columns() {
        let mut builder = WorldGenColumnBuilder::new(ColumnCoord::new(-1, 2));

        builder
            .set_block_local(ColumnLocalCellPos::new(31, 5, 0), BlockRuntimeId(4))
            .expect("valid local cell");

        assert_eq!(
            builder.finish().writes,
            vec![WorldTerrainWrite::SetBlock {
                pos: WorldCellPos::new(-1, 5, 64),
                block_id: BlockRuntimeId(4),
            }]
        );
    }

    #[test]
    fn column_builder_rejects_out_of_range_local_cells() {
        let mut builder = WorldGenColumnBuilder::new(ColumnCoord::new(0, 0));

        let err = builder
            .set_block_local(ColumnLocalCellPos::new(32, 0, 0), BlockRuntimeId(1))
            .unwrap_err();

        assert!(matches!(
            err,
            WorldGenOutputBuildError::LocalCoordOutOfRange { axis: "x", .. }
        ));
    }

    #[test]
    fn column_builder_vertical_run_len_one_emits_set_block() {
        let mut builder = WorldGenColumnBuilder::new(ColumnCoord::new(1, -1));

        builder
            .fill_vertical_run_local(2, 3, 10..11, BlockRuntimeId(5))
            .expect("valid one-cell run");

        assert_eq!(
            builder.finish().writes,
            vec![WorldTerrainWrite::SetBlock {
                pos: WorldCellPos::new(34, 10, -29),
                block_id: BlockRuntimeId(5),
            }]
        );
    }

    #[test]
    fn column_builder_vertical_run_len_many_emits_fill_box() {
        let mut builder = WorldGenColumnBuilder::new(ColumnCoord::new(1, -1));

        builder
            .fill_vertical_run_local(2, 3, 10..14, BlockRuntimeId(5))
            .expect("valid vertical run");

        assert_eq!(
            builder.finish().writes,
            vec![WorldTerrainWrite::FillBox {
                min: WorldCellPos::new(34, 10, -29),
                max: WorldCellPos::new(35, 14, -28),
                block_id: BlockRuntimeId(5),
            }]
        );
    }

    #[test]
    fn column_builder_full_local_section_emits_fill_section() {
        let mut builder = WorldGenColumnBuilder::new(ColumnCoord::new(9, 10));

        builder
            .try_fill_box_local(
                ColumnLocalCellPos::new(0, -32, 0),
                ColumnLocalCellPos::new(32, 0, 32),
                BlockRuntimeId(3),
            )
            .expect("valid full section");

        assert_eq!(
            builder.finish().writes,
            vec![WorldTerrainWrite::FillSection {
                sy: SectionY::new(-1),
                block_id: BlockRuntimeId(3),
            }]
        );
    }

    #[test]
    fn column_builder_emit_uniform_section_cells_emits_fill_section() {
        let mut builder = WorldGenColumnBuilder::new(ColumnCoord::new(0, 0));
        let cells = vec![BlockRuntimeId(8); CHUNK_SECTION_VOLUME];

        builder
            .emit_section_cells(SectionY::new(2), &cells)
            .expect("valid section cells");

        assert_eq!(
            builder.finish().writes,
            vec![WorldTerrainWrite::FillSection {
                sy: SectionY::new(2),
                block_id: BlockRuntimeId(8),
            }]
        );
    }

    #[test]
    fn column_builder_emit_section_cells_emits_vertical_runs() {
        let mut builder = WorldGenColumnBuilder::new(ColumnCoord::new(0, 0));
        let mut cells = vec![BlockRuntimeId(1); CHUNK_SECTION_VOLUME];
        cells[section_index(0, 16, 0)] = BlockRuntimeId(2);

        builder
            .emit_section_cells(SectionY::new(0), &cells)
            .expect("valid section cells");

        let output = builder.finish();

        assert_eq!(
            output.writes.len(),
            CHUNK_SECTION_DIM * CHUNK_SECTION_DIM + 2
        );
        assert_eq!(
            &output.writes[0..3],
            &[
                WorldTerrainWrite::FillBox {
                    min: WorldCellPos::new(0, 0, 0),
                    max: WorldCellPos::new(1, 16, 1),
                    block_id: BlockRuntimeId(1),
                },
                WorldTerrainWrite::SetBlock {
                    pos: WorldCellPos::new(0, 16, 0),
                    block_id: BlockRuntimeId(2),
                },
                WorldTerrainWrite::FillBox {
                    min: WorldCellPos::new(0, 17, 0),
                    max: WorldCellPos::new(1, 32, 1),
                    block_id: BlockRuntimeId(1),
                },
            ]
        );
    }

    #[test]
    fn column_builder_emit_section_cells_rejects_wrong_len() {
        let mut builder = WorldGenColumnBuilder::new(ColumnCoord::new(0, 0));
        let cells = vec![BlockRuntimeId(8); CHUNK_SECTION_VOLUME - 1];

        let err = builder
            .emit_section_cells(SectionY::new(0), &cells)
            .unwrap_err();

        assert_eq!(
            err,
            WorldGenOutputBuildError::InvalidSectionCellBufferLen {
                len: CHUNK_SECTION_VOLUME - 1,
                expected: CHUNK_SECTION_VOLUME,
            }
        );
    }

    #[test]
    fn builder_preserves_bootstrap_hint() {
        let mut builder = WorldGenColumnBuilder::new(ColumnCoord::new(0, 0));

        builder.set_initial_world_spawn_hint([16.5, 65.0, 16.5]);

        assert_eq!(
            builder
                .finish()
                .bootstrap
                .initial_world_spawn_hint
                .expect("spawn hint")
                .feet_position,
            [16.5, 65.0, 16.5]
        );
    }
}
