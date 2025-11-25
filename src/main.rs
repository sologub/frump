use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

use frump::{export_csv, export_json, import_json, parser, ChangeType, FrumpRepo, PropertyKey, Task, TaskId, TaskType, TaskTemplate, TemplateManager};

#[derive(Parser)]
#[command(name = "frump")]
#[command(about = "Distributed task management tool based on Git and Markdown", long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "frump.md")]
    file: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all tasks
    List {
        /// Filter by task type
        #[arg(short = 't', long)]
        task_type: Option<String>,

        /// Filter by status
        #[arg(short = 's', long)]
        status: Option<String>,

        /// Filter by assignee
        #[arg(short = 'a', long)]
        assignee: Option<String>,
    },

    /// Show details of a specific task
    Show {
        /// Task ID
        id: u32,
    },

    /// Add a new task
    Add {
        /// Task type (e.g., Task, Bug, Issue, Feature)
        #[arg(short = 't', long, default_value = "Task")]
        task_type: String,

        /// Task subject/title
        subject: String,

        /// Task body/description (optional)
        #[arg(short, long)]
        body: Option<String>,

        /// Assignee name (optional)
        #[arg(short, long)]
        assignee: Option<String>,

        /// Status (optional)
        #[arg(short, long)]
        status: Option<String>,
    },

    /// Close a task by removing it from frump.md
    Close {
        /// Task ID
        id: u32,
    },

    /// Assign a task to a team member
    Assign {
        /// Task ID
        id: u32,

        /// Assignee name
        assignee: String,
    },

    /// Set a property on a task
    Set {
        /// Task ID
        id: u32,

        /// Property name (must be capitalized, max 3 words)
        property: String,

        /// Property value
        value: String,
    },

    /// Show the history of a task
    History {
        /// Task ID
        id: u32,
    },

    /// List all closed (deleted) tasks
    Closed,

    /// Update a task's subject or body
    Update {
        /// Task ID
        id: u32,

        /// New subject (optional)
        #[arg(long)]
        subject: Option<String>,

        /// New body (optional)
        #[arg(long)]
        body: Option<String>,
    },

    /// Search tasks by keyword
    Search {
        /// Search query
        query: String,

        /// Search in body as well as subject
        #[arg(short, long)]
        full: bool,
    },

    /// Show task statistics
    Stats,

    /// Validate frump.md file
    Validate,

    /// Export tasks to JSON or CSV
    Export {
        /// Output format
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Import tasks from JSON
    Import {
        /// Input file
        file: PathBuf,

        /// Merge with existing tasks instead of replacing
        #[arg(short, long)]
        merge: bool,
    },

    /// Manage task templates
    Template {
        #[command(subcommand)]
        action: TemplateAction,
    },

    /// Bulk operations on tasks
    Bulk {
        #[command(subcommand)]
        action: BulkAction,
    },

    /// Check for duplicate task IDs (merge conflicts)
    CheckConflicts,

    /// Resolve duplicate task IDs by renumbering
    ResolveConflicts {
        /// Automatically commit the resolution
        #[arg(short, long)]
        commit: bool,
    },
}

#[derive(Subcommand)]
enum TemplateAction {
    /// Add a new template
    Add {
        /// Template name
        name: String,

        /// Task type
        #[arg(short = 't', long, default_value = "Task")]
        task_type: String,

        /// Subject template (use {placeholder} for variables)
        subject: String,

        /// Body template (optional)
        #[arg(short, long)]
        body: Option<String>,
    },

    /// List all templates
    List,

    /// Remove a template
    Remove {
        /// Template name
        name: String,
    },

    /// Show template details
    Show {
        /// Template name
        name: String,
    },
}

#[derive(Subcommand)]
enum BulkAction {
    /// Close multiple tasks by status
    CloseByStatus {
        /// Status to close
        status: String,
    },

    /// Assign multiple tasks to a person
    AssignByType {
        /// Task type to assign
        task_type: String,

        /// Assignee name
        assignee: String,
    },

