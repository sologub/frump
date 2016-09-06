use std::collections::HashMap;
use std::str::FromStr;

use author::Author;

#[derive(Debug)]
pub struct Task {
    pub task_type: String,
    pub id: u32,
    pub title: String,
    description: String,
    assignees: Vec<Author>,
    properties: HashMap<String, String>,
}

impl Task {
    pub fn new(lines: &Vec<&str>) -> Task {
        let subject = lines[0][4..].to_string();

        let words: Vec<&str> = lines[0][4..].split_whitespace().collect();

        let task_type = words[0].to_string();

        let id_str = words[1];
        let id: u32 = FromStr::from_str(id_str).unwrap();

        let title = if words[2] == "-" {
            let index = subject.find('-').unwrap();
            subject[index+1..].trim().to_string()
        } else {
            let index = subject.find(id_str).unwrap();
            subject[(index + id_str.len() + 1)..].trim().to_string()
        };

        Task {
            id: id,
            title: title,
            task_type: task_type,
            description: "".to_string(),
            assignees: Vec::new(),
            properties: HashMap::new(),
        }
    }
}
