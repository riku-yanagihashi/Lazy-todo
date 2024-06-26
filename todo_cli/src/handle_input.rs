use crate::add_task::add_task;
use crate::delete_task::delete_task;
use crate::input::{InputMode, PrioritySelection};
use crate::search::search_todos;
use crate::sort::{cycle_sort_mode, sort_todos, SortMode};
use crate::todo::{save_todos, Todo};
use crossterm::cursor::{Hide, Show};
use crossterm::event::KeyCode;
use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use std::io;
use std::time::{Duration, Instant};
use tui::backend::CrosstermBackend;
use tui::Terminal;
use tui::widgets::ListState;
use std::collections::VecDeque;

static mut LAST_SPACE_PRESS: Option<Instant> = None;
static mut DELETED_TODOS: VecDeque<Todo> = VecDeque::new();

pub fn handle_input(
    key: crossterm::event::KeyEvent,
    todos: &mut Vec<Todo>,
    filtered_todos: &mut Vec<Todo>,
    state: &mut ListState,
    input_mode: &mut InputMode,
    input_title: &mut String,
    input_content: &mut String,
    input_priority: &mut PrioritySelection,
    input_deadline: &mut String,
    search_query: &mut String,
    search_state: &mut ListState,
    sort_mode: &mut SortMode,
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    subtask_state: &mut ListState,
) -> Result<(), io::Error> {
    match input_mode {
        InputMode::Normal => {
            if key.code == KeyCode::Char(' ') {
                let now = Instant::now();
                unsafe {
                    if let Some(last_press) = LAST_SPACE_PRESS {
                        if now.duration_since(last_press) < Duration::from_millis(500) {
                            *input_mode = InputMode::Searching;
                            terminal.backend_mut().execute(Hide)?;
                            LAST_SPACE_PRESS = None;
                        } else {
                            LAST_SPACE_PRESS = Some(now);
                        }
                    } else {
                        LAST_SPACE_PRESS = Some(now);
                    }
                }
            } else {
                unsafe {
                    LAST_SPACE_PRESS = None;
                }
                match key.code {
                    KeyCode::Char('q') => {
                        disable_raw_mode()?;
                        terminal.backend_mut().execute(LeaveAlternateScreen)?;
                        terminal.show_cursor()?;
                        std::process::exit(0);
                    }
                    KeyCode::Char('a') => {
                        *input_mode = InputMode::AddingTitle;
                    }
                    KeyCode::Char('d') => {
                        unsafe {
                            delete_task(filtered_todos, todos, state, &mut DELETED_TODOS);
                        }
                        save_todos(todos);
                    }
                    KeyCode::Char('u') => {
                        unsafe {
                            if let Some(todo) = DELETED_TODOS.pop_back() {
                                filtered_todos.push(todo.clone());
                                *todos = filtered_todos.clone();
                                save_todos(todos);
                            }
                        }
                    }
                    KeyCode::Char('e') => {
                        if let Some(selected) = state.selected() {
                            let (main_index, sub_index) = get_main_and_sub_index(filtered_todos, selected);
                            if let Some(main_index) = main_index {
                                if let Some(sub_index) = sub_index {
                                    let subtask = &filtered_todos[main_index].subtasks[sub_index];
                                    *input_mode = InputMode::EditingTitle(selected);
                                    *input_title = subtask.title.clone();
                                    *input_content = subtask.content.clone();
                                    *input_priority = match subtask.priority.as_str() {
                                        "low" => PrioritySelection::Low,
                                        "medium" => PrioritySelection::Medium,
                                        "high" => PrioritySelection::High,
                                        _ => PrioritySelection::Low,
                                    };
                                    *input_deadline = subtask.deadline.clone();
                                } else {
                                    let todo = &filtered_todos[main_index];
                                    *input_mode = InputMode::EditingTitle(selected);
                                    *input_title = todo.title.clone();
                                    *input_content = todo.content.clone();
                                    *input_priority = match todo.priority.as_str() {
                                        "low" => PrioritySelection::Low,
                                        "medium" => PrioritySelection::Medium,
                                        "high" => PrioritySelection::High,
                                        _ => PrioritySelection::Low,
                                    };
                                    *input_deadline = todo.deadline.clone();
                                }
                            }
                        }
                    }
                    KeyCode::Char('l') => {
                        if let Some(selected) = state.selected() {
                            let (main_index, sub_index) = get_main_and_sub_index(filtered_todos, selected);
                            if let Some(main_index) = main_index {
                                if sub_index.is_none() {
                                    filtered_todos[main_index].expanded = true;
                                }
                            }
                        }
                    }
                    KeyCode::Char('h') => {
                        if let Some(selected) = state.selected() {
                            let (main_index, sub_index) = get_main_and_sub_index(filtered_todos, selected);
                            if let Some(main_index) = main_index {
                                if sub_index.is_none() {
                                    filtered_todos[main_index].expanded = false;
                                }
                            }
                        }
                    }
                    KeyCode::Char('o') => {
                        if let Some(selected) = state.selected() {
                            let (main_index, sub_index) = get_main_and_sub_index(filtered_todos, selected);
                            if let Some(main_index) = main_index {
                                if sub_index.is_none() {
                                    *input_mode = InputMode::ViewingDetails(main_index);
                                    subtask_state.select(Some(0));
                                }
                            }
                        }
                    }
                    KeyCode::Char('j') => {
                        let mut index = state.selected().unwrap_or(0);
                        index = move_cursor_down(filtered_todos, index);
                        state.select(Some(index));
                    }
                    KeyCode::Char('k') => {
                        let mut index = state.selected().unwrap_or(0);
                        index = move_cursor_up(filtered_todos, index);
                        state.select(Some(index));
                    }
                    KeyCode::Char('s') => {
                        cycle_sort_mode(sort_mode);
                        sort_todos(filtered_todos, sort_mode.clone());
                    }
                    KeyCode::Enter => {
                        if let Some(selected) = state.selected() {
                            let (main_index, sub_index) = get_main_and_sub_index(filtered_todos, selected);
                            if let Some(main_index) = main_index {
                                if let Some(sub_index) = sub_index {
                                    filtered_todos[main_index].subtasks[sub_index].done = !filtered_todos[main_index].subtasks[sub_index].done;
                                } else {
                                    filtered_todos[main_index].done = !filtered_todos[main_index].done;
                                }
                                save_todos(todos);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        InputMode::AddingTitle => match key.code {
            KeyCode::Enter => {
                if !input_title.is_empty() {
                    *input_mode = InputMode::AddingContent;
                }
            }
            KeyCode::Char(c) => {
                input_title.push(c);
            }
            KeyCode::Backspace => {
                input_title.pop();
            }
            KeyCode::Esc => {
                *input_mode = InputMode::Normal;
                input_title.clear();
            }
            _ => {}
        },
        InputMode::AddingContent => match key.code {
            KeyCode::Enter => {
                *input_mode = InputMode::AddingPriority;
            }
            KeyCode::Char(c) => {
                input_content.push(c);
            }
            KeyCode::Backspace => {
                input_content.pop();
            }
            KeyCode::Esc => {
                *input_mode = InputMode::Normal;
                input_title.clear();
                input_content.clear();
            }
            _ => {}
        },
        InputMode::AddingPriority => match key.code {
            KeyCode::Enter => {
                *input_mode = InputMode::AddingDeadline;
            }
            KeyCode::Char('j') => {
                *input_priority = input_priority.next();
            }
            KeyCode::Char('k') => {
                *input_priority = input_priority.prev();
            }
            KeyCode::Esc => {
                *input_mode = InputMode::Normal;
                input_title.clear();
                input_content.clear();
                input_deadline.clear();
            }
            _ => {}
        },
        InputMode::AddingDeadline => match key.code {
            KeyCode::Enter => {
                if !input_title.is_empty() {
                    add_task(
                        todos,
                        filtered_todos,
                        input_title.clone(),
                        input_content.clone(),
                        input_priority.to_str().to_string(),
                        input_deadline.clone(),
                    );
                    save_todos(todos);
                    *input_mode = InputMode::Normal;
                    input_title.clear();
                    input_content.clear();
                    input_deadline.clear();
                }
            }
            KeyCode::Char(c) => {
                input_deadline.push(c);
            }
            KeyCode::Backspace => {
                input_deadline.pop();
            }
            KeyCode::Esc => {
                *input_mode = InputMode::Normal;
                input_title.clear();
                input_content.clear();
                input_deadline.clear();
            }
            _ => {}
        },
        InputMode::EditingTitle(index) => match key.code {
            KeyCode::Enter => {
                if !input_title.is_empty() {
                    filtered_todos[*index].title = input_title.clone();
                    *input_mode = InputMode::EditingContent(*index);
                }
            }
            KeyCode::Char(c) => {
                input_title.push(c);
            }
            KeyCode::Backspace => {
                input_title.pop();
            }
            KeyCode::Esc => {
                *input_mode = InputMode::Normal;
                input_title.clear();
                input_content.clear();
                input_deadline.clear();
            }
            _ => {}
        },
        InputMode::EditingContent(index) => match key.code {
            KeyCode::Enter => {
                filtered_todos[*index].content = input_content.clone();
                *input_mode = InputMode::EditingPriority(*index);
            }
            KeyCode::Char(c) => {
                input_content.push(c);
            }
            KeyCode::Backspace => {
                input_content.pop();
            }
            KeyCode::Esc => {
                *input_mode = InputMode::Normal;
                input_title.clear();
                input_content.clear();
                input_deadline.clear();
            }
            _ => {}
        },
        InputMode::EditingPriority(index) => match key.code {
            KeyCode::Enter => {
                *input_mode = InputMode::EditingDeadline(*index);
            }
            KeyCode::Char('j') => {
                *input_priority = input_priority.next();
            }
            KeyCode::Char('k') => {
                *input_priority = input_priority.prev();
            }
            KeyCode::Esc => {
                *input_mode = InputMode::Normal;
                input_title.clear();
                input_content.clear();
                input_deadline.clear();
            }
            _ => {}
        },
        InputMode::EditingDeadline(index) => match key.code {
            KeyCode::Enter => {
                filtered_todos[*index].priority = input_priority.to_str().to_string();
                filtered_todos[*index].deadline = input_deadline.clone();
                filtered_todos[*index].date_time = chrono::Utc::now().to_rfc3339();
                *todos = filtered_todos.clone();
                save_todos(todos);
                *input_mode = InputMode::Normal;
                input_title.clear();
                input_content.clear();
                input_deadline.clear();
            }
            KeyCode::Char(c) => {
                input_deadline.push(c);
            }
            KeyCode::Backspace => {
                input_deadline.pop();
            }
            KeyCode::Esc => {
                *input_mode = InputMode::Normal;
                input_title.clear();
                input_content.clear();
                input_deadline.clear();
            }
            _ => {}
        },
        InputMode::Searching => match key.code {
            KeyCode::Char(c) => {
                search_query.push(c);
                search_state.select(Some(0));
            }
            KeyCode::Backspace => {
                search_query.pop();
                search_state.select(Some(0));
            }
            KeyCode::Enter => {
                *filtered_todos = search_todos(todos, search_query);
                *input_mode = InputMode::Normal;
                state.select(Some(0));
                terminal.backend_mut().execute(Show)?;
            }
            KeyCode::Esc => {
                *input_mode = InputMode::Normal;
                search_query.clear();
                terminal.backend_mut().execute(Show)?;
            }
            KeyCode::Char('j') => {
                if let Some(selected) = search_state.selected() {
                    if selected < todos.len() - 1 {
                        search_state.select(Some(selected + 1));
                    }
                }
            }
            KeyCode::Char('k') => {
                if let Some(selected) = search_state.selected() {
                    if selected > 0 {
                        search_state.select(Some(selected - 1));
                    }
                }
            }
            _ => {}
        },
        InputMode::ViewingDetails(index) => {
            if key.code == KeyCode::Char('q') {
                *input_mode = InputMode::Normal;
            } else if key.code == KeyCode::Char(' ') {
                *input_mode = InputMode::AddingSubtask(*index);
            } else if key.code == KeyCode::Char('j') {
                if let Some(selected) = subtask_state.selected() {
                    let new_index = selected.saturating_add(1);
                    if new_index < filtered_todos[*index].subtasks.len() {
                        subtask_state.select(Some(new_index));
                    }
                } else {
                    subtask_state.select(Some(0));
                }
            } else if key.code == KeyCode::Char('k') {
                if let Some(selected) = subtask_state.selected() {
                    let new_index = selected.saturating_sub(1);
                    subtask_state.select(Some(new_index));
                }
            } else if key.code == KeyCode::Enter {
                if let Some(selected) = subtask_state.selected() {
                    let subtask = &mut filtered_todos[*index].subtasks[selected];
                    subtask.done = !subtask.done;
                    save_todos(filtered_todos);
                }
            }
        }
        InputMode::ViewingSubtaskDetails(task_index, subtask_index) => {
            if key.code == KeyCode::Char('q') {
                *input_mode = InputMode::ViewingDetails(*task_index);
            } else if key.code == KeyCode::Enter {
                let subtask = &mut filtered_todos[*task_index].subtasks[*subtask_index];
                subtask.done = !subtask.done;
                save_todos(filtered_todos);
            }
        }
        InputMode::AddingSubtask(index) => match key.code {
            KeyCode::Enter => {
                if !input_title.is_empty() {
                    filtered_todos[*index].add_subtask(input_title.clone());
                    save_todos(filtered_todos);
                    *input_mode = InputMode::ViewingDetails(*index);
                    input_title.clear();
                }
            }
            KeyCode::Char(c) => {
                input_title.push(c);
            }
            KeyCode::Backspace => {
                input_title.pop();
            }
            KeyCode::Esc => {
                *input_mode = InputMode::ViewingDetails(*index);
                input_title.clear();
            }
            _ => {}
        },
    }
    Ok(())
}

fn move_cursor_down(todos: &Vec<Todo>, index: usize) -> usize {
    let mut current_index = 0;
    let mut target_index = index;
    for (main_index, todo) in todos.iter().enumerate() {
        if current_index == index {
            target_index = current_index + 1;
            break;
        }
        current_index += 1;
        if todo.expanded {
            for (sub_index, _subtask) in todo.subtasks.iter().enumerate() {
                if current_index == index {
                    target_index = current_index + 1;
                    break;
                }
                current_index += 1;
            }
        }
    }
    target_index.min(total_items(todos) - 1)
}

fn move_cursor_up(todos: &Vec<Todo>, index: usize) -> usize {
    let mut current_index = 0;
    let mut target_index = index;
    for (main_index, todo) in todos.iter().enumerate() {
        if current_index == index {
            if current_index > 0 {
                target_index = current_index - 1;
            }
            break;
        }
        current_index += 1;
        if todo.expanded {
            for (sub_index, _subtask) in todo.subtasks.iter().enumerate() {
                if current_index == index {
                    if current_index > 0 {
                        target_index = current_index - 1;
                    }
                    break;
                }
                current_index += 1;
            }
        }
    }
    target_index
}

fn toggle_done(todos: &mut Vec<Todo>, index: usize) {
    let mut current_index = 0;
    for todo in todos {
        if current_index == index {
            todo.done = !todo.done;
            break;
        }
        current_index += 1;
        if todo.expanded {
            for subtask in &mut todo.subtasks {
                if current_index == index {
                    subtask.done = !subtask.done;
                    break;
                }
                current_index += 1;
            }
        }
    }
}

fn total_items(todos: &Vec<Todo>) -> usize {
    todos.iter().map(|todo| {
        if todo.expanded {
            1 + todo.subtasks.len()
        } else {
            1
        }
    }).sum()
}

fn get_main_and_sub_index(todos: &Vec<Todo>, index: usize) -> (Option<usize>, Option<usize>) {
    let mut current_index = 0;
    for (main_index, todo) in todos.iter().enumerate() {
        if current_index == index {
            return (Some(main_index), None);
        }
        current_index += 1;
        if todo.expanded {
            for (sub_index, _subtask) in todo.subtasks.iter().enumerate() {
                if current_index == index {
                    return (Some(main_index), Some(sub_index));
                }
                current_index += 1;
            }
        }
    }
    (None, None)
}
