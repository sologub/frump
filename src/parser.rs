use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
pub struct Task {
    pub id: u32,
    pub task_type: String,
    pub subject: String,
    pub body: String,
    pub properties: HashMap<String, String>,
}

#[derive(Debug)]
pub struct TeamMember {
    pub name: String,
    pub email: String,
    pub role: Option<String>,
}

#[derive(Debug)]
pub struct FrumpDoc {
    pub header: String,
    pub team: Vec<TeamMember>,
    pub tasks: Vec<Task>,
}

impl FrumpDoc {
    pub fn parse(content: &str) -> Result<Self> {
        let mut header = String::new();
        let mut team = Vec::new();
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
                    if let Some(member) = parse_team_member(line) {
                        team.push(member);
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

        Ok(FrumpDoc { header, team, tasks })
    }
}

impl fmt::Display for FrumpDoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write header
        write!(f, "{}", self.header)?;

        // Write Team section if there are team members
        if !self.team.is_empty() {
            writeln!(f, "## Team")?;
            writeln!(f)?;
            for member in &self.team {
                write!(f, "* {} <{}>", member.name, member.email)?;
                if let Some(role) = &member.role {
                    write!(f, " - {}", role)?;
                }
                writeln!(f)?;
            }
            writeln!(f)?;
        }

        // Write Tasks section
        writeln!(f, "## Tasks")?;
        writeln!(f)?;
        for task in &self.tasks {
            writeln!(f, "### {} {} - {}", task.task_type, task.id, task.subject)?;

            if !task.body.is_empty() {
                writeln!(f)?;
                writeln!(f, "{}", task.body.trim())?;
            }

            if !task.properties.is_empty() {
                if !task.body.is_empty() {
                    writeln!(f)?;
                }
                for (key, value) in &task.properties {
                    writeln!(f, "{}: {}", key, value)?;
                }
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

fn parse_team_member(line: &str) -> Option<TeamMember> {
    // Format: * Name <email> - Role
    // or: * Name <email>
    let line = line.trim_start_matches("* ").trim_start_matches("- ");

    if let Some(email_start) = line.find('<') {
        if let Some(email_end) = line.find('>') {
            let name = line[..email_start].trim().to_string();
            let email = line[email_start + 1..email_end].to_string();

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

            return Some(TeamMember { name, email, role });
        }
    }

    None
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

    let task_type = parts[0].to_string();
    let id: u32 = parts[1].parse()
        .map_err(|_| anyhow!("Invalid task ID: {}", parts[1]))?;

    let subject = if parts.len() > 2 {
        parts[2].trim_start_matches("- ").trim().to_string()
    } else {
        String::new()
    };

    // Parse body and properties
    let mut body_lines = Vec::new();
    let mut properties = HashMap::new();
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
        if let Some((key, value)) = parse_property(trimmed) {
            in_body = false;
            properties.insert(key, value);
        } else if in_body {
            body_lines.push(trimmed);
        }
    }

    let body = body_lines.join("\n").trim().to_string();

    Ok(Some(Task {
        id,
        task_type,
        subject,
        body,
        properties,
    }))
}

fn parse_property(line: &str) -> Option<(String, String)> {
    // A property is a name/value pair separated by ':'
    // The name must be capitalized and can consist of maximum 3 words
    if let Some(colon_pos) = line.find(':') {
        let key = line[..colon_pos].trim();
        let value = line[colon_pos + 1..].trim();

        // Check if key starts with uppercase and has max 3 words
        if key.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
            let word_count = key.split_whitespace().count();
            if word_count <= 3 {
                return Some((key.to_string(), value.to_string()));
            }
        }
    }

    None
}
