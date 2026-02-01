//! Middleware modules for cross-cutting concerns
//!
//! This module contains middleware for handling concerns that apply
//! to all or most HTTP requests, such as:
//! - Logging and observability
//! - Metrics collection
//! - Request ID generation
//! - CORS handling (already in server module)
//! - Rate limiting (future)
//! - Authentication (handled by extractors)

pub mod logging;
