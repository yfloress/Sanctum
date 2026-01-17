//! UI Callback Setup Modules
//!
//! Domain-specific callback registration for Slint UI.
//! Each module exports a `setup_*_callbacks()` function.

pub mod crypto;
pub mod dashboard;
pub mod finance;
pub mod habits;
pub mod ingestion;
pub mod settings;
pub mod translations;
pub mod vault;

pub use crypto::*;
pub use dashboard::*;
pub use finance::*;
pub use habits::*;
pub use ingestion::*;
pub use settings::*;
pub use translations::*;
pub use vault::*;
