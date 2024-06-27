use crate::todo::Todo;

pub fn add_task(todos: &mut Vec<Todo>, filtered_todos: &mut Vec<Todo>, title: String, content: String, priority: String, deadline: String) {
    let new_todo = Todo::new(title, content, priority, deadline);
    filtered_todos.push(new_todo);
    *todos = filtered_todos.clone();
}

