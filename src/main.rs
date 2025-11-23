use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

use frump::{parser, PropertyKey, Task, TaskId, TaskType};

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

            // Find the next available task ID
            let next_id = doc.tasks.next_id();

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
    }

    Ok(())
}
