//! Application state management module.
//!
//! This module contains the core state management for the application, including:
//! - Main `State` struct that holds all application data
//! - Navigation types (View, Focus, Menu, etc.)
//! - Form editing types (CustomFieldValue, EditFormState, etc.)
//! - State error handling

mod error;
mod form;
mod navigation;

pub use error::StateError;
pub use form::{CustomFieldValue, EditFormState};
pub use navigation::{Focus, Menu, SearchTarget, TaskDetailPanel, View, ViewMode};

// Re-export implementation from state_impl.rs
// State struct, methods and Default impl are in state_impl.rs
#[path = "state_impl.rs"]
mod state_impl;

// Re-export State
pub use state_impl::State;
