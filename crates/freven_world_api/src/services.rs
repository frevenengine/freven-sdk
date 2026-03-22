use freven_block_guest::BlockMutationBatch;

use crate::observability::HostLogRecord;

pub use freven_world_guest::{
    ClientVisibilityRequest, ClientVisibilityResponse, RuntimeCharacterPhysicsRequest,
    RuntimeClientControlRequest, RuntimeEntityTarget, RuntimeLevelRef, RuntimeObservabilityRequest,
    RuntimeOutput, WorldQueryRequest, WorldQueryResponse, WorldServiceRequest,
    WorldServiceResponse, WorldSessionRequest, WorldSessionResponse,
};

/// Runtime-provided services exposed to SDK hooks.
pub trait Services {
    fn world_service(&mut self, _request: &WorldServiceRequest) -> WorldServiceResponse {
        WorldServiceResponse::Unsupported
    }

    fn apply_block_mutations(
        &mut self,
        mutations: &BlockMutationBatch,
    ) -> Result<(), RuntimeOutputApplyError> {
        if mutations.is_empty() {
            Ok(())
        } else {
            Err(RuntimeOutputApplyError::UnsupportedFamily {
                family: "block_mutations",
            })
        }
    }

    fn record_guest_log(&mut self, _record: &HostLogRecord) {}
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum RuntimeOutputApplyError {
    #[error("runtime output family '{family}' is not supported in this host context")]
    UnsupportedFamily { family: &'static str },
    #[error("runtime output application failed: {message}")]
    Rejected { message: String },
}

/// Empty services implementation used by runtimes that do not expose services yet.
#[derive(Debug, Default)]
pub struct NoServices;

impl Services for NoServices {}
