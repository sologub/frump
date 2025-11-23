use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

mod parser;
use parser::{FrumpDoc, Task};

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
    List,

    /// Show details of a specific task
    Show {
        /// Task ID
        id: u32,
    },

    /// Add a new task
    Add {
        /// Task type (e.g., Task, Bug, Issue)
        #[arg(short, long, default_value = "Task")]
        task_type: String,

        /// Task subject/title
        subject: String,

        /// Task body/description (optional)
        #[arg(short, long)]
        body: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::List => {
            let content = fs::read_to_string(&cli.file)
                .context("Failed to read frump.md file")?;
            let doc = FrumpDoc::parse(&content)?;

            if doc.tasks.is_empty() {
                println!("No tasks found.");
            } else {
                for task in &doc.tasks {
                    println!("{} {} - {}", task.task_type, task.id, task.subject);
                    if let Some(status) = &task.properties.get("Status") {
                        println!("  Status: {}", status);
                    }
                }
            }
        }

        Commands::Show { id } => {
            let content = fs::read_to_string(&cli.file)
                .context("Failed to read frump.md file")?;
            let doc = FrumpDoc::parse(&content)?;

            if let Some(task) = doc.tasks.iter().find(|t| t.id == *id) {
                println!("### {} {} - {}\n", task.task_type, task.id, task.subject);

                if !task.body.is_empty() {
                    println!("{}\n", task.body);
                }

                if !task.properties.is_empty() {
                    for (key, value) in &task.properties {
                        println!("{}: {}", key, value);
                    }
                }
            } else {
                println!("Task {} not found.", id);
            }
        }

        Commands::Add { task_type, subject, body } => {
            let content = fs::read_to_string(&cli.file)
                .context("Failed to read frump.md file")?;
            let mut doc = FrumpDoc::parse(&content)?;

            // Find the next available task ID
            let next_id = doc.tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1;

            let new_task = Task {
                id: next_id,
                task_type: task_type.clone(),
                subject: subject.clone(),
                body: body.clone().unwrap_or_default(),
                properties: std::collections::HashMap::new(),
            };

            doc.tasks.push(new_task);

            // Write back to file
            let new_content = doc.to_string();
            fs::write(&cli.file, new_content)
                .context("Failed to write frump.md file")?;

            println!("Added {} {} - {}", task_type, next_id, subject);
        }
    }

    Ok(())
}
