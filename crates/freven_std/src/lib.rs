//! Optional standard gameplay helpers built on top of `freven_api`.

pub mod action_defaults;
pub mod action_payloads;
pub mod humanoid_input;

/// Convenience re-exports for common client-facing helper enums.
///
/// These remain secondary to the opaque input/action payload contracts.
pub mod client_input {
    pub use freven_api::{ClientBlockFace, ClientKeyCode, ClientMouseButton};
}
