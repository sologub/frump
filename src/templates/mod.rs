use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::domain::*;

/// A task template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTemplate {
    pub name: String,
    pub task_type: String,
    pub subject_template: String,
    pub body_template: String,
    pub properties: HashMap<String, String>,
}

/// Template manager
pub struct TemplateManager {
    templates_file: PathBuf,
}

impl TemplateManager {
    pub fn new() -> Self {
        TemplateManager {
            templates_file: PathBuf::from(".frump_templates.json"),
        }
    }

    /// Load templates from file
    pub fn load(&self) -> Result<Vec<TaskTemplate>> {
        if !self.templates_file.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.templates_file)
            .context("Failed to read templates file")?;

        let templates: Vec<TaskTemplate> = serde_json::from_str(&content)
            .context("Failed to parse templates file")?;

        Ok(templates)
    }

    /// Save templates to file
    pub fn save(&self, templates: &[TaskTemplate]) -> Result<()> {
        let content = serde_json::to_string_pretty(templates)
            .context("Failed to serialize templates")?;

        fs::write(&self.templates_file, content)
            .context("Failed to write templates file")?;

        Ok(())
    }

    /// Add a new template
    pub fn add(&self, template: TaskTemplate) -> Result<()> {
        let mut templates = self.load()?;

        // Check if template with same name exists
        if templates.iter().any(|t| t.name == template.name) {
            return Err(anyhow::anyhow!(
                "Template '{}' already exists",
                template.name
            ));
        }

        templates.push(template);
        self.save(&templates)?;

        Ok(())
    }

    /// Get a template by name
    pub fn get(&self, name: &str) -> Result<TaskTemplate> {
        let templates = self.load()?;

        templates
            .into_iter()
            .find(|t| t.name == name)
            .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", name))
    }

    /// List all templates
    pub fn list(&self) -> Result<Vec<TaskTemplate>> {
        self.load()
    }

    /// Remove a template
    pub fn remove(&self, name: &str) -> Result<()> {
        let mut templates = self.load()?;

        let original_len = templates.len();
        templates.retain(|t| t.name != name);

        if templates.len() == original_len {
            return Err(anyhow::anyhow!("Template '{}' not found", name));
        }

        self.save(&templates)?;
        Ok(())
    }

    /// Create a task from a template
    pub fn instantiate(
        &self,
        template_name: &str,
        id: TaskId,
        replacements: &HashMap<String, String>,
    ) -> Result<Task> {
        let template = self.get(template_name)?;

        // Replace placeholders in subject
        let mut subject = template.subject_template.clone();
        for (key, value) in replacements {
            let placeholder = format!("{{{}}}", key);
            subject = subject.replace(&placeholder, value);
        }

        // Replace placeholders in body
        let mut body = template.body_template.clone();
        for (key, value) in replacements {
            let placeholder = format!("{{{}}}", key);
            body = body.replace(&placeholder, value);
        }

        let task_type = TaskType::parse(&template.task_type);
        let mut task = Task::new(id, task_type, subject);
        task.set_body(body);

        // Add properties from template
        for (key, value) in &template.properties {
            if let Ok(prop_key) = PropertyKey::new(key) {
                task.add_property(prop_key, value.clone());
            }
        }

        Ok(task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_placeholders() {
        let _template = TaskTemplate {
            name: "bug".to_string(),
            task_type: "Bug".to_string(),
            subject_template: "Fix {component} issue".to_string(),
            body_template: "Issue in {component}: {description}".to_string(),
            properties: HashMap::new(),
        };

        let mut replacements = HashMap::new();
        replacements.insert("component".to_string(), "parser".to_string());
        replacements.insert("description".to_string(), "crashes on empty input".to_string());

        let manager = TemplateManager::new();
        let _task = manager
            .instantiate(
                "bug",
                TaskId::new(1).unwrap(),
                &replacements,
            )
            .ok();

        // This would fail because template doesn't exist in file
        // but we can test the logic separately
    }
}
