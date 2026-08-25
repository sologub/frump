use anyhow::{anyhow, Context, Result};

use crate::Task;

pub fn assignment_announcement(task: &Task, assignee: &str) -> String {
    format!(
        "{} {} is assigned to {}.",
        task.task_type, task.id, assignee
    )
}

/// Announce a persisted assignment through the shared crew channel.
pub fn announce_assignment(task: &Task, assignee: &str) -> Result<String> {
    let body = assignment_announcement(task, assignee);
    let output = std::process::Command::new("metateam")
        .args(["crew", "message", "--from", "frump", "all", &body])
        .output()
        .context("Failed to run Metateam")?;
    if !output.status.success() {
        return Err(anyhow!(
            "Metateam could not send assignment announcement: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TaskId, TaskType};

    #[test]
    fn assignment_announcement_identifies_the_task_and_assignee() {
        let task = Task::new(TaskId::new(4).unwrap(), TaskType::Bug, "Fix body".into());
        assert_eq!(
            assignment_announcement(&task, "Data"),
            "Bug 4 is assigned to Data."
        );
    }
}
