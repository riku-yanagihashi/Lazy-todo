use crate::todo::Todo;
use tui::widgets::ListState;
use std::collections::VecDeque;

pub fn delete_task(filtered_todos: &mut Vec<Todo>, todos: &mut Vec<Todo>, state: &mut ListState, deleted_todos: &mut VecDeque<Todo>) {
    if let Some(selected) = state.selected() {
        if !filtered_todos.is_empty() {
            let deleted_todo = filtered_todos.remove(selected);
            deleted_todos.push_back(deleted_todo);
            *todos = filtered_todos.clone();
            if selected > 0 {
                state.select(Some(selected - 1));
            }
        }
    }
}
