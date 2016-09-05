use std::collections::HashMap;

use author::Author;

#[derive(Debug)]
pub struct Task {
    pub id: String,
    pub title: String,
    description: String,
    assignees: Vec<Author>,
    properties: HashMap<String, String>,
}

impl Task {
    pub fn new(title: String) -> Task {
        Task {
            id: "".to_string(),
            title: title,
            description: "".to_string(),
            assignees: Vec::new(),
            properties: HashMap::new(),
        }
    }
}
