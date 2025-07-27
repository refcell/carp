//! # Carp CLI
//!
//! Command-line tool for the Claude Agent Registry Portal.
//!
//! This crate provides functionality to search, pull, publish, and create
//! Claude AI agents from the Carp registry.

pub mod api;
pub mod auth;
pub mod commands;
pub mod config;
pub mod utils;

pub use utils::error::{CarpError, CarpResult};
