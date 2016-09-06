pub mod task;
pub mod author;

use std::io::prelude::*;
use std::str::*;
use std::fs::File;

use task::Task;

fn get_task_lines(lines: Lines) -> Vec<Vec<&str>> {
    let mut result = vec![];
    let mut task_lines = vec![];
    let mut is_inside_task_section = false;
    let mut is_inside_task = false;

    for (_, line) in lines.enumerate() {
        if line.trim().to_uppercase().starts_with("## TASKS") {
            is_inside_task_section = true;
        } else if line.trim().starts_with("## ") {
            is_inside_task_section = false;
        } else if is_inside_task_section {
            if line.trim().starts_with("### ") {
                is_inside_task = true;
            }
            if is_inside_task {
                if line.trim().starts_with("### ") && task_lines.len() > 0 {
                    result.push(task_lines.clone());
                    task_lines.clear();
                }
                task_lines.push(line);
            }
        }
    }

    // add last task
    if task_lines.len() > 0 {
        result.push(task_lines);
    }

    result
}

fn load_tasks(file_body: String) -> Vec<Task> {
    let mut tasks = vec![];
    let lines = get_task_lines(file_body.lines());

    for task_lines in lines {
        tasks.push(Task::new(&task_lines));
    }
    tasks
}

fn main() {
    let file_name = "frump.md".to_string();

    let mut file = File::open(file_name).unwrap();
    let mut file_body = String::new();
    file.read_to_string(&mut file_body).unwrap();

    let tasks = load_tasks(file_body);
    for task in tasks {
        println!("{} {} - {}", task.task_type, task.id, task.title);
    }
}
