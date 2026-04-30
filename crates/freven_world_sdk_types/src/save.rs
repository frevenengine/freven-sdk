use serde::{Deserialize, Serialize};

pub const WORLD_SAVE_FORMAT_VERSION: u32 = 1;
pub const DIMENSION_SAVE_FORMAT_VERSION: u32 = 1;
pub const DEFAULT_PRIMARY_DIMENSION_ID: &str = "overworld";
const MM_PER_METER: f32 = 1000.0;

/// Explicit authoritative vertical section contract for one dimension.
///
/// This is world-owned bootstrap/save truth and must not be inferred from
/// observed/materialized runtime terrain.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct DimensionVerticalContract {
    pub min_section_y: i8,
    pub section_count: u16,
    pub vertical_streaming_enabled: bool,
}

impl DimensionVerticalContract {
    #[must_use]
    pub const fn primary_default() -> Self {
        Self {
            min_section_y: 0,
            section_count: 1,
            vertical_streaming_enabled: false,
        }
    }
}

impl Default for DimensionVerticalContract {
    fn default() -> Self {
        Self::primary_default()
    }
}

/// Persisted, host-resolved initial world spawn for a world save.
///
/// - This is world-owned truth (not advisory worldgen output).
/// - Position is the world-space feet position quantized to millimeters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InitialWorldSpawn {
    pub feet_position_mm: [i32; 3],
    #[serde(default = "default_initial_world_spawn_dimension_id")]
    pub dimension_id: String,
}

impl InitialWorldSpawn {
    #[must_use]
    pub fn from_feet_position_meters(feet_position_m: [f32; 3]) -> Self {
        Self::from_feet_position_meters_in_dimension(feet_position_m, DEFAULT_PRIMARY_DIMENSION_ID)
    }

    #[must_use]
    pub fn from_feet_position_meters_in_dimension(
        feet_position_m: [f32; 3],
        dimension_id: impl Into<String>,
    ) -> Self {
        Self {
            feet_position_mm: [
                meters_to_mm(feet_position_m[0]),
                meters_to_mm(feet_position_m[1]),
                meters_to_mm(feet_position_m[2]),
            ],
            dimension_id: dimension_id.into(),
        }
    }

    #[must_use]
    pub fn feet_position_meters(&self) -> [f32; 3] {
        [
            self.feet_position_mm[0] as f32 / MM_PER_METER,
            self.feet_position_mm[1] as f32 / MM_PER_METER,
            self.feet_position_mm[2] as f32 / MM_PER_METER,
        ]
    }
}

fn default_initial_world_spawn_dimension_id() -> String {
    String::new()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorldSaveMetadata {
    pub format_version: u32,
    pub world_id: String,
    pub seed: u64,
    pub bound_experience_id: String,
    pub primary_dimension_id: String,
    #[serde(default)]
    pub initial_world_spawn: Option<InitialWorldSpawn>,
}

impl WorldSaveMetadata {
    #[must_use]
    pub fn new(
        world_id: String,
        seed: u64,
        bound_experience_id: String,
        primary_dimension_id: String,
    ) -> Self {
        Self {
            format_version: WORLD_SAVE_FORMAT_VERSION,
            world_id,
            seed,
            bound_experience_id,
            primary_dimension_id,
            initial_world_spawn: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DimensionSaveMetadata {
    pub format_version: u32,
    pub dimension_id: String,
    #[serde(default)]
    pub vertical_contract: Option<DimensionVerticalContract>,
}

impl DimensionSaveMetadata {
    #[must_use]
    pub fn new(
        dimension_id: impl Into<String>,
        vertical_contract: DimensionVerticalContract,
    ) -> Self {
        Self {
            format_version: DIMENSION_SAVE_FORMAT_VERSION,
            dimension_id: dimension_id.into(),
            vertical_contract: Some(vertical_contract),
        }
    }

    #[must_use]
    pub fn primary() -> Self {
        Self::new(
            DEFAULT_PRIMARY_DIMENSION_ID,
            DimensionVerticalContract::primary_default(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldBootstrapSpec {
    pub world_id: String,
    pub seed: Option<u64>,
    pub bound_experience_id: String,
    pub primary_dimension_id: String,
    pub dimensions: Vec<DimensionLayoutSpec>,
}

impl WorldBootstrapSpec {
    #[must_use]
    pub fn single_primary_dimension(
        world_id: impl Into<String>,
        bound_experience_id: impl Into<String>,
    ) -> Self {
        Self {
            world_id: world_id.into(),
            seed: None,
            bound_experience_id: bound_experience_id.into(),
            primary_dimension_id: DEFAULT_PRIMARY_DIMENSION_ID.to_string(),
            dimensions: vec![DimensionLayoutSpec::primary()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DimensionLayoutSpec {
    pub dimension_id: String,
    pub vertical_contract: DimensionVerticalContract,
}

impl DimensionLayoutSpec {
    #[must_use]
    pub fn new(
        dimension_id: impl Into<String>,
        vertical_contract: DimensionVerticalContract,
    ) -> Self {
        Self {
            dimension_id: dimension_id.into(),
            vertical_contract,
        }
    }

    #[must_use]
    pub fn primary() -> Self {
        Self::new(
            DEFAULT_PRIMARY_DIMENSION_ID,
            DimensionVerticalContract::primary_default(),
        )
    }
}

fn meters_to_mm(value: f32) -> i32 {
    let mm = (value * MM_PER_METER).round();
    let bounded = mm.clamp(i32::MIN as f32, i32::MAX as f32);
    bounded as i32
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_PRIMARY_DIMENSION_ID, DimensionLayoutSpec, DimensionSaveMetadata,
        DimensionVerticalContract, InitialWorldSpawn, WorldSaveMetadata,
    };

    #[test]
    fn initial_world_spawn_meter_roundtrip_is_mm_quantized() {
        let spawn = InitialWorldSpawn::from_feet_position_meters([12.3456, 64.0004, -7.8912]);
        assert_eq!(spawn.feet_position_mm, [12346, 64000, -7891]);
        assert_eq!(spawn.dimension_id, DEFAULT_PRIMARY_DIMENSION_ID);
        assert_eq!(spawn.feet_position_meters(), [12.346, 64.0, -7.891]);
    }

    #[test]
    fn world_save_metadata_defaults_to_unresolved_initial_spawn() {
        let metadata = WorldSaveMetadata::new(
            "world_0".to_string(),
            7,
            "exp.main".to_string(),
            "overworld".to_string(),
        );
        assert!(metadata.initial_world_spawn.is_none());
    }

    #[test]
    fn dimension_metadata_primary_includes_explicit_vertical_contract() {
        let metadata = DimensionSaveMetadata::primary();
        assert_eq!(
            metadata.vertical_contract,
            Some(DimensionVerticalContract::primary_default())
        );
    }

    #[test]
    fn dimension_layout_primary_includes_explicit_vertical_contract() {
        let layout = DimensionLayoutSpec::primary();
        assert_eq!(
            layout.vertical_contract,
            DimensionVerticalContract::primary_default()
        );
    }
}
