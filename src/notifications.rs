use std::ffi::OsStr;
use std::path::Path;

use anyhow::{anyhow, Context, Result};

use crate::Task;

pub fn assignment_announcement(task: &Task, assignee: &str) -> String {
    format!(
        "{} {} is assigned to {}.",
        task.task_type, task.id, assignee
    )
}

/// Announce a persisted assignment through the shared crew channel.
///
/// Returns `None` when the optional `metateam` command is not installed, so an
/// assignment stays a purely local operation on machines without Metateam.
pub fn announce_assignment(task: &Task, assignee: &str) -> Result<Option<String>> {
    let body = assignment_announcement(task, assignee);
    send_metateam_message(
        &["crew", "message", "--from", "frump", "all", &body],
        "send assignment announcement",
    )
}

/// Run `metateam` with the given arguments.
///
/// `metateam` is an optional integration. When the command is not on `PATH`
/// the message is skipped and `Ok(None)` is returned, so the caller keeps
/// working. A command that exists but fails is still an error.
pub fn send_metateam_message(args: &[&str], failure: &str) -> Result<Option<String>> {
    if !metateam_available() {
        return Ok(None);
    }
    let output = std::process::Command::new("metateam")
        .args(args)
        .output()
        .context("Failed to run Metateam")?;
    if !output.status.success() {
        return Err(anyhow!(
            "Metateam could not {failure}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(Some(
        String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string(),
    ))
}

/// True when an executable named `metateam` is found on `PATH`.
pub fn metateam_available() -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    metateam_in_path(&path)
}

fn metateam_in_path(path: &OsStr) -> bool {
    let names: &[&str] = if cfg!(windows) {
        &["metateam.exe", "metateam"]
    } else {
        &["metateam"]
    };
    std::env::split_paths(path).any(|directory| {
        names.iter().any(|name| {
            let candidate = directory.join(name);
            candidate.is_file() && is_executable(&candidate)
        })
    })
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path)
        .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TaskId, TaskType};
    use std::fs;

    #[test]
    fn assignment_announcement_identifies_the_task_and_assignee() {
        let task = Task::new(TaskId::new(4).unwrap(), TaskType::Bug, "Fix body".into());
        assert_eq!(
            assignment_announcement(&task, "Ada"),
            "Bug 4 is assigned to Ada."
        );
    }

    #[test]
    fn metateam_in_path_accepts_only_an_executable_command() {
        let root = std::env::temp_dir().join(format!("frump-metateam-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let empty = root.join("empty");
        let tools = root.join("tools");
        fs::create_dir_all(&empty).unwrap();
        fs::create_dir_all(&tools).unwrap();

        let path = std::env::join_paths([&empty, &tools]).unwrap();
        assert!(
            !metateam_in_path(&path),
            "no command on PATH is not available"
        );

        let plain = tools.join("metateam");
        fs::write(&plain, "not executable\n").unwrap();
        assert!(
            !metateam_in_path(&path),
            "a file without execute permission is not a command"
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&plain).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&plain, permissions).unwrap();
            assert!(metateam_in_path(&path), "an executable file is a command");
        }

        fs::remove_dir_all(&root).unwrap();
    }
}
