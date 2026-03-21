//! Canonical public runtime-loaded block gameplay contract.
//!
//! Ownership:
//! - block/profile vocabulary lives in `freven_block_sdk_types`
//! - runtime-loaded block gameplay/query/mutation contracts live here

extern crate alloc;

use alloc::{string::String, vec::Vec};

use freven_block_sdk_types::BlockRuntimeId;
use serde::{Deserialize, Serialize};

/// Runtime output block mutations.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct BlockMutationBatch {
    pub mutations: Vec<BlockMutation>,
}

impl BlockMutationBatch {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.mutations.is_empty()
    }
}

/// Runtime output block mutations.
///
/// This is standard block gameplay/runtime vocabulary, not generic world truth.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BlockMutation {
    SetBlock {
        pos: (i32, i32, i32),
        block_id: BlockRuntimeId,
        expected_old: Option<BlockRuntimeId>,
    },
}

impl BlockMutation {
    #[must_use]
    pub const fn clear_block(pos: (i32, i32, i32), expected_old: Option<BlockRuntimeId>) -> Self {
        Self::SetBlock {
            pos,
            block_id: BlockRuntimeId(0),
            expected_old,
        }
    }
}

/// Authoritative/runtime block query family.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlockQueryRequest {
    AuthoritativeBlock { pos: (i32, i32, i32) },
    BlockIdByKey { key: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlockQueryResponse {
    AuthoritativeBlock(Option<BlockRuntimeId>),
    BlockIdByKey(Option<BlockRuntimeId>),
}

/// Client-visible block query family.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlockClientQueryRequest {
    ClientVisibleBlock { pos: (i32, i32, i32) },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlockClientQueryResponse {
    ClientVisibleBlock(Option<BlockRuntimeId>),
}

/// Block-specific runtime service family carried inside the generic world service envelope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlockServiceRequest {
    Query(BlockQueryRequest),
    ClientQuery(BlockClientQueryRequest),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlockServiceResponse {
    Query(BlockQueryResponse),
    ClientQuery(BlockClientQueryResponse),
}
