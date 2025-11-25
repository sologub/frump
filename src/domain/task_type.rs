use std::fmt;

/// Common task types with extensibility for custom types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskType {
    Task,
    Bug,
    Issue,
    Feature,
    Custom(String),
}

impl TaskType {
    /// Parse a task type from a string
    pub fn parse(s: &str) -> Self {
        match s {
            "Task" => TaskType::Task,
            "Bug" => TaskType::Bug,
            "Issue" => TaskType::Issue,
            "Feature" => TaskType::Feature,
            _ => TaskType::Custom(s.to_string()),
        }
    }

    /// Get the string representation
    pub fn as_str(&self) -> &str {
        match self {
            TaskType::Task => "Task",
            TaskType::Bug => "Bug",
            TaskType::Issue => "Issue",
            TaskType::Feature => "Feature",
            TaskType::Custom(s) => s,
        }
    }
}

impl fmt::Display for TaskType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<String> for TaskType {
    fn from(s: String) -> Self {
        TaskType::parse(&s)
    }
}

impl From<&str> for TaskType {
    fn from(s: &str) -> Self {
        TaskType::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_types() {
        assert_eq!(TaskType::parse("Task"), TaskType::Task);
        assert_eq!(TaskType::parse("Bug"), TaskType::Bug);
        assert_eq!(TaskType::parse("Issue"), TaskType::Issue);
        assert_eq!(TaskType::parse("Feature"), TaskType::Feature);
    }

    #[test]
    fn test_custom_type() {
        let custom = TaskType::parse("Enhancement");
        assert_eq!(custom, TaskType::Custom("Enhancement".to_string()));
    }

    #[test]
    fn test_display() {
        assert_eq!(TaskType::Task.to_string(), "Task");
        assert_eq!(TaskType::Bug.to_string(), "Bug");
        assert_eq!(TaskType::Custom("Test".to_string()).to_string(), "Test");
    }
}
