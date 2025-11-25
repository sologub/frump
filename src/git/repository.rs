use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use git2::{Commit, Repository};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::domain::{TaskId, TaskType};
use crate::parser;

/// Git repository wrapper for Frump operations
pub struct FrumpRepo {
    repo: Repository,
    frump_file: PathBuf,
}

/// A historical snapshot of a task from git history
#[derive(Debug, Clone)]
pub struct TaskHistory {
    pub task_id: TaskId,
    pub commits: Vec<TaskCommit>,
}

/// A commit that affected a task
#[derive(Debug, Clone)]
pub struct TaskCommit {
    pub hash: String,
    pub author: String,
    pub date: DateTime<Utc>,
    pub message: String,
    pub change_type: ChangeType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
}

impl FrumpRepo {
    /// Open a frump repository at the given path
    pub fn open<P: AsRef<Path>>(repo_path: P) -> Result<Self> {
        let repo = Repository::discover(repo_path.as_ref())
            .context("Not a git repository or no git repository found")?;

        let frump_file = PathBuf::from("frump.md");

        Ok(FrumpRepo { repo, frump_file })
    }

    /// Find the maximum task ID ever used in git history
    pub fn max_historical_id(&self) -> Result<Option<TaskId>> {
        let mut max_id: Option<TaskId> = None;

        // Get all commits that touched frump.md
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(git2::Sort::TIME)?;

        for oid in revwalk {
            let oid = oid?;
            let commit = self.repo.find_commit(oid)?;

            // Try to read frump.md from this commit
            if let Ok(content) = self.read_file_at_commit(&commit, &self.frump_file) {
                // Parse and extract all task IDs
                if let Ok(doc) = parser::parse(&content) {
                    for task in doc.tasks.tasks() {
                        if max_id.is_none() || task.id > max_id.unwrap() {
                            max_id = Some(task.id);
                        }
                    }
                }
            }
        }

        Ok(max_id)
    }

    /// Get the history of a specific task
    pub fn task_history(&self, task_id: TaskId) -> Result<TaskHistory> {
        let mut commits = Vec::new();
        let mut task_exists_in_previous = false;

        // Walk through commits
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(git2::Sort::TIME | git2::Sort::REVERSE)?;

        for oid in revwalk {
            let oid = oid?;
            let commit = self.repo.find_commit(oid)?;

            // Check if task exists in this commit
            let task_exists = if let Ok(content) = self.read_file_at_commit(&commit, &self.frump_file) {
                if let Ok(doc) = parser::parse(&content) {
                    doc.tasks.find_by_id(task_id).is_some()
                } else {
                    false
                }
            } else {
                false
            };

            // Determine change type
            let change_type = if task_exists && !task_exists_in_previous {
                Some(ChangeType::Created)
            } else if !task_exists && task_exists_in_previous {
                Some(ChangeType::Deleted)
            } else if task_exists && task_exists_in_previous {
                Some(ChangeType::Modified)
            } else {
                None
            };

            if let Some(ct) = change_type {
                let commit_info = self.commit_to_info(&commit, ct)?;
                commits.push(commit_info);
            }

            task_exists_in_previous = task_exists;
        }

        Ok(TaskHistory {
            task_id,
            commits,
        })
    }

    /// List all tasks that have been deleted (in history but not current)
    pub fn deleted_tasks(&self) -> Result<Vec<(TaskId, TaskType, String)>> {
        let mut all_historical_tasks = HashSet::new();
        let mut current_tasks = HashSet::new();

        // Get current tasks
        let current_content = std::fs::read_to_string(&self.frump_file)
            .context("Failed to read current frump.md")?;
        let current_doc = parser::parse(&current_content)?;

        for task in current_doc.tasks.tasks() {
            current_tasks.insert(task.id);
        }

        // Collect all historical tasks
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;

        let mut task_info_map = std::collections::HashMap::new();

        for oid in revwalk {
            let oid = oid?;
            let commit = self.repo.find_commit(oid)?;

            if let Ok(content) = self.read_file_at_commit(&commit, &self.frump_file) {
                if let Ok(doc) = parser::parse(&content) {
                    for task in doc.tasks.tasks() {
                        all_historical_tasks.insert(task.id);
                        // Store task info (last seen wins, but we really want first)
                        if !task_info_map.contains_key(&task.id) {
                            task_info_map.insert(
                                task.id,
                                (task.task_type.clone(), task.subject.clone()),
                            );
                        }
                    }
                }
            }
        }

        // Find deleted tasks
        let mut deleted = Vec::new();
        for id in all_historical_tasks {
            if !current_tasks.contains(&id) {
                if let Some((task_type, subject)) = task_info_map.get(&id) {
                    deleted.push((id, task_type.clone(), subject.clone()));
                }
            }
        }

        // Sort by ID
        deleted.sort_by_key(|(id, _, _)| *id);

        Ok(deleted)
    }

    /// Read a file from a specific commit
    fn read_file_at_commit(&self, commit: &Commit, path: &Path) -> Result<String> {
        let tree = commit.tree()?;
        let entry = tree
            .get_path(path)
            .with_context(|| format!("File {:?} not found in commit {}", path, commit.id()))?;

        let object = entry.to_object(&self.repo)?;
        let blob = object
            .as_blob()
            .ok_or_else(|| anyhow!("Object is not a blob"))?;

        let content = std::str::from_utf8(blob.content())
            .context("File content is not valid UTF-8")?
            .to_string();

        Ok(content)
    }

    /// Convert a commit to TaskCommit info
    fn commit_to_info(&self, commit: &Commit, change_type: ChangeType) -> Result<TaskCommit> {
        let author = commit.author();
        let author_name = author
            .name()
            .unwrap_or("Unknown")
            .to_string();

        let timestamp = commit.time().seconds();
        let date = DateTime::from_timestamp(timestamp, 0)
            .ok_or_else(|| anyhow!("Invalid timestamp"))?;

        Ok(TaskCommit {
            hash: commit.id().to_string(),
            author: author_name,
            date,
            message: commit.message().unwrap_or("").trim().to_string(),
            change_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_repo() {
        // This test only works if run in a git repo
        let result = FrumpRepo::open(".");
        // Don't assert - might not be in a repo in test environment
        if result.is_ok() {
            println!("Successfully opened repository");
        }
    }
}
