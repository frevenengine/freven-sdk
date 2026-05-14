//! Canonical public runtime-loaded gameplay-state contract.
//!
//! Ownership:
//! - identity/value vocabulary lives in `freven_gameplay_state_sdk_types`
//! - runtime-loaded gameplay-state query/mutation shapes live here
//!
//! This crate does not define Vanilla inventory semantics. It only defines the
//! generic runtime-facing carrier family that hosts can authorize and apply.

extern crate alloc;

use alloc::vec::Vec;

use freven_gameplay_state_sdk_types::{GameplayStateKey, GameplayStatePolicy, GameplayStateValue};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct GameplayStateMutationBatch {
    pub mutations: Vec<GameplayStateMutation>,
}

impl GameplayStateMutationBatch {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.mutations.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum GameplayStateMutation {
    Set {
        key: GameplayStateKey,
        value: GameplayStateValue,
        policy: GameplayStatePolicy,
    },
    Delete {
        key: GameplayStateKey,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum GameplayStateQueryRequest {
    Get { key: GameplayStateKey },
    Exists { key: GameplayStateKey },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum GameplayStateQueryResponse {
    Get(Option<GameplayStateValue>),
    Exists(bool),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum GameplayStateServiceRequest {
    Query(GameplayStateQueryRequest),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum GameplayStateServiceResponse {
    Query(GameplayStateQueryResponse),
}

#[cfg(test)]
mod tests {
    use super::*;
    use freven_gameplay_state_sdk_types::{GameplayStateCodec, GameplayStateOwner};

    fn postcard_roundtrip<T>(value: &T) -> T
    where
        T: Clone + PartialEq + core::fmt::Debug + serde::Serialize + serde::de::DeserializeOwned,
    {
        let bytes = postcard::to_allocvec(value).expect("postcard encode");
        let decoded = postcard::from_bytes(&bytes).expect("postcard decode");
        assert_eq!(decoded, *value);
        decoded
    }

    #[test]
    fn mutation_batch_empty_matches_payload_state() {
        let key = GameplayStateKey::new(
            GameplayStateOwner::Player { player_id: 1 },
            "example.mod",
            "loadout",
        )
        .expect("valid key");

        let mut batch = GameplayStateMutationBatch::default();
        assert!(batch.is_empty());

        batch.mutations.push(GameplayStateMutation::Set {
            key,
            value: GameplayStateValue::new(1, GameplayStateCodec::OpaqueBytes, [1, 2, 3])
                .expect("valid value"),
            policy: GameplayStatePolicy::default(),
        });

        assert!(!batch.is_empty());
        postcard_roundtrip(&batch);
    }

    #[test]
    fn query_request_roundtrips() {
        let key = GameplayStateKey::new(
            GameplayStateOwner::Player { player_id: 9 },
            "freven.vanilla",
            "selected_slot",
        )
        .expect("valid key");

        postcard_roundtrip(&GameplayStateServiceRequest::Query(
            GameplayStateQueryRequest::Get { key },
        ));
    }
}
