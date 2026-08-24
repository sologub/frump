use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

use frump::{export_csv, export_json, import_json, parser, ChangeType, FrumpRepo, PropertyKey, Task, TaskId, TaskType, TaskTemplate, TemplateManager};

#[derive(Parser)]
#[command(name = "frump")]
#[command(about = "Distributed task management tool based on Git and Markdown", long_about = None)]
#[command(
    after_help = "AI agents: run `frump usage` for the complete usage guide.\nBuild from source with `cargo build --release`."
)]
struct Cli {
    #[arg(short, long, global = true, default_value = "frump.md")]
    file: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print the complete usage guide (USAGE.md)
    Usage,

    /// Start a local Kanban board for the task file
    Web {
        /// TCP port for the loopback-only web server
        #[arg(long, default_value_t = 3000)]
        port: u16,
    },

    /// Commit the tracked task file with a short Git message
    Commit {
        /// Commit message
        #[arg(short, long, default_value = "Update frump tasks")]
        message: String,
    },

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

    /// Remove a property from a task, including Status
    Unset { id: u32, property: String },

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

        /// Append text to the task body instead of replacing it
        #[arg(long, conflicts_with = "body")]
        append_body: Option<String>,
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

    /// Create a new empty Frump board
    Init {
        #[arg(long, default_value = "My Project")]
        title: String,
    },

    /// Show the dependency tree for a task
    Deps { id: u32 },

    /// List active tasks that depend on a task
    Dependents { id: u32 },

    /// List tasks whose active dependencies are satisfied
    Ready,

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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let _write_lock = if cli.file.exists() && matches!(cli.command, Commands::Add { .. } | Commands::Close { .. } | Commands::Assign { .. } | Commands::Set { .. } | Commands::Unset { .. } | Commands::Update { .. } | Commands::Import { .. } | Commands::Bulk { .. } | Commands::ResolveConflicts { .. }) {
        Some(acquire_write_lock(&cli.file)?)
    } else { None };

    match &cli.command {
        Commands::Usage => {
            print!("{}", include_str!("../USAGE.md"));
        }

        Commands::Web { port } => {
            frump::web::serve(cli.file.clone(), *port).await?;
        }

        Commands::Commit { message } => {
            commit_task_file(&cli.file, message)?;
            println!("Committed {}", cli.file.display());
        }

        Commands::List {
            task_type,
            status,
            assignee,
        } => {
            let content = fs::read_to_string(&cli.file).map_err(|error| {
                if error.kind() == std::io::ErrorKind::NotFound {
                    anyhow::anyhow!("Task file {} does not exist. Initialize it with `frump init --file {}`.", cli.file.display(), cli.file.display())
                } else { anyhow::Error::from(error).context("Failed to read frump.md file") }
            })?;
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
                anyhow::bail!("Task {} not found.", id);
            }
        }

