//! Public gameplay-state identity and value contracts for Freven.
//!
//! Ownership:
//! - stable gameplay-state owner/scope identity
//! - namespaced state keys
//! - opaque/versioned state values
//! - public persistence and replication policy vocabulary
//!
//! Non-responsibilities:
//! - runtime storage implementation
//! - save/load lifecycle implementation
//! - network replication implementation
//! - Vanilla inventory/hotbar semantics
//! - block/entity actor lifecycle semantics
//!
//! This crate defines the reusable public vocabulary. Gameplay layers own the
//! schema and meaning of their state values. Engine/runtime layers own authority,
//! isolation, lifecycle, limits, diagnostics, persistence, and replication policy.

extern crate alloc;

use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

pub const MAX_GAMEPLAY_STATE_NAMESPACE_LEN: usize = 128;
pub const MAX_GAMEPLAY_STATE_KEY_LEN: usize = 128;
pub const MAX_GAMEPLAY_STATE_CODEC_LEN: usize = 64;
pub const MAX_GAMEPLAY_STATE_OBJECT_ID_LEN: usize = 256;
pub const DEFAULT_MAX_GAMEPLAY_STATE_VALUE_BYTES: usize = 64 * 1024;

/// Stable owner/scope identity for gameplay state.
///
/// This is intentionally broader than Vanilla inventory or blocks. The engine
/// may support only a subset in a given phase, but the public identity model is
/// shaped so later player/entity/world/object state can share one vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum GameplayStateOwner {
    Player {
        player_id: u64,
    },
    Entity {
        entity_id: u64,
    },
    World {
        world_id: String,
    },
    Level {
        level_id: u32,
    },
    WorldPosition {
        level_id: u32,
        pos: (i32, i32, i32),
    },
    Chunk {
        level_id: u32,
        cx: i32,
        cz: i32,
    },
    Object {
        namespace: String,
        object_id: String,
    },
}

/// Fully qualified gameplay-state key.
///
/// `namespace` identifies the gameplay/mod owner, for example
/// `freven.vanilla` or `example.mod`.
///
/// `key` identifies the state inside that namespace, for example
/// `selected_slot`, `hotbar`, `mask_filter`, or `weapon_loadout`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GameplayStateKey {
    pub owner: GameplayStateOwner,
    pub namespace: String,
    pub key: String,
}

impl GameplayStateKey {
    pub fn new(
        owner: GameplayStateOwner,
        namespace: impl Into<String>,
        key: impl Into<String>,
    ) -> Result<Self, GameplayStateValidationError> {
        let value = Self {
            owner,
            namespace: namespace.into(),
            key: key.into(),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), GameplayStateValidationError> {
        validate_identifier(
            "namespace",
            &self.namespace,
            MAX_GAMEPLAY_STATE_NAMESPACE_LEN,
        )?;
        validate_identifier("key", &self.key, MAX_GAMEPLAY_STATE_KEY_LEN)?;
        self.owner.validate()
    }
}

impl GameplayStateOwner {
    pub fn validate(&self) -> Result<(), GameplayStateValidationError> {
        match self {
            Self::Object {
                namespace,
                object_id,
            } => {
                validate_identifier(
                    "object_namespace",
                    namespace,
                    MAX_GAMEPLAY_STATE_NAMESPACE_LEN,
                )?;
                validate_non_empty_bounded("object_id", object_id, MAX_GAMEPLAY_STATE_OBJECT_ID_LEN)
            }
            Self::World { world_id } => {
                validate_non_empty_bounded("world_id", world_id, MAX_GAMEPLAY_STATE_OBJECT_ID_LEN)
            }
            Self::Player { .. }
            | Self::Entity { .. }
            | Self::Level { .. }
            | Self::WorldPosition { .. }
            | Self::Chunk { .. } => Ok(()),
        }
    }
}

/// Payload codec marker for an opaque gameplay-state value.
///
/// The engine does not interpret gameplay semantics. The marker is for
/// diagnostics, migration, and tooling.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum GameplayStateCodec {
    #[default]
    OpaqueBytes,
    Postcard,
    Json,
    Custom(String),
}

impl GameplayStateCodec {
    pub fn validate(&self) -> Result<(), GameplayStateValidationError> {
        match self {
            Self::Custom(codec) => {
                validate_identifier("codec", codec, MAX_GAMEPLAY_STATE_CODEC_LEN)
            }
            Self::OpaqueBytes | Self::Postcard | Self::Json => Ok(()),
        }
    }
}

