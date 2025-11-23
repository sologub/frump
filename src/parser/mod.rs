use anyhow::{anyhow, Result};

use crate::domain::*;

pub fn parse(content: &str) -> Result<FrumpDoc> {
    let mut header = String::new();
    let mut team_members = Vec::new();
    let mut tasks = Vec::new();

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    // Parse header (everything before ## Team or ## Tasks)
    while i < lines.len() {
        let line = lines[i].trim();
        if line.to_uppercase().starts_with("## TEAM") || line.to_uppercase().starts_with("## TASKS") {
            break;
        }
        header.push_str(lines[i]);
        header.push('\n');
        i += 1;
    }

    // Parse Team section if present
    if i < lines.len() && lines[i].trim().to_uppercase().starts_with("## TEAM") {
        i += 1; // Skip the ## Team line
        while i < lines.len() {
            let line = lines[i].trim();
            if line.starts_with("## ") {
                break;
            }
            if line.starts_with("* ") || line.starts_with("- ") {
                if let Some(member) = parse_team_member(line)? {
                    team_members.push(member);
                }
            }
            i += 1;
        }
    }

    // Parse Tasks section if present
    if i < lines.len() && lines[i].trim().to_uppercase().starts_with("## TASKS") {
        i += 1; // Skip the ## Tasks line

        while i < lines.len() {
            let line = lines[i].trim();

            // Start of a new task
            if line.starts_with("### ") {
                let task_start = i;
                i += 1;

                // Find the end of this task (next ### or end of file)
                while i < lines.len() && !lines[i].trim().starts_with("### ") {
                    i += 1;
                }

                if let Some(task) = parse_task(&lines[task_start..i])? {
                    tasks.push(task);
                }
            } else {
                i += 1;
            }
        }
    }

    let team = Team::new(team_members);
    let task_collection = TaskCollection::new(tasks);

    Ok(FrumpDoc::new(header, team, task_collection))
}

pub fn serialize(doc: &FrumpDoc) -> String {
    let mut output = String::new();

    // Write header
    output.push_str(&doc.header);

    // Write Team section if there are team members
    if !doc.team.is_empty() {
        output.push_str("## Team\n\n");
        for member in doc.team.members() {
            output.push_str(&format!("* {} <{}>", member.name, member.email));
            if let Some(role) = &member.role {
                output.push_str(&format!(" - {}", role));
            }
            output.push('\n');
        }
        output.push('\n');
    }

    // Write Tasks section
    output.push_str("## Tasks\n\n");
    for task in doc.tasks.tasks() {
        output.push_str(&format!("### {} {} - {}\n", task.task_type, task.id, task.subject));

        if !task.body.is_empty() {
            output.push('\n');
            output.push_str(task.body.trim());
            output.push('\n');
        }

        if !task.properties.is_empty() {
            if !task.body.is_empty() {
                output.push('\n');
            }
            for prop in &task.properties {
                output.push_str(&format!("{}: {}\n", prop.key, prop.value));
            }
        }

        output.push('\n');
    }

    output
}

fn parse_team_member(line: &str) -> Result<Option<TeamMember>> {
    // Format: * Name <email> - Role
    // or: * Name <email>
    let line = line.trim_start_matches("* ").trim_start_matches("- ");

    if let Some(email_start) = line.find('<') {
        if let Some(email_end) = line.find('>') {
            let name = line[..email_start].trim().to_string();
            let email_str = &line[email_start + 1..email_end];
            let email = Email::new(email_str)?;

            let role = if email_end + 1 < line.len() {
                let remainder = line[email_end + 1..].trim();
                if remainder.starts_with('-') {
                    Some(remainder[1..].trim().to_string())
                } else {
                    None
                }
            } else {
                None
            };

            let mut member = TeamMember::new(name, email);
            if let Some(r) = role {
                member.set_role(r);
            }

            return Ok(Some(member));
        }
    }

    Ok(None)
}

