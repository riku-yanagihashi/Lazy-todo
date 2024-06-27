use crate::todo::Todo;
use std::cmp::Ordering;

#[derive(Clone)]
pub enum SortMode {
    ByCompletion,
    ByDeadline,
    ByPriority,
}

pub fn cycle_sort_mode(sort_mode: &mut SortMode) {
    match sort_mode {
        SortMode::ByCompletion => *sort_mode = SortMode::ByDeadline,
        SortMode::ByDeadline => *sort_mode = SortMode::ByPriority,
        SortMode::ByPriority => *sort_mode = SortMode::ByCompletion,
    }
}

pub fn sort_todos(todos: &mut Vec<Todo>, mode: SortMode) {
    match mode {
        SortMode::ByCompletion => {
            todos.sort_by(|a, b| a.done.cmp(&b.done));
        }
        SortMode::ByDeadline => {
            todos.sort_by(|a, b| {
                if a.deadline.is_empty() {
                    Ordering::Greater
                } else if b.deadline.is_empty() {
                    Ordering::Less
                } else {
                    a.deadline.cmp(&b.deadline)
                }
            });
        }
        SortMode::ByPriority => {
            todos.sort_by(|a, b| {
                let priority_order = |p: &str| match p {
                    "high" => 0,
                    "medium" => 1,
                    "low" => 2,
                    _ => 3,
                };
                priority_order(&a.priority).cmp(&priority_order(&b.priority))
            });
        }
    }
}