    /// Set property on multiple tasks
    SetByStatus {
        /// Status to filter
        status: String,

        /// Property to set
        property: String,

        /// Property value
        value: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::List {
            task_type,
            status,
            assignee,
        } => {
            let content = fs::read_to_string(&cli.file).context("Failed to read frump.md file")?;
            let doc = parser::parse(&content)?;

            let mut tasks = doc.tasks.tasks().to_vec();

            // Apply filters
            if let Some(tt) = task_type {
                let filter_type = TaskType::parse(tt);
                tasks.retain(|t| t.task_type == filter_type);
            }

            if let Some(s) = status {
                tasks.retain(|t| t.status().map(|st| st == s).unwrap_or(false));
            }

            if let Some(a) = assignee {
                tasks.retain(|t| t.assignee().map(|name| name == a).unwrap_or(false));
            }

            if tasks.is_empty() {
                println!("No tasks found.");
            } else {
                for task in &tasks {
                    println!("{} {} - {}", task.task_type, task.id, task.subject);
                    if let Some(status) = task.status() {
                        println!("  Status: {}", status);
                    }
                    if let Some(assignee) = task.assignee() {
                        println!("  Assigned to: {}", assignee);
                    }
                }
            }
        }

        Commands::Show { id } => {
            let content = fs::read_to_string(&cli.file).context("Failed to read frump.md file")?;
            let doc = parser::parse(&content)?;

            let task_id = TaskId::new(*id)?;
            if let Some(task) = doc.tasks.find_by_id(task_id) {
                println!("### {} {} - {}\n", task.task_type, task.id, task.subject);

                if !task.body.is_empty() {
                    println!("{}\n", task.body);
                }

                if !task.properties.is_empty() {
                    for prop in &task.properties {
                        println!("{}: {}", prop.key, prop.value);
                    }
                }
            } else {
                println!("Task {} not found.", id);
            }
        }

        Commands::Add {
            task_type,
            subject,
            body,
            assignee,
            status,
        } => {
            let content = fs::read_to_string(&cli.file).context("Failed to read frump.md file")?;
            let mut doc = parser::parse(&content)?;

            // Find the next available task ID, considering git history
            let next_id = if let Ok(repo) = FrumpRepo::open(".") {
                if let Ok(Some(max_historical)) = repo.max_historical_id() {
                    let current_max = doc.tasks.max_id();
                    if let Some(current) = current_max {
                        if max_historical > current {
                            max_historical.next()
                        } else {
                            current.next()
                        }
                    } else {
                        max_historical.next()
                    }
                } else {
                    doc.tasks.next_id()
                }
            } else {
                // Not in a git repo or can't open, use current max
                doc.tasks.next_id()
            };

            let mut new_task = Task::new(
                next_id,
                TaskType::parse(task_type),
                subject.clone(),
            );

            if let Some(b) = body {
                new_task.set_body(b.clone());
            }

            if let Some(a) = assignee {
                new_task.set_assignee(a.clone());
            } else if let Some(default) = doc.team.default_assignee() {
                new_task.set_assignee(default.name.clone());
            }

            if let Some(s) = status {
                new_task.set_status(s.clone());
            }

            doc.tasks.add(new_task);

            // Write back to file
            let new_content = parser::serialize(&doc);
            fs::write(&cli.file, new_content).context("Failed to write frump.md file")?;

            println!("Added {} {} - {}", task_type, next_id, subject);
        }

        Commands::Close { id } => {
            let content = fs::read_to_string(&cli.file).context("Failed to read frump.md file")?;
            let mut doc = parser::parse(&content)?;

            let task_id = TaskId::new(*id)?;
            if let Some(task) = doc.tasks.remove(task_id) {
                let new_content = parser::serialize(&doc);
                fs::write(&cli.file, new_content).context("Failed to write frump.md file")?;

                println!("Closed {} {} - {}", task.task_type, task.id, task.subject);
                println!("\nRemember to commit this change with a descriptive message.");
            } else {
                println!("Task {} not found.", id);
            }
        }

        Commands::Assign { id, assignee } => {
            let content = fs::read_to_string(&cli.file).context("Failed to read frump.md file")?;
            let mut doc = parser::parse(&content)?;

            let task_id = TaskId::new(*id)?;
            if let Some(task) = doc.tasks.find_by_id_mut(task_id) {
                task.set_assignee(assignee.clone());

                let new_content = parser::serialize(&doc);
                fs::write(&cli.file, new_content).context("Failed to write frump.md file")?;

                println!("Assigned task {} to {}", id, assignee);
            } else {
                println!("Task {} not found.", id);
            }
        }

        Commands::Set {
            id,
            property,
            value,
        } => {
            let content = fs::read_to_string(&cli.file).context("Failed to read frump.md file")?;
            let mut doc = parser::parse(&content)?;

            let task_id = TaskId::new(*id)?;
            let prop_key = PropertyKey::new(property)?;

            if let Some(task) = doc.tasks.find_by_id_mut(task_id) {
                task.set_property(prop_key, value.clone());

                let new_content = parser::serialize(&doc);
                fs::write(&cli.file, new_content).context("Failed to write frump.md file")?;

                println!("Set {} = {} on task {}", property, value, id);
            } else {
                println!("Task {} not found.", id);
            }
        }

        Commands::History { id } => {
            let repo = FrumpRepo::open(".").context("Not in a git repository")?;
            let task_id = TaskId::new(*id)?;
            let history = repo.task_history(task_id)?;

            if history.commits.is_empty() {
                println!("No history found for task {}", id);
            } else {
                println!("History for Task {}:\n", id);
                for commit in &history.commits {
                    let change_icon = match commit.change_type {
                        ChangeType::Created => "✓ Created",
                        ChangeType::Modified => "• Modified",
                        ChangeType::Deleted => "✗ Deleted",
                    };

                    println!("{} by {} on {}", change_icon, commit.author, commit.date.format("%Y-%m-%d %H:%M"));
                    println!("  Commit: {}", &commit.hash[..8]);
                    if !commit.message.is_empty() {
                        // Show first line of commit message
                        let first_line = commit.message.lines().next().unwrap_or("");
                        println!("  Message: {}", first_line);
                    }
                    println!();
                }
            }
        }

        Commands::Closed => {
            let repo = FrumpRepo::open(".").context("Not in a git repository")?;
            let deleted = repo.deleted_tasks()?;

            if deleted.is_empty() {
                println!("No closed tasks found.");
            } else {
                println!("Closed tasks:\n");
                for (id, task_type, subject) in &deleted {
                    println!("{} {} - {}", task_type, id, subject);
                }
                println!("\nTotal: {} closed tasks", deleted.len());
            }
        }

        Commands::Update { id, subject, body } => {
            if subject.is_none() && body.is_none() {
                println!("Error: At least one of --subject or --body must be provided");
                return Ok(());
            }

            let content = fs::read_to_string(&cli.file).context("Failed to read frump.md file")?;
            let mut doc = parser::parse(&content)?;

            let task_id = TaskId::new(*id)?;
            if let Some(task) = doc.tasks.find_by_id_mut(task_id) {
                if let Some(new_subject) = subject {
                    task.subject = new_subject.clone();
                    println!("Updated subject for task {}", id);
                }
                if let Some(new_body) = body {
                    task.set_body(new_body.clone());
                    println!("Updated body for task {}", id);
                }

                let new_content = parser::serialize(&doc);
                fs::write(&cli.file, new_content).context("Failed to write frump.md file")?;
            } else {
                println!("Task {} not found.", id);
            }
        }

        Commands::Search { query, full } => {
            let content = fs::read_to_string(&cli.file).context("Failed to read frump.md file")?;
            let doc = parser::parse(&content)?;

            let query_lower = query.to_lowercase();
            let mut found = Vec::new();

            for task in doc.tasks.tasks() {
                let mut matches = false;

                // Search in subject
                if task.subject.to_lowercase().contains(&query_lower) {
                    matches = true;
                }

                // Search in body if --full flag is set
                if *full && task.body.to_lowercase().contains(&query_lower) {
                    matches = true;
                }

                if matches {
                    found.push(task);
                }
            }

            if found.is_empty() {
                println!("No tasks found matching '{}'", query);
            } else {
                println!("Found {} task(s) matching '{}':\n", found.len(), query);
                for task in found {
                    println!("{} {} - {}", task.task_type, task.id, task.subject);
                    if *full && !task.body.is_empty() {
                        // Show a snippet of the body
                        let snippet = task.body.lines().take(2).collect::<Vec<_>>().join(" ");
                        let truncated = if snippet.len() > 80 {
                            format!("{}...", &snippet[..80])
                        } else {
                            snippet
                        };
                        println!("  {}", truncated);
                    }
                }
            }
        }

        Commands::Stats => {
            let content = fs::read_to_string(&cli.file).context("Failed to read frump.md file")?;
            let doc = parser::parse(&content)?;

            let total = doc.tasks.len();

            // Count by type
            let mut type_counts = std::collections::HashMap::new();
            for task in doc.tasks.tasks() {
                *type_counts.entry(task.task_type.as_str()).or_insert(0) += 1;
            }

            // Count by status
            let mut status_counts = std::collections::HashMap::new();
            let mut no_status = 0;
            for task in doc.tasks.tasks() {
                if let Some(status) = task.status() {
                    *status_counts.entry(status).or_insert(0) += 1;
                } else {
                    no_status += 1;
                }
            }

            // Count by assignee
            let mut assignee_counts = std::collections::HashMap::new();
            let mut no_assignee = 0;
            for task in doc.tasks.tasks() {
                if let Some(assignee) = task.assignee() {
                    *assignee_counts.entry(assignee).or_insert(0) += 1;
                } else {
                    no_assignee += 1;
                }
            }

            println!("Task Statistics\n");
            println!("Total tasks: {}\n", total);

            println!("By Type:");
            let mut types: Vec<_> = type_counts.iter().collect();
            types.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
            for (task_type, count) in types {
                println!("  {}: {}", task_type, count);
            }

            println!("\nBy Status:");
            if !status_counts.is_empty() {
                let mut statuses: Vec<_> = status_counts.iter().collect();
                statuses.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
                for (status, count) in statuses {
                    println!("  {}: {}", status, count);
                }
            }
            if no_status > 0 {
                println!("  (no status): {}", no_status);
            }

            println!("\nBy Assignee:");
            if !assignee_counts.is_empty() {
                let mut assignees: Vec<_> = assignee_counts.iter().collect();
                assignees.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
                for (assignee, count) in assignees {
                    println!("  {}: {}", assignee, count);
                }
            }
            if no_assignee > 0 {
                println!("  (no assignee): {}", no_assignee);
            }

            // Optional: show closed tasks count if in git repo
            if let Ok(repo) = FrumpRepo::open(".") {
                if let Ok(deleted) = repo.deleted_tasks() {
                    println!("\nClosed tasks: {}", deleted.len());
                }
            }
        }

        Commands::Validate => {
            let content = fs::read_to_string(&cli.file).context("Failed to read frump.md file")?;

            match parser::parse(&content) {
                Ok(doc) => {
                    println!("✓ File structure is valid");

                    // Check for duplicate IDs
                    let mut ids = std::collections::HashSet::new();
                    let mut duplicates = Vec::new();
                    for task in doc.tasks.tasks() {
                        if !ids.insert(task.id) {
                            duplicates.push(task.id);
                        }
                    }

                    if !duplicates.is_empty() {
                        println!("✗ Found duplicate task IDs: {:?}", duplicates);
                    } else {
                        println!("✓ All task IDs are unique");
                    }

                    // Check for sequential IDs
                    let mut ids_vec: Vec<_> = doc.tasks.tasks().iter().map(|t| t.id.value()).collect();
                    ids_vec.sort();
                    let mut gaps = Vec::new();
                    for i in 1..ids_vec.len() {
                        if ids_vec[i] > ids_vec[i - 1] + 1 {
                            gaps.push((ids_vec[i - 1] + 1, ids_vec[i] - 1));
                        }
                    }

                    if !gaps.is_empty() {
                        println!("⚠ ID gaps found (possibly closed tasks):");
                        for (start, end) in gaps {
                            if start == end {
                                println!("  ID {}", start);
                            } else {
                                println!("  IDs {}-{}", start, end);
                            }
                        }
                    } else {
                        println!("✓ Task IDs are sequential");
                    }

                    // Validate team emails
                    // Email is already validated by the Email type during parsing

                    println!("\n✓ Validation complete: {} tasks, {} team members",
                             doc.tasks.len(),
                             doc.team.len());
                }
                Err(e) => {
                    println!("✗ Validation failed: {}", e);
                }
            }
        }

        Commands::Export { format, output } => {
            let content = fs::read_to_string(&cli.file).context("Failed to read frump.md file")?;
            let doc = parser::parse(&content)?;

            let exported = match format.to_lowercase().as_str() {
                "json" => export_json(&doc)?,
                "csv" => export_csv(&doc)?,
                _ => {
                    println!("Error: Unknown format '{}'. Supported formats: json, csv", format);
                    return Ok(());
                }
            };

            if let Some(output_path) = output {
                fs::write(output_path, &exported)
                    .context("Failed to write output file")?;
                println!("Exported {} tasks to {:?}", doc.tasks.len(), output_path);
            } else {
                println!("{}", exported);
            }
        }

        Commands::Import { file, merge } => {
            let import_content = fs::read_to_string(file)
                .context("Failed to read import file")?;

            let imported_doc = import_json(&import_content)?;

            if *merge {
                // Merge: add imported tasks to existing document
                let current_content = fs::read_to_string(&cli.file)
                    .context("Failed to read current frump.md")?;
                let mut current_doc = parser::parse(&current_content)?;

                // Find next available ID
                let mut next_id = current_doc.tasks.max_id()
                    .map(|id| id.value() + 1)
                    .unwrap_or(1);

                // Add imported tasks with new IDs
                let mut added = 0;
                for task in imported_doc.tasks.tasks() {
                    let new_id = TaskId::new(next_id)?;
                    let mut new_task = Task::new(
                        new_id,
                        task.task_type.clone(),
                        task.subject.clone(),
                    );
                    new_task.set_body(task.body.clone());

                    for prop in &task.properties {
                        new_task.add_property(prop.key.clone(), prop.value.clone());
                    }

                    current_doc.tasks.add(new_task);
                    next_id += 1;
                    added += 1;
                }

                let new_content = parser::serialize(&current_doc);
                fs::write(&cli.file, new_content)
                    .context("Failed to write frump.md")?;

                println!("Merged {} tasks into frump.md", added);
            } else {
                // Replace: overwrite with imported document
                let new_content = parser::serialize(&imported_doc);
                fs::write(&cli.file, new_content)
                    .context("Failed to write frump.md")?;

                println!("Imported {} tasks, {} team members",
                         imported_doc.tasks.len(),
                         imported_doc.team.len());
            }
        }

        Commands::Template { action } => {
            let manager = TemplateManager::new();

            match action {
                TemplateAction::Add { name, task_type, subject, body } => {
                    let template = TaskTemplate {
                        name: name.clone(),
                        task_type: task_type.clone(),
                        subject_template: subject.clone(),
                        body_template: body.clone().unwrap_or_default(),
                        properties: std::collections::HashMap::new(),
                    };

                    manager.add(template)?;
                    println!("Added template '{}'", name);
                }

                TemplateAction::List => {
                    let templates = manager.list()?;

                    if templates.is_empty() {
                        println!("No templates found.");
                    } else {
                        println!("Available templates:\n");
                        for template in templates {
                            println!("{} ({})", template.name, template.task_type);
                            println!("  Subject: {}", template.subject_template);
                            if !template.body_template.is_empty() {
                                println!("  Body: {}", template.body_template);
                            }
                            println!();
                        }
                    }
                }

                TemplateAction::Remove { name } => {
                    manager.remove(name)?;
                    println!("Removed template '{}'", name);
                }

                TemplateAction::Show { name } => {
                    let template = manager.get(name)?;
                    println!("Template: {}", template.name);
                    println!("Type: {}", template.task_type);
                    println!("Subject: {}", template.subject_template);
                    if !template.body_template.is_empty() {
                        println!("Body: {}", template.body_template);
                    }
                    if !template.properties.is_empty() {
                        println!("Properties:");
                        for (key, value) in &template.properties {
                            println!("  {}: {}", key, value);
                        }
                    }
                }
            }
        }

        Commands::Bulk { action } => {
            let content = fs::read_to_string(&cli.file).context("Failed to read frump.md file")?;
            let mut doc = parser::parse(&content)?;

            match action {
                BulkAction::CloseByStatus { status } => {
                    let tasks_to_close: Vec<TaskId> = doc
                        .tasks
                        .tasks()
                        .iter()
                        .filter(|t| t.status().map(|s| s == status).unwrap_or(false))
                        .map(|t| t.id)
                        .collect();

                    if tasks_to_close.is_empty() {
                        println!("No tasks found with status '{}'", status);
                        return Ok(());
                    }

                    let count = tasks_to_close.len();
                    for id in tasks_to_close {
                        doc.tasks.remove(id);
                    }

                    let new_content = parser::serialize(&doc);
                    fs::write(&cli.file, new_content).context("Failed to write frump.md")?;

                    println!("Closed {} task(s) with status '{}'", count, status);
                }

                BulkAction::AssignByType { task_type, assignee } => {
                    let filter_type = TaskType::parse(task_type);
                    let mut count = 0;

                    for task in doc.tasks.tasks_mut() {
                        if task.task_type == filter_type {
                            task.set_assignee(assignee.clone());
                            count += 1;
                        }
                    }

                    if count == 0 {
                        println!("No tasks found with type '{}'", task_type);
                        return Ok(());
                    }

                    let new_content = parser::serialize(&doc);
                    fs::write(&cli.file, new_content).context("Failed to write frump.md")?;

                    println!("Assigned {} task(s) of type '{}' to {}", count, task_type, assignee);
                }

                BulkAction::SetByStatus { status, property, value } => {
                    let prop_key = PropertyKey::new(property)?;
                    let mut count = 0;

                    for task in doc.tasks.tasks_mut() {
                        if task.status().map(|s| s == status).unwrap_or(false) {
                            task.set_property(prop_key.clone(), value.clone());
                            count += 1;
                        }
                    }

                    if count == 0 {
                        println!("No tasks found with status '{}'", status);
                        return Ok(());
                    }

                    let new_content = parser::serialize(&doc);
                    fs::write(&cli.file, new_content).context("Failed to write frump.md")?;

                    println!("Set {} = {} on {} task(s) with status '{}'",
                             property, value, count, status);
                }
            }
        }

        Commands::CheckConflicts => {
            let content = fs::read_to_string(&cli.file).context("Failed to read frump.md file")?;
            let doc = parser::parse(&content)?;

            // Find duplicate IDs
            let mut id_occurrences: std::collections::HashMap<TaskId, Vec<&Task>> = std::collections::HashMap::new();
            for task in doc.tasks.tasks() {
                id_occurrences.entry(task.id).or_insert_with(Vec::new).push(task);
            }

            let duplicates: Vec<_> = id_occurrences
                .iter()
                .filter(|(_, tasks)| tasks.len() > 1)
                .collect();

            if duplicates.is_empty() {
                println!("✓ No duplicate task IDs found");
                println!("✓ File is ready for merge");
            } else {
                println!("✗ Found {} duplicate task ID(s):\n", duplicates.len());
                for (id, tasks) in duplicates {
                    println!("ID {}:", id);
                    for task in tasks {
                        println!("  - {} {}: {}", task.task_type, id, task.subject);
                    }
                    println!();
                }
                println!("Run 'frump resolve-conflicts' to automatically renumber conflicts");
            }
        }

        Commands::ResolveConflicts { commit } => {
            let content = fs::read_to_string(&cli.file).context("Failed to read frump.md file")?;
            let mut doc = parser::parse(&content)?;

            // Find duplicate IDs
            let mut id_occurrences: std::collections::HashMap<TaskId, Vec<usize>> = std::collections::HashMap::new();
            for (idx, task) in doc.tasks.tasks().iter().enumerate() {
                id_occurrences.entry(task.id).or_insert_with(Vec::new).push(idx);
            }

            let duplicates: Vec<_> = id_occurrences
                .iter()
                .filter(|(_, indices)| indices.len() > 1)
                .collect();

            if duplicates.is_empty() {
                println!("✓ No duplicate task IDs found");
                println!("Nothing to resolve.");
                return Ok(());
            }

            // Find the maximum ID in the document
            let max_id = doc.tasks.max_id().ok_or_else(|| anyhow::anyhow!("No tasks found"))?;
            let mut next_id = max_id.next();

            // Renumber conflicting tasks (keep first occurrence, renumber the rest)
            let mut renumbered = Vec::new();
            for (_dup_id, indices) in duplicates {
                // Skip the first occurrence (keep original ID)
                for &idx in indices.iter().skip(1) {
                    let task = &mut doc.tasks.tasks_mut()[idx];
                    let old_id = task.id;
                    task.id = next_id;
                    renumbered.push((old_id, next_id, task.subject.clone()));
                    next_id = next_id.next();
                }
            }

            // Write back to file
            let new_content = parser::serialize(&doc);
            fs::write(&cli.file, new_content).context("Failed to write frump.md")?;

            println!("✓ Resolved {} duplicate task ID(s):\n", renumbered.len());
            for (old_id, new_id, subject) in &renumbered {
                println!("  {} → {}: {}", old_id, new_id, subject);
            }

            if *commit {
                // Create a git commit
                let commit_message = format!(
                    "Resolve task ID conflicts\n\nRenumbered {} conflicting task(s)",
                    renumbered.len()
                );

                // Stage the frump.md file
                let status = std::process::Command::new("git")
                    .args(&["add", cli.file.to_str().unwrap()])
                    .status()
                    .context("Failed to stage file with git")?;

                if !status.success() {
                    println!("\n✗ Failed to stage changes");
                    return Ok(());
                }

                // Create commit
                let status = std::process::Command::new("git")
                    .args(&["commit", "-m", &commit_message])
                    .status()
                    .context("Failed to create git commit")?;

                if status.success() {
                    println!("\n✓ Changes committed automatically");
                } else {
                    println!("\n⚠ Changes saved but commit failed");
                    println!("You may need to commit manually");
                }
            } else {
                println!("\nRemember to commit these changes.");
                println!("Run with --commit flag to commit automatically.");
            }
        }
    }

    Ok(())
}
