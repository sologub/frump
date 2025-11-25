use anyhow::{anyhow, Result};
use std::fmt;

/// Represents a unique task identifier
/// Task IDs are positive integers that must be unique across the entire history
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskId(u32);

impl TaskId {
    /// Create a new TaskId, validating that it's positive
    pub fn new(id: u32) -> Result<Self> {
        if id == 0 {
            return Err(anyhow!("Task ID must be positive (got 0)"));
        }
        Ok(TaskId(id))
    }

    /// Get the next sequential task ID
    pub fn next(&self) -> TaskId {
        TaskId(self.0 + 1)
    }

    /// Get the raw u32 value
    pub fn value(&self) -> u32 {
        self.0
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<TaskId> for u32 {
    fn from(id: TaskId) -> u32 {
        id.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_task_id() {
        let id = TaskId::new(1).unwrap();
        assert_eq!(id.value(), 1);
    }

    #[test]
    fn test_zero_task_id_fails() {
        assert!(TaskId::new(0).is_err());
    }

    #[test]
    fn test_task_id_next() {
        let id = TaskId::new(5).unwrap();
        let next = id.next();
        assert_eq!(next.value(), 6);
    }

    #[test]
    fn test_task_id_ordering() {
        let id1 = TaskId::new(1).unwrap();
        let id2 = TaskId::new(2).unwrap();
        assert!(id1 < id2);
    }
}
