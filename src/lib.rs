// src/lib.rs - Library root for integration tests
// This file makes our modules available to integration tests

// Re-export modules for external use (like integration tests)
pub mod todo;
pub mod storage;
pub mod cli;
pub mod handlers;
pub mod tui;
