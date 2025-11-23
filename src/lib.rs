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
