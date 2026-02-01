//! Error handling module for SilentAlliance
//!
//! This module provides unified error types and handling for the entire application.
//! All errors are converted to appropriate HTTP responses with consistent formatting.

mod api_error;

pub use api_error::*;