        Commands::Add {
            task_type,
            subject,
            body,
            assignee,
            status,
        } => {
            let content = fs::read_to_string(&cli.file).map_err(|error| {
                if error.kind() == std::io::ErrorKind::NotFound {
                    anyhow::anyhow!("Task file {} does not exist. Initialize it with `frump init --file {}`.", cli.file.display(), cli.file.display())
                } else { anyhow::Error::from(error).context("Failed to read frump.md file") }
            })?;
            let mut doc = parser::parse(&content)?;

            let query = normalize_search_text(subject);
            let candidates: Vec<_> = doc.tasks.tasks().iter().filter(|task| {
                all_query_tokens_match(&query, &task.subject, &task.body)
                    && search_similarity(&query, &task.subject).max(search_similarity(&query, &task.body)) >= 0.78
            }).collect();
            if !candidates.is_empty() {
                eprintln!("Warning: similar task(s) already exist:");
                for task in candidates { eprintln!("  {} {} - {}", task.task_type, task.id, task.subject); }
            }

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
                warn_if_new_status(&doc, s);
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
            if doc.tasks.find_by_id(task_id).is_some() {
                ensure_close_is_recoverable(&cli.file)?;
                let dependency_errors = validate_dependencies(&doc);
                if !dependency_errors.is_empty() { anyhow::bail!("Refusing to close: {}", dependency_errors.join("; ")); }
                ensure_close_ready(&doc, task_id)?;
                let task = doc.tasks.remove(task_id).expect("task checked above");
                let new_content = parser::serialize(&doc);
                fs::write(&cli.file, new_content).context("Failed to write frump.md file")?;

                println!("Closed {} {} - {}", task.task_type, task.id, task.subject);
                println!("\nRemember to commit this change with a descriptive message.");
            } else {
                anyhow::bail!("Task {} not found.", id);
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
                anyhow::bail!("Task {} not found.", id);
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

            if property == "Status" { warn_if_new_status(&doc, value); }

            if let Some(task) = doc.tasks.find_by_id_mut(task_id) {
                task.set_property(prop_key, value.clone());

                let new_content = parser::serialize(&doc);
                fs::write(&cli.file, new_content).context("Failed to write frump.md file")?;

                println!("Set {} = {} on task {}", property, value, id);
            } else {
                anyhow::bail!("Task {} not found.", id);
            }
        }

        Commands::Unset { id, property } => {
            let content = fs::read_to_string(&cli.file).context("Failed to read frump.md file")?;
            let mut doc = parser::parse(&content)?;
            let key = PropertyKey::new(property)?;
            let task = doc.tasks.find_by_id_mut(TaskId::new(*id)?).ok_or_else(|| anyhow::anyhow!("Task {} not found.", id))?;
            if task.get_property(&key).is_none() { anyhow::bail!("Task {} has no {} property.", id, property); }
            task.remove_property(&key);
            fs::write(&cli.file, parser::serialize(&doc)).context("Failed to write frump.md file")?;
            println!("Removed {} from task {}", property, id);
        }

        Commands::History { id } => {
            let repo = FrumpRepo::open(".").map_err(|_| anyhow::anyhow!("Task history requires a Git repository containing tracked frump.md history."))?;
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

        Commands::Update { id, subject, body, append_body } => {
            if subject.is_none() && body.is_none() && append_body.is_none() {
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
                if let Some(extra) = append_body {
                    let combined = if task.body.is_empty() { extra.clone() } else { format!("{}\n\n{}", task.body, extra) };
                    task.set_body(combined);
                    println!("Appended body for task {}", id);
                }

                let new_content = parser::serialize(&doc);
                fs::write(&cli.file, new_content).context("Failed to write frump.md file")?;
            } else {
                anyhow::bail!("Task {} not found.", id);
            }
        }

        Commands::Search { query, full } => {
            let content = fs::read_to_string(&cli.file).context("Failed to read frump.md file")?;
            let doc = parser::parse(&content)?;

            let normalized_query = normalize_search_text(query);
            let mut found = Vec::new();

            for task in doc.tasks.tasks() {
                if !all_query_tokens_match(&normalized_query, &task.subject, &task.body) {
                    continue;
                }
                let subject_score = search_similarity(&normalized_query, &task.subject);
                let body_score = search_similarity(&normalized_query, &task.body);
                let score = subject_score.max(body_score);
                if score >= 0.78 {
                    found.push((task, score, subject_score >= body_score));
                }
            }

            found.sort_by(|left, right| right.1.total_cmp(&left.1).then_with(|| right.2.cmp(&left.2)));

            if found.is_empty() {
                println!("No tasks found matching '{}'", query);
            } else {
                println!("Found {} similar task(s) matching '{}':\n", found.len(), query);
                for (task, score, subject_match) in found {
                    println!("{} {} - {}", task.task_type, task.id, task.subject);
                    println!("  Similarity: {:.0}% ({})", score * 100.0, if subject_match { "subject" } else { "body" });
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

                    let dependency_errors = validate_dependencies(&doc);
                    if dependency_errors.is_empty() {
                        println!("✓ Dependencies resolve and are acyclic");
                    } else {
                        for error in dependency_errors { println!("✗ {}", error); }
                        anyhow::bail!("Dependency validation failed");
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

        Commands::Init { title } => {
            if cli.file.exists() { anyhow::bail!("{} already exists; refusing to overwrite it.", cli.file.display()); }
            fs::write(&cli.file, format!("# {}\n\n## Team\n\n## Tasks\n", title)).with_context(|| format!("Failed to initialize {}", cli.file.display()))?;
            println!("Initialized {}", cli.file.display());
        }

        Commands::Deps { id } => {
            let doc = read_document(&cli.file)?;
            print_dependency_tree(&doc, TaskId::new(*id)?, 0, &mut std::collections::HashSet::new())?;
        }

        Commands::Dependents { id } => {
            let doc = read_document(&cli.file)?;
            let id = TaskId::new(*id)?;
            for task in doc.tasks.tasks().iter().filter(|task| dependency_ids(task).iter().any(|dependency| *dependency == id)) {
                println!("{} {} - {}", task.task_type, task.id, task.subject);
            }
        }

        Commands::Ready => {
            let doc = read_document(&cli.file)?;
            for task in doc.tasks.tasks().iter().filter(|task| dependency_ids(task).iter().all(|dependency| doc.tasks.find_by_id(*dependency).map(|prerequisite| prerequisite.status() == Some("done")).unwrap_or(true))) {
                println!("{} {} - {}", task.task_type, task.id, task.subject);
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

fn ensure_close_is_recoverable(file: &Path) -> Result<()> {
    let root_output = std::process::Command::new("git")
        .current_dir(file.parent().unwrap_or_else(|| Path::new(".")))
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("Failed to locate the Git repository for task closure")?;
    if !root_output.status.success() {
        anyhow::bail!("Refusing to close a task: {} is not in a Git repository.", file.display());
    }
    let root = PathBuf::from(String::from_utf8_lossy(&root_output.stdout).trim());
    let absolute_file = file.canonicalize().with_context(|| format!("Failed to resolve {}", file.display()))?;
    let repository_path = absolute_file.strip_prefix(&root)
        .map_err(|_| anyhow::anyhow!("Refusing to close a task: {} is outside the current Git repository.", file.display()))?;
    let status = std::process::Command::new("git")
        .current_dir(&root)
        .args(["ls-files", "--error-unmatch", "--"])
        .arg(repository_path)
        .stderr(std::process::Stdio::null())
        .status()
        .context("Failed to verify whether the task file is tracked by Git")?;

    if !status.success() {
        anyhow::bail!(
            "Refusing to close a task: {} is not tracked by Git, so its closure cannot be recovered from history.",
            file.display()
        );
    }
    Ok(())
}

fn commit_task_file(file: &Path, message: &str) -> Result<()> {
    let parent = file.parent().unwrap_or_else(|| Path::new("."));
    let name = file.file_name().ok_or_else(|| anyhow::anyhow!("Task file path has no filename"))?;
    let add = std::process::Command::new("git").current_dir(parent).args(["add", "--"]).arg(name).status()
        .context("Failed to stage task file")?;
    if !add.success() { anyhow::bail!("Git could not stage {}", file.display()); }
    let commit = std::process::Command::new("git").current_dir(parent).args(["commit", "-m", message, "--", name.to_str().unwrap_or("frump.md")]).status()
        .context("Failed to commit task file")?;
    if !commit.success() { anyhow::bail!("Git could not create a commit for {}", file.display()); }
    Ok(())
}

fn ensure_close_ready(doc: &frump::FrumpDoc, id: TaskId) -> Result<()> {
    let task = doc.tasks.find_by_id(id).ok_or_else(|| anyhow::anyhow!("Task {} not found.", id))?;
    if task.status() != Some("done") {
        anyhow::bail!("Refusing to close task {}: Status must be done.", id);
    }
    for dependency in dependency_ids(task) {
        if let Some(prerequisite) = doc.tasks.find_by_id(dependency) {
            if prerequisite.status() != Some("done") {
                anyhow::bail!("Refusing to close task {}: dependency {} is not done.", id, dependency);
            }
        }
    }
    Ok(())
}

fn normalize_search_text(value: &str) -> String {
    value
        .chars()
        .map(|character| if matches!(character, '-' | '_') { ' ' } else { character })
        .collect::<String>()
        .to_lowercase()
}

fn warn_if_new_status(doc: &frump::FrumpDoc, status: &str) {
    let existing: std::collections::HashSet<_> = doc.tasks.tasks().iter().filter_map(|task| task.status()).collect();
    if !existing.contains(status) {
        eprintln!("Warning: '{}' is a new status. Prefer an existing status unless there is a documented reason.", status);
    }
}

fn read_document(file: &Path) -> Result<frump::FrumpDoc> {
    parser::parse(&fs::read_to_string(file).with_context(|| format!("Failed to read {}", file.display()))?)
}

fn dependency_ids(task: &Task) -> Vec<TaskId> {
    task.get_property(&PropertyKey::new("Depends On").expect("constant is valid"))
        .map(|value| value.split(',').filter_map(|id| id.trim().parse::<u32>().ok()).filter_map(|id| TaskId::new(id).ok()).collect())
        .unwrap_or_default()
}

fn validate_dependencies(doc: &frump::FrumpDoc) -> Vec<String> {
    let ids: std::collections::HashSet<_> = doc.tasks.tasks().iter().map(|task| task.id).collect();
    let closed: std::collections::HashSet<_> = FrumpRepo::open(".").ok().and_then(|repo| repo.deleted_tasks().ok()).unwrap_or_default().into_iter().map(|(id, _, _)| id).collect();
    let mut errors = Vec::new();
    for task in doc.tasks.tasks() {
        if let Some(value) = task.get_property(&PropertyKey::new("Depends On").expect("constant is valid")) {
            for raw in value.split(',') {
                if raw.trim().is_empty() || raw.trim().parse::<u32>().ok().and_then(|id| TaskId::new(id).ok()).is_none() {
                    errors.push(format!("Task {} has invalid Depends On value '{}'", task.id, raw.trim()));
                }
            }
        }
        for dependency in dependency_ids(task) {
            if dependency == task.id { errors.push(format!("Task {} depends on itself", task.id)); }
            else if !ids.contains(&dependency) && !closed.contains(&dependency) { errors.push(format!("Task {} references unknown dependency {}", task.id, dependency)); }
        }
    }
    fn visit(id: TaskId, doc: &frump::FrumpDoc, visiting: &mut std::collections::HashSet<TaskId>, visited: &mut std::collections::HashSet<TaskId>) -> bool {
        if visited.contains(&id) { return false; }
        if !visiting.insert(id) { return true; }
        let cyclic = doc.tasks.find_by_id(id).map(|task| dependency_ids(task).into_iter().filter(|dependency| doc.tasks.find_by_id(*dependency).is_some()).any(|dependency| visit(dependency, doc, visiting, visited))).unwrap_or(false);
        visiting.remove(&id); visited.insert(id); cyclic
    }
    let mut visiting = std::collections::HashSet::new(); let mut visited = std::collections::HashSet::new();
    for task in doc.tasks.tasks() { if visit(task.id, doc, &mut visiting, &mut visited) { errors.push("Dependency cycle detected".to_string()); break; } }
    errors
}

fn print_dependency_tree(doc: &frump::FrumpDoc, id: TaskId, depth: usize, seen: &mut std::collections::HashSet<TaskId>) -> Result<()> {
    let task = doc.tasks.find_by_id(id).ok_or_else(|| anyhow::anyhow!("Task {} not found.", id))?;
    println!("{}{} {} - {}", "  ".repeat(depth), task.task_type, task.id, task.subject);
    if !seen.insert(id) { return Ok(()); }
    for dependency in dependency_ids(task) { print_dependency_tree(doc, dependency, depth + 1, seen)?; }
    Ok(())
}

fn search_similarity(query: &str, value: &str) -> f64 {
    let value = normalize_search_text(value);
    if value.contains(query) {
        return 1.0;
    }
    let query_words: Vec<_> = query.split_whitespace().collect();
    let words: Vec<_> = value.split_whitespace().collect();
    if query_words.is_empty() || words.is_empty() {
        return 0.0;
    }
    let window = query_words.len();
    (1..=words.len())
        .map(|start| {
            let end = (start + window).min(words.len());
            strsim::jaro_winkler(query, &words[start - 1..end].join(" "))
        })
        .fold(0.0, f64::max)
}

fn all_query_tokens_match(query: &str, subject: &str, body: &str) -> bool {
    let query_tokens: Vec<_> = query.split_whitespace().collect();
    let searchable = format!("{} {}", normalize_search_text(subject), normalize_search_text(body));
    let searchable_tokens: Vec<_> = searchable.split_whitespace().collect();
    query_tokens.iter().all(|query_token| {
        searchable_tokens.iter().any(|candidate| strsim::normalized_levenshtein(query_token, candidate) >= 0.80)
    })
}

fn acquire_write_lock(file: &Path) -> Result<std::fs::File> {
    use fs2::FileExt;
    let lock = OpenOptions::new().read(true).write(true).open(file)
        .with_context(|| format!("Failed to open {} for locking", file.display()))?;
    lock.lock_exclusive().with_context(|| format!("Failed to acquire write lock for {}", file.display()))?;
    Ok(lock)
}
