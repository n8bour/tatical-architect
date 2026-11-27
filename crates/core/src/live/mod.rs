//! Live FM27 bridge integration (`TacticalBibleBridge`, `127.0.0.1:27029`)
//! and 6-phase animated tactical pitch simulation model.

pub mod fmbridge;
pub mod sim_model;

pub use fmbridge::{ApplyTacticOptions, FmbClient, FmbIds, TacticPiSpec, TacticRoleSpec, TacticTiSpec};
