// SPDX-License-Identifier: MIT

//! # Frump - Distributed Task Management
//!
//! Frump is a Git-based task management tool that uses Markdown files for task storage.
//! This crate provides the core library functionality for parsing, manipulating, and
//! managing Frump documents.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use frump::{parser, Task, TaskId, TaskType};
//!
//! // Parse a frump.md file
//! let content = std::fs::read_to_string("frump.md").unwrap();
//! let doc = parser::parse(&content).unwrap();
//!
//! // Work with tasks
//! for task in doc.tasks.tasks() {
//!     println!("{} - {}", task.id, task.subject);
//! }
//!
//! // Create a new task
//! let task = Task::new(
//!     TaskId::new(1).unwrap(),
//!     TaskType::Task,
//!     "My task".to_string()
//! );
//! ```
//!
//! ## Modules
//!
//! - [`domain`]: Core domain types (Task, Team, Properties)
//! - [`parser`]: Markdown parsing and serialization
//! - [`git`]: Git history integration
//! - [`export`]: JSON/CSV import/export
//! - [`templates`]: Task template management

pub mod domain;
pub mod export;
pub mod git;
pub mod parser;
pub mod templates;

// Re-export for convenience
pub use domain::{
    Email, FrumpDoc, Property, PropertyKey, Task, TaskCollection, TaskId, TaskType, Team,
    TeamMember,
};
pub use export::{export_csv, export_json, import_json};
pub use git::{ChangeType, FrumpRepo, TaskCommit, TaskHistory};
pub use templates::{TaskTemplate, TemplateManager};
