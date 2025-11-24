use super::property::PropertyKey;
use super::task::Task;
use super::task_id::TaskId;
use super::task_type::TaskType;
use super::team::Team;

/// Collection of tasks with useful operations
#[derive(Debug)]
pub struct TaskCollection {
    tasks: Vec<Task>,
}

impl TaskCollection {
    /// Create a new task collection
    pub fn new(tasks: Vec<Task>) -> Self {
        TaskCollection { tasks }
    }

    /// Create an empty task collection
    pub fn empty() -> Self {
        TaskCollection { tasks: Vec::new() }
    }

    /// Get all tasks
    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }

    /// Get mutable access to tasks
    pub fn tasks_mut(&mut self) -> &mut Vec<Task> {
        &mut self.tasks
    }

    /// Find a task by ID
    pub fn find_by_id(&self, id: TaskId) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == id)
    }

    /// Find a task by ID (mutable)
    pub fn find_by_id_mut(&mut self, id: TaskId) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.id == id)
    }

    /// Get the maximum task ID
    pub fn max_id(&self) -> Option<TaskId> {
        self.tasks.iter().map(|t| t.id).max()
    }

    /// Get the next available task ID
    pub fn next_id(&self) -> TaskId {
        self.max_id()
            .map(|id| id.next())
            .unwrap_or_else(|| TaskId::new(1).unwrap())
    }

    /// Add a task
    pub fn add(&mut self, task: Task) {
        self.tasks.push(task);
    }

    /// Remove a task by ID, returning it if found
    pub fn remove(&mut self, id: TaskId) -> Option<Task> {
        let pos = self.tasks.iter().position(|t| t.id == id)?;
        Some(self.tasks.remove(pos))
    }

    /// Filter tasks by assignee
    pub fn filter_by_assignee(&self, assignee: &str) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|t| t.assignee().map(|a| a == assignee).unwrap_or(false))
            .collect()
    }

    /// Filter tasks by status
    pub fn filter_by_status(&self, status: &str) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|t| t.status().map(|s| s == status).unwrap_or(false))
            .collect()
    }

    /// Filter tasks by type
    pub fn filter_by_type(&self, task_type: &TaskType) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|t| &t.task_type == task_type)
            .collect()
    }

    /// Check if collection is empty
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Get the number of tasks
    pub fn len(&self) -> usize {
        self.tasks.len()
    }
}

/// The complete frump.md document structure
#[derive(Debug)]
pub struct FrumpDoc {
    pub header: String,
    pub team: Team,
    pub tasks: TaskCollection,
}

impl FrumpDoc {
    /// Create a new frump document
    pub fn new(header: String, team: Team, tasks: TaskCollection) -> Self {
        FrumpDoc {
            header,
            team,
            tasks,
        }
    }

    /// Apply default assignees to tasks that don't have one
    pub fn apply_default_assignees(&mut self) {
        if let Some(default) = self.team.default_assignee() {
            for task in self.tasks.tasks_mut() {
                if task.assignee().is_none() {
                    task.add_property(PropertyKey::assigned_to(), default.name.clone());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_task(id: u32, subject: &str) -> Task {
        Task::new(
            TaskId::new(id).unwrap(),
            TaskType::Task,
            subject.to_string(),
        )
    }

    #[test]
    fn test_empty_collection() {
        let collection = TaskCollection::empty();
        assert!(collection.is_empty());
        assert_eq!(collection.len(), 0);
    }

    #[test]
    fn test_next_id_empty() {
        let collection = TaskCollection::empty();
        assert_eq!(collection.next_id().value(), 1);
    }

    #[test]
    fn test_next_id() {
        let tasks = vec![create_test_task(1, "test"), create_test_task(3, "test2")];
        let collection = TaskCollection::new(tasks);
        assert_eq!(collection.next_id().value(), 4);
    }

    #[test]
    fn test_find_by_id() {
        let tasks = vec![create_test_task(1, "test"), create_test_task(2, "test2")];
        let collection = TaskCollection::new(tasks);

        assert!(collection.find_by_id(TaskId::new(1).unwrap()).is_some());
        assert!(collection.find_by_id(TaskId::new(3).unwrap()).is_none());
    }

    #[test]
    fn test_add_task() {
        let mut collection = TaskCollection::empty();
        collection.add(create_test_task(1, "test"));
        assert_eq!(collection.len(), 1);
    }

    #[test]
    fn test_remove_task() {
        let mut collection = TaskCollection::new(vec![create_test_task(1, "test")]);
        let removed = collection.remove(TaskId::new(1).unwrap());
        assert!(removed.is_some());
        assert!(collection.is_empty());
    }

    #[test]
    fn test_filter_by_type() {
        let tasks = vec![
            Task::new(TaskId::new(1).unwrap(), TaskType::Task, "test".to_string()),
            Task::new(TaskId::new(2).unwrap(), TaskType::Bug, "bug".to_string()),
        ];
        let collection = TaskCollection::new(tasks);

        let bugs = collection.filter_by_type(&TaskType::Bug);
        assert_eq!(bugs.len(), 1);
        assert_eq!(bugs[0].id.value(), 2);
    }

    #[test]
    fn test_filter_by_status() {
        let mut task1 = create_test_task(1, "test1");
        task1.set_status("working".to_string());

        let mut task2 = create_test_task(2, "test2");
        task2.set_status("done".to_string());

        let collection = TaskCollection::new(vec![task1, task2]);
        let working = collection.filter_by_status("working");
        assert_eq!(working.len(), 1);
    }
}
