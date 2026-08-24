//! Backward-compatible storage for a single board file and an already-sharded board.

use anyhow::{bail, Context, Result};
use std::{collections::BTreeSet, fs, path::{Path, PathBuf}};

use crate::{parser, FrumpDoc, Task, TaskCollection};

pub fn is_sharded(path: &Path) -> bool {
    let root = if path.is_dir() { path } else { path.parent().unwrap_or(path) };
    root.join("general.md").is_file() && root.join("tasks").is_dir()
}

pub fn sharded_root(path: &Path) -> Option<PathBuf> {
    if path.is_dir() && is_sharded(path) { Some(path.to_path_buf()) }
    else if path.file_name().is_some_and(|name| name == "general.md") && is_sharded(path.parent()?) { path.parent().map(Path::to_path_buf) }
    else { None }
}

pub fn read(path: &Path) -> Result<FrumpDoc> {
    let Some(root) = sharded_root(path) else {
        return parser::parse(&fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?);
    };
    let general = parser::parse(&fs::read_to_string(root.join("general.md")).context("Failed to read general.md")?)?;
    let mut tasks = Vec::new();
    for entry in fs::read_dir(root.join("tasks")).context("Failed to read tasks directory")? {
        let entry = entry?;
        if entry.path().extension().is_some_and(|ext| ext == "md") {
            let fragment = fs::read_to_string(entry.path())?;
            let parsed = parser::parse(&format!("## Tasks\n\n{fragment}"))?;
            if parsed.tasks.len() != 1 { bail!("{} must contain exactly one task", entry.path().display()); }
            tasks.extend(parsed.tasks.tasks().iter().cloned());
        }
    }
    tasks.sort_by_key(|task| task.id);
    Ok(FrumpDoc::new(general.header, general.team, TaskCollection::new(tasks)))
}

pub fn write(path: &Path, doc: &FrumpDoc) -> Result<()> {
    let Some(root) = sharded_root(path) else {
        return write_if_changed(path, &parser::serialize(doc));
    };
    let general = FrumpDoc::new(doc.header.clone(), doc.team.clone(), TaskCollection::empty());
    write_if_changed(&root.join("general.md"), &serialize_general(&general))?;
    let tasks_dir = root.join("tasks");
    let expected: BTreeSet<_> = doc.tasks.tasks().iter().map(|task| task.id.value()).collect();
    for task in doc.tasks.tasks() {
        let path = tasks_dir.join(format!("{}.md", task.id.value()));
        if !task_file_matches(&path, task) {
            write_if_changed(&path, &serialize_task(task))?;
        }
    }
    for entry in fs::read_dir(&tasks_dir)? {
        let entry = entry?;
        if let Some(id) = entry.path().file_stem().and_then(|name| name.to_str()).and_then(|name| name.parse::<u32>().ok()) {
            if !expected.contains(&id) { fs::remove_file(entry.path())?; }
        }
    }
    Ok(())
}

fn task_file_matches(path: &Path, task: &Task) -> bool {
    let Ok(fragment) = fs::read_to_string(path) else { return false; };
    let Ok(parsed) = parser::parse(&format!("## Tasks\n\n{fragment}")) else { return false; };
    parsed.tasks.len() == 1 && serialize_task(&parsed.tasks.tasks()[0]) == serialize_task(task)
}

fn write_if_changed(path: &Path, content: &str) -> Result<()> {
    if fs::read_to_string(path).ok().as_deref() != Some(content) {
        fs::write(path, content).with_context(|| format!("Failed to write {}", path.display()))?;
    }
    Ok(())
}

fn serialize_general(doc: &FrumpDoc) -> String {
    let mut content = doc.header.clone();
    if !doc.team.is_empty() {
        content.push_str("## Team\n\n");
        for member in doc.team.members() {
            content.push_str(&format!("* {} <{}>", member.name, member.email));
            if let Some(role) = &member.role { content.push_str(&format!(" - {role}")); }
            content.push('\n');
        }
    }
    content
}

fn serialize_task(task: &Task) -> String {
    let doc = FrumpDoc::new(String::new(), crate::Team::empty(), TaskCollection::new(vec![task.clone()]));
    parser::serialize(&doc).splitn(2, "## Tasks\n\n").nth(1).unwrap_or_default().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TaskId, TaskType, Team};

    #[test]
    fn sharded_board_round_trips_tasks_as_individual_files() {
        let root = std::env::temp_dir().join(format!("frump-storage-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("tasks")).unwrap();
        fs::write(root.join("general.md"), "# Check\n\n## Team\n").unwrap();
        let mut task = Task::new(TaskId::new(1).unwrap(), TaskType::Task, "One".into());
        task.set_body("Evidence".into());
        let doc = FrumpDoc::new("# Check\n\n".into(), Team::empty(), TaskCollection::new(vec![task]));
        write(&root, &doc).unwrap();
        assert!(root.join("tasks/1.md").is_file());
        let loaded = read(&root).unwrap();
        assert_eq!(loaded.tasks.len(), 1);
        assert_eq!(loaded.tasks.tasks()[0].body, "Evidence");
        let _ = fs::remove_dir_all(root);
    }
}
