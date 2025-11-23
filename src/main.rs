use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

use frump::{parser, ChangeType, FrumpRepo, PropertyKey, Task, TaskId, TaskType};

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
    }

    Ok(())
}