/// Opaque/versioned gameplay-state value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GameplayStateValue {
    pub version: u32,
    pub codec: GameplayStateCodec,
    pub bytes: Vec<u8>,
}

impl GameplayStateValue {
    pub fn new(
        version: u32,
        codec: GameplayStateCodec,
        bytes: impl Into<Vec<u8>>,
    ) -> Result<Self, GameplayStateValidationError> {
        let value = Self {
            version,
            codec,
            bytes: bytes.into(),
        };
        value.validate(DEFAULT_MAX_GAMEPLAY_STATE_VALUE_BYTES)?;
        Ok(value)
    }

    pub fn validate(&self, max_bytes: usize) -> Result<(), GameplayStateValidationError> {
        self.codec.validate()?;
        if self.bytes.len() > max_bytes {
            return Err(GameplayStateValidationError::ValueTooLarge {
                len: self.bytes.len(),
                max: max_bytes,
            });
        }
        Ok(())
    }
}

/// Persistence policy requested for a gameplay-state value.
///
/// Engine/runtime support may initially accept only `Transient`; the public
/// vocabulary is intentionally wider so later persistence work does not need a
/// new identity model.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum GameplayStatePersistencePolicy {
    #[default]
    Transient,
    PlayerProfile,
    WorldSave,
    InstanceSave,
}

/// Replication policy requested for a gameplay-state value.
///
/// `ServerOnly` is the safest default. More visible policies require explicit
/// host/runtime support.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum GameplayStateReplicationPolicy {
    #[default]
    ServerOnly,
    OwningClient,
    Observers,
    Public,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(default)]
pub struct GameplayStatePolicy {
    pub persistence: GameplayStatePersistencePolicy,
    pub replication: GameplayStateReplicationPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum GameplayStateValidationError {
    EmptyField {
        field: &'static str,
    },
    FieldTooLong {
        field: &'static str,
        len: usize,
        max: usize,
    },
    InvalidIdentifierChar {
        field: &'static str,
        ch: char,
    },
    ValueTooLarge {
        len: usize,
        max: usize,
    },
}

impl core::fmt::Display for GameplayStateValidationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyField { field } => write!(f, "{field} must not be empty"),
            Self::FieldTooLong { field, len, max } => {
                write!(f, "{field} length {len} exceeds maximum {max}")
            }
            Self::InvalidIdentifierChar { field, ch } => {
                write!(f, "{field} contains invalid character {ch:?}")
            }
            Self::ValueTooLarge { len, max } => {
                write!(f, "gameplay-state value length {len} exceeds maximum {max}")
            }
        }
    }
}

impl std::error::Error for GameplayStateValidationError {}

fn validate_non_empty_bounded(
    field: &'static str,
    value: &str,
    max: usize,
) -> Result<(), GameplayStateValidationError> {
    if value.is_empty() {
        return Err(GameplayStateValidationError::EmptyField { field });
    }
    if value.len() > max {
        return Err(GameplayStateValidationError::FieldTooLong {
            field,
            len: value.len(),
            max,
        });
    }
    Ok(())
}

fn validate_identifier(
    field: &'static str,
    value: &str,
    max: usize,
) -> Result<(), GameplayStateValidationError> {
    validate_non_empty_bounded(field, value, max)?;
    for ch in value.chars() {
        if !(ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '_' | '-' | '.')) {
            return Err(GameplayStateValidationError::InvalidIdentifierChar { field, ch });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gameplay_state_key_accepts_mod_namespace_and_state_key() {
        let key = GameplayStateKey::new(
            GameplayStateOwner::Player { player_id: 7 },
            "freven.vanilla",
            "selected_slot",
        )
        .expect("valid key");

        assert_eq!(key.namespace, "freven.vanilla");
        assert_eq!(key.key, "selected_slot");
    }

    #[test]
    fn gameplay_state_key_rejects_invalid_namespace() {
        let err = GameplayStateKey::new(
            GameplayStateOwner::Player { player_id: 7 },
            "Freven.Vanilla",
            "selected_slot",
        )
        .expect_err("uppercase namespace should be rejected");

        assert!(matches!(
            err,
            GameplayStateValidationError::InvalidIdentifierChar {
                field: "namespace",
                ..
            }
        ));
    }

    #[test]
    fn gameplay_state_value_enforces_size_limit() {
        let value = GameplayStateValue {
            version: 1,
            codec: GameplayStateCodec::OpaqueBytes,
            bytes: vec![0; 4],
        };

        assert_eq!(
            value.validate(3),
            Err(GameplayStateValidationError::ValueTooLarge { len: 4, max: 3 })
        );
    }
}
