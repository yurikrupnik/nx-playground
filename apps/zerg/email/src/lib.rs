//! Zerg Email Worker Service
//!
//! Event-driven email processing service using Redis Streams.

pub mod config;
pub mod handlers;
pub mod state;
pub mod worker;
