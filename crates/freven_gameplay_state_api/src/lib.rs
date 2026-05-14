//! Compile-time / builtin-facing gameplay-state authority contracts.
//!
//! Ownership:
//! - identity/value vocabulary lives in `freven_gameplay_state_sdk_types`
//! - runtime-loaded mutation/query shapes live in `freven_gameplay_state_guest`
//! - builtin/compile-time authority and view traits live here

use freven_gameplay_state_guest::GameplayStateMutation;
use freven_gameplay_state_sdk_types::{GameplayStateKey, GameplayStatePolicy, GameplayStateValue};

pub trait GameplayStateView {
    fn gameplay_state(&self, key: &GameplayStateKey) -> Option<GameplayStateValue>;

    fn contains_gameplay_state(&self, key: &GameplayStateKey) -> bool {
        self.gameplay_state(key).is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum GameplayStateMutationResult {
    Applied,
    Deleted,
    Missing,
    UnsupportedOwner,
    UnsupportedPolicy,
    Rejected { message: String },
}

pub trait GameplayStateAuthority: GameplayStateView {
    fn try_apply_gameplay_state(
        &mut self,
        mutation: &GameplayStateMutation,
    ) -> GameplayStateMutationResult;

    fn set_gameplay_state(
        &mut self,
        key: GameplayStateKey,
        value: GameplayStateValue,
        policy: GameplayStatePolicy,
    ) -> GameplayStateMutationResult {
        self.try_apply_gameplay_state(&GameplayStateMutation::Set { key, value, policy })
    }

    fn delete_gameplay_state(&mut self, key: GameplayStateKey) -> GameplayStateMutationResult {
        self.try_apply_gameplay_state(&GameplayStateMutation::Delete { key })
    }
}