fn parse_task(lines: &[&str]) -> Result<Option<Task>> {
    if lines.is_empty() {
        return Ok(None);
    }

    // Parse the heading: ### Task 3 - Write docs
    let heading = lines[0].trim_start_matches("### ").trim();

    // Split into task_type, id, and subject
    let parts: Vec<&str> = heading.splitn(3, ' ').collect();

    if parts.len() < 2 {
        return Err(anyhow!("Invalid task heading format: {}", heading));
    }

    let task_type = TaskType::parse(parts[0]);
    let id = TaskId::new(
        parts[1]
            .parse()
            .map_err(|_| anyhow!("Invalid task ID: {}", parts[1]))?,
    )?;

    let subject = if parts.len() > 2 {
        parts[2].trim_start_matches("- ").trim().to_string()
    } else {
        String::new()
    };

    let mut task = Task::new(id, task_type, subject);

    // Parse body and properties
    let mut body_lines = Vec::new();
    let mut in_body = true;

    for line in &lines[1..] {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            if in_body && !body_lines.is_empty() {
                body_lines.push("");
            }
            continue;
        }

        // Check if this line is a property
        if let Some((key, value)) = try_parse_property(trimmed) {
            in_body = false;
            task.add_property(key, value);
        } else if in_body {
            body_lines.push(trimmed);
        }
    }

    let body = body_lines.join("\n").trim().to_string();
    task.set_body(body);

    Ok(Some(task))
}

fn try_parse_property(line: &str) -> Option<(PropertyKey, String)> {
    // A property is a name/value pair separated by ':'
    // The name must be capitalized and can consist of maximum 3 words
    if let Some(colon_pos) = line.find(':') {
        let key_str = line[..colon_pos].trim();
        let value = line[colon_pos + 1..].trim();

        // Try to parse as a property key
        if let Some(key) = PropertyKey::try_parse(key_str) {
            return Some((key, value.to_string()));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_task() {
        let content = r#"# Test

## Tasks

### Task 1 - test task
"#;
        let doc = parse(content).unwrap();
        assert_eq!(doc.tasks.len(), 1);
        let task = doc.tasks.tasks().first().unwrap();
        assert_eq!(task.id.value(), 1);
        assert_eq!(task.subject, "test task");
    }

    #[test]
    fn test_parse_task_with_body() {
        let content = r#"# Test

## Tasks

### Task 1 - test task

This is the body
"#;
        let doc = parse(content).unwrap();
        let task = doc.tasks.tasks().first().unwrap();
        assert_eq!(task.body, "This is the body");
    }

    #[test]
    fn test_parse_task_with_properties() {
        let content = r#"# Test

## Tasks

### Task 1 - test task

Some body text
Status: working
Assigned To: John
"#;
        let doc = parse(content).unwrap();
        let task = doc.tasks.tasks().first().unwrap();
        assert_eq!(task.status(), Some("working"));
        assert_eq!(task.assignee(), Some("John"));
    }

    #[test]
    fn test_parse_team() {
        let content = r#"# Test

## Team

* John Doe <john@example.com> - Developer
* Jane Smith <jane@example.com>

## Tasks
"#;
        let doc = parse(content).unwrap();
        assert_eq!(doc.team.len(), 2);
        let first = doc.team.members().first().unwrap();
        assert_eq!(first.name, "John Doe");
        assert_eq!(first.role, Some("Developer".to_string()));
    }

    #[test]
    fn test_round_trip() {
        let content = r#"# Test

## Team

* John Doe <john@example.com> - Developer

## Tasks

### Task 1 - test task

Body text
Status: working

"#;
        let doc = parse(content).unwrap();
        let serialized = serialize(&doc);
        let doc2 = parse(&serialized).unwrap();

        assert_eq!(doc.tasks.len(), doc2.tasks.len());
        assert_eq!(doc.team.len(), doc2.team.len());
    }
}
