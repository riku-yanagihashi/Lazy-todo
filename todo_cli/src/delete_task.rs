use crate::todo::Todo;
use tui::widgets::ListState;

pub fn delete_task(filtered_todos: &mut Vec<Todo>, todos: &mut Vec<Todo>, state: &mut ListState) {
    if let Some(selected) = state.selected() {
        if !filtered_todos.is_empty() {
            filtered_todos.remove(selected);
            *todos = filtered_todos.clone();
            if selected > 0 {
                state.select(Some(selected - 1));
            }
        }
    }
}

