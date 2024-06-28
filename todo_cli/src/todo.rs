use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use chrono::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Todo {
    pub title: String,
    #[serde(default)]
    pub content: String,
    #[serde(default = "default_priority")]
    pub priority: String,
    #[serde(default = "default_date_time")]
    pub date_time: String,
    #[serde(default)]
    pub deadline: String,
    #[serde(default)]
    pub done: bool,
    #[serde(default)]
    pub subtasks: Vec<Todo>,
    #[serde(default)]
    pub expanded: bool,
}

impl Default for Todo {
    fn default() -> Self {
        Todo {
            title: String::new(),
            content: String::new(),
            priority: default_priority(),
            date_time: default_date_time(),
            deadline: String::new(),
            done: false,
            subtasks: vec![],
            expanded: false,
        }
    }
}

fn default_priority() -> String {
    "low".to_string()
}

fn default_date_time() -> String {
    Utc::now().to_rfc3339()
}

impl Todo {
    pub fn new(title: String, content: String, priority: String, deadline: String) -> Self {
        Todo {
            title,
            content,
            priority,
            date_time: Utc::now().to_rfc3339(),
            deadline,
            done: false,
            subtasks: vec![],
            expanded: false,
        }
    }

    pub fn add_subtask(&mut self, title: String) {
        self.subtasks.push(Todo::new(title, String::new(), "low".to_string(), String::new()));
    }

    pub fn completion_rate(&self) -> f32 {
        if self.subtasks.is_empty() {
            return if self.done { 100.0 } else { 0.0 };
        }
        let completed_subtasks = self.subtasks.iter().filter(|t| t.done).count() as f32;
        (completed_subtasks / self.subtasks.len() as f32) * 100.0
    }
}

const DB_FILE: &str = "todos.json";

pub fn load_todos() -> Vec<Todo> {
    if Path::new(DB_FILE).exists() {
        let data = fs::read_to_string(DB_FILE).expect("Unable to read file");
        serde_json::from_str(&data).expect("Unable to parse JSON")
    } else {
        vec![]
    }
}

pub fn save_todos(todos: &Vec<Todo>) {
    let data = serde_json::to_string_pretty(todos).expect("Unable to serialize");
    fs::write(DB_FILE, data).expect("Unable to write file");
}
