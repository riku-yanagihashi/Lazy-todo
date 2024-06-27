use crate::todo::Todo;

pub fn search_todos(todos: &Vec<Todo>, query: &str) -> Vec<Todo> {
    todos
        .iter()
        .filter(|todo| todo.title.contains(query) || todo.content.contains(query))
        .cloned()
        .collect()
}

