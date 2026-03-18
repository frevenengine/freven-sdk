use serde::{Deserialize, Serialize};

pub const WORLD_SAVE_FORMAT_VERSION: u32 = 1;
pub const DIMENSION_SAVE_FORMAT_VERSION: u32 = 1;
pub const DEFAULT_PRIMARY_DIMENSION_ID: &str = "overworld";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorldSaveMetadata {
    pub format_version: u32,
    pub world_id: String,
    pub seed: u64,
    pub bound_experience_id: String,
    pub primary_dimension_id: String,
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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DimensionSaveMetadata {
    pub format_version: u32,
    pub dimension_id: String,
}

impl DimensionSaveMetadata {
    #[must_use]
    pub fn new(dimension_id: impl Into<String>) -> Self {
        Self {
            format_version: DIMENSION_SAVE_FORMAT_VERSION,
            dimension_id: dimension_id.into(),
        }
    }

    #[must_use]
    pub fn primary() -> Self {
        Self::new(DEFAULT_PRIMARY_DIMENSION_ID)
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
}

impl DimensionLayoutSpec {
    #[must_use]
    pub fn primary() -> Self {
        Self {
            dimension_id: DEFAULT_PRIMARY_DIMENSION_ID.to_string(),
        }
    }
}
