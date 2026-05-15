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

/// A single cell edit carried by a bulk block mutation.
///
/// `expected_old` keeps the same compare-and-set meaning as scalar `SetBlock`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockEdit {
    pub pos: (i32, i32, i32),
    pub block_id: BlockRuntimeId,
    pub expected_old: Option<BlockRuntimeId>,
}

/// Replacement policy for region-style block mutations.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BlockReplacePolicy {
    /// Replace any currently loaded block.
    Any,
    /// Replace only air blocks.
    OnlyAir,
    /// Replace only blocks matching the expected runtime id.
    Matching(BlockRuntimeId),
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
    /// Apply a compact list of independent cell edits.
    ///
    /// Hosts may normalize, de-duplicate, budget, and group these edits by
    /// chunk/section before applying them, but the semantic payload remains a
    /// deterministic list of compare-and-set cell edits.
    SetBlocks { edits: Vec<BlockEdit> },
    /// Fill a half-open world-cell box: `[min, max)`.
    ///
    /// This intentionally matches the half-open bounds used by
    /// `WorldTerrainWrite::FillBox` so runtime mutations and worldgen writes use
    /// the same spatial convention.
    FillBox {
        min: (i32, i32, i32),
        max: (i32, i32, i32),
        block_id: BlockRuntimeId,
        replace: BlockReplacePolicy,
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
    AuthoritativeBlock {
        pos: (i32, i32, i32),
    },
    BlockIdByKey {
        key: String,
    },
    /// Return whether `block_id` is a member of the semantic block tag.
    ///
    /// `tag_key` is a namespaced key such as `freven:stones` or
    /// `modid:gas_permeable`. The host owns the resolved tag registry; this
    /// query is only the transport-neutral SDK contract.
    BlockHasTag {
        block_id: BlockRuntimeId,
        tag_key: String,
    },
    /// Return all runtime block ids currently resolved for a semantic block tag.
    BlocksWithTag {
        tag_key: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlockQueryResponse {
    AuthoritativeBlock(Option<BlockRuntimeId>),
    BlockIdByKey(Option<BlockRuntimeId>),
    BlockHasTag(bool),
    BlocksWithTag(Vec<BlockRuntimeId>),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bulk_block_mutations_are_transport_neutral_contract_shapes() {
        let block_id = BlockRuntimeId(7);
        let edit = BlockEdit {
            pos: (1, 2, 3),
            block_id,
            expected_old: Some(BlockRuntimeId(2)),
        };

        assert_eq!(
            BlockMutation::SetBlocks {
                edits: vec![edit.clone()]
            },
            BlockMutation::SetBlocks { edits: vec![edit] }
        );
        assert_eq!(
            BlockMutation::FillBox {
                min: (0, 0, 0),
                max: (16, 16, 16),
                block_id,
                replace: BlockReplacePolicy::OnlyAir,
            },
            BlockMutation::FillBox {
                min: (0, 0, 0),
                max: (16, 16, 16),
                block_id,
                replace: BlockReplacePolicy::OnlyAir,
            }
        );
        assert_eq!(
            BlockReplacePolicy::Matching(block_id),
            BlockReplacePolicy::Matching(block_id)
        );
    }

    #[test]
    fn block_tag_queries_are_transport_neutral_contract_shapes() {
        let block_id = BlockRuntimeId(7);
        let tag_key = "freven:stones".to_string();

        let has_tag = BlockQueryRequest::BlockHasTag {
            block_id,
            tag_key: tag_key.clone(),
        };
        assert_eq!(
            has_tag,
            BlockQueryRequest::BlockHasTag {
                block_id,
                tag_key: tag_key.clone()
            }
        );

        let blocks = BlockQueryResponse::BlocksWithTag(vec![block_id]);
        assert_eq!(blocks, BlockQueryResponse::BlocksWithTag(vec![block_id]));
        assert_eq!(
            BlockQueryResponse::BlockHasTag(true),
            BlockQueryResponse::BlockHasTag(true)
        );
    }
}
