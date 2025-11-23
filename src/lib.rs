pub mod domain;
pub mod parser;

// Re-export for convenience
pub use domain::{
    Email, FrumpDoc, Property, PropertyKey, Task, TaskCollection, TaskId, TaskType, Team,
    TeamMember,
};
