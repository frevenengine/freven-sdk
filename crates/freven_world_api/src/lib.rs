//! Stable SDK contracts for Freven builtin / compile-time mod authoring.
//!
//! Responsibilities:
//! - define experience/mod descriptors used by boot/runtime layers
//! - expose deterministic registration surfaces (components/messages/worldgen/modnet/capabilities)
//! - define stable hook contexts and registration errors
//! - act as the builtin / compile-time facade over the canonical declaration model exposed by `freven_guest`
//! - import standard block/profile vocabulary from `freven_block_sdk_types` instead of owning it here
//! - remain free of avatar/controller/presentation public ownership, which lives in the avatar-owned family
//! - delegate volumetric worldgen provider traits to `freven_volumetric_api` so the topology-owned layer retains public ownership
//!
//! Current state note:
//! - some world-stack consumer contracts in this crate still reference block/profile vocabulary
//! - that does not make `freven_world_api` the owner of `BlockRuntimeId`, `BlockDescriptor`, or `RenderLayer`
//! - dedicated block gameplay contract extraction can happen later without moving block/profile vocabulary out of `freven_block_sdk_types`
//!
//! Extension guidance:
//! - add new registries behind stable string keys
//! - keep hook/context types engine-agnostic
//! - avoid leaking runtime/transport implementation details
//! - do not redefine or re-export standard block/profile vocabulary from this crate

pub mod action;
pub mod client;
pub mod lifecycle;
pub mod messages;
pub mod observability;
pub mod registration;
pub mod services;
pub use action::*;
pub use client::*;
pub use lifecycle::*;
pub use messages::*;
pub use observability::*;
pub use registration::*;
pub use services::*;
