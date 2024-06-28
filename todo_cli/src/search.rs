use crate::todo::Todo;
use strsim::levenshtein;

pub fn search_todos(todos: &Vec<Todo>, query: &str) -> Vec<Todo> {
    let threshold = 3; // 許容される編集距離の閾値

    todos
        .iter()
        .filter(|todo| {
            levenshtein(&todo.title, query) <= threshold || levenshtein(&todo.content, query) <= threshold
        })
        .cloned()
        .collect()
}
