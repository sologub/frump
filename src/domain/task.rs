use super::property::{Property, PropertyKey};
use super::task_id::TaskId;
use super::task_type::TaskType;

/// Represents a task with all its metadata
#[derive(Debug, Clone)]
pub struct Task {
    pub id: TaskId,
    pub task_type: TaskType,
    pub subject: String,
    pub body: String,
    pub properties: Vec<Property>,
}

impl Task {
    /// Create a new task with minimal required fields
    pub fn new(id: TaskId, task_type: TaskType, subject: String) -> Self {
        Task {
            id,
            task_type,
            subject,
            body: String::new(),
            properties: Vec::new(),
        }
    }

    /// Builder method to set the body
    pub fn with_body(mut self, body: String) -> Self {
        self.body = body;
        self
    }

    /// Set the body
    pub fn set_body(&mut self, body: String) {
        self.body = body;
    }

    /// Add a property to the task
    pub fn add_property(&mut self, key: PropertyKey, value: String) {
        self.properties.push(Property::new(key, value));
    }

    /// Get the value of a property by key
    pub fn get_property(&self, key: &PropertyKey) -> Option<&str> {
        self.properties
            .iter()
            .find(|p| &p.key == key)
            .map(|p| p.value.as_str())
    }

    /// Set or update a property value
    pub fn set_property(&mut self, key: PropertyKey, value: String) {
        if let Some(prop) = self.properties.iter_mut().find(|p| p.key == key) {
            prop.value = value;
        } else {
            self.add_property(key, value);
        }
    }

    /// Remove a property by key
    pub fn remove_property(&mut self, key: &PropertyKey) {
        self.properties.retain(|p| &p.key != key);
    }

    /// Get the assignee (convenience method)
    pub fn assignee(&self) -> Option<&str> {
        self.get_property(&PropertyKey::assigned_to())
    }

    /// Set the assignee (convenience method)
    pub fn set_assignee(&mut self, assignee: String) {
        self.set_property(PropertyKey::assigned_to(), assignee);
    }

    /// Get the status (convenience method)
    pub fn status(&self) -> Option<&str> {
        self.get_property(&PropertyKey::status())
    }

    /// Set the status (convenience method)
    pub fn set_status(&mut self, status: String) {
        self.set_property(PropertyKey::status(), status);
    }

    /// Get tags (convenience method)
    pub fn tags(&self) -> Option<Vec<&str>> {
        self.get_property(&PropertyKey::tags())
            .map(|tags| tags.split(',').map(|t| t.trim()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_task() {
        let id = TaskId::new(1).unwrap();
        let task = Task::new(id, TaskType::Task, "test task".to_string());
        assert_eq!(task.id, id);
        assert_eq!(task.subject, "test task");
        assert!(task.body.is_empty());
        assert!(task.properties.is_empty());
    }

    #[test]
    fn test_with_body() {
        let id = TaskId::new(1).unwrap();
        let task = Task::new(id, TaskType::Task, "test".to_string())
            .with_body("This is the body".to_string());
        assert_eq!(task.body, "This is the body");
    }

    #[test]
    fn test_add_property() {
        let id = TaskId::new(1).unwrap();
        let mut task = Task::new(id, TaskType::Task, "test".to_string());
        task.add_property(PropertyKey::status(), "working".to_string());
        assert_eq!(task.get_property(&PropertyKey::status()), Some("working"));
    }

    #[test]
    fn test_set_property_updates_existing() {
        let id = TaskId::new(1).unwrap();
        let mut task = Task::new(id, TaskType::Task, "test".to_string());
        task.add_property(PropertyKey::status(), "pending".to_string());
        task.set_property(PropertyKey::status(), "working".to_string());
        assert_eq!(task.get_property(&PropertyKey::status()), Some("working"));
        assert_eq!(task.properties.len(), 1);
    }

    #[test]
    fn test_convenience_methods() {
        let id = TaskId::new(1).unwrap();
        let mut task = Task::new(id, TaskType::Task, "test".to_string());

        task.set_status("working".to_string());
        assert_eq!(task.status(), Some("working"));

        task.set_assignee("John".to_string());
        assert_eq!(task.assignee(), Some("John"));
    }

    #[test]
    fn test_remove_property() {
        let id = TaskId::new(1).unwrap();
        let mut task = Task::new(id, TaskType::Task, "test".to_string());
        task.add_property(PropertyKey::status(), "working".to_string());
        task.remove_property(&PropertyKey::status());
        assert_eq!(task.get_property(&PropertyKey::status()), None);
    }
}
