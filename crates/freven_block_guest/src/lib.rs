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
