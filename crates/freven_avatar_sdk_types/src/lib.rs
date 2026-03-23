//! Explicit avatar-owned SDK contracts.
//!
//! This crate owns public contracts that are not world-neutral:
//! - avatar identity
//! - client control input and control-provider contracts
//! - controller-facing movement contracts
//! - client presentation views over avatar/player state

pub mod control;
pub mod controller;
pub mod identity;
pub mod presentation;

pub use control::*;
pub use controller::*;
pub use identity::*;
pub use presentation::*;
