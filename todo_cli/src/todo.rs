use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use chrono::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Todo {
    pub title: String,
    pub content: String,
    pub priority: String,
    pub date_time: String,
    pub deadline: String,
    pub done: bool,
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
        }
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

