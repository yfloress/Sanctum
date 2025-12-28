//! UI Callback Setup Modules
//!
//! Domain-specific callback registration for Slint UI.
//! Each module exports a `setup_*_callbacks()` function.

pub mod finance;
// pub mod dashboard;  // TODO: Extract
// pub mod habits;     // TODO: Extract
// pub mod crypto;     // TODO: Extract

pub use finance::*;
