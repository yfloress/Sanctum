//! Sanctum - Personal Finance Manager
//!
//! Core library modules for database, models, and business logic.

pub mod controller;
pub mod crypto;
pub mod db;
pub mod models;
pub mod security_log;
pub mod services;

pub fn init_logger() {
    // Initialize logger if needed
    // env_logger::init();
}
