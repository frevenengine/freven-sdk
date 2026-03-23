//! Explicit avatar-owned public SDK facade.
//!
//! This crate is the public owner for avatar identity, control, controller,
//! and presentation contracts. Registration-facing helpers live here so the
//! owner layer is visible at import sites.

pub mod control;
pub mod controller;
pub mod identity;
pub mod lifecycle;
pub mod presentation;

pub use control::*;
pub use controller::*;
pub use identity::*;
pub use lifecycle::*;
pub use presentation::*;
