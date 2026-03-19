//! Stable SDK contracts for Freven builtin / compile-time mod authoring.
//!
//! Responsibilities:
//! - define experience/mod descriptors used by boot/runtime layers
//! - expose deterministic registration surfaces (components/messages/worldgen/modnet/capabilities)
//! - define stable hook contexts and registration errors
//! - act as the builtin / compile-time facade over the canonical declaration model exposed by `freven_guest`
//! - import volumetric topology/addressing truth from `freven_volumetric_sdk_types` instead of owning it here
//!
//! Extension guidance:
//! - add new registries behind stable string keys
//! - keep hook/context types engine-agnostic
//! - avoid leaking runtime/transport implementation details

pub mod action;
pub mod character;
pub mod client;
pub mod lifecycle;
pub mod messages;
pub mod observability;
pub mod registration;
pub mod services;
pub mod worldgen;

pub use action::*;
pub use character::*;
pub use client::*;
pub use lifecycle::*;
pub use messages::*;
pub use observability::*;
pub use registration::*;
pub use services::*;
pub use worldgen::*;
