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
use tui::backend::CrosstermBackend;
use tui::Terminal;
use tui::widgets::ListState;

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
) -> Result<(), io::Error> {
    match input_mode {
        InputMode::Normal => {
            static mut SPACE_COUNT: u8 = 0;
            if key.code == KeyCode::Char(' ') {
                unsafe {
                    SPACE_COUNT += 1;
                    if SPACE_COUNT >= 2 {
                        *input_mode = InputMode::Searching;
                        terminal.backend_mut().execute(Hide)?;
                        SPACE_COUNT = 0;
                    }
                }
            } else {
                unsafe {
                    SPACE_COUNT = 0;
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
                        delete_task(filtered_todos, todos, state);
                        save_todos(todos);
                    }
                    KeyCode::Char('e') => {
                        if let Some(selected) = state.selected() {
                            if !filtered_todos.is_empty() {
                                *input_mode = InputMode::EditingTitle(selected);
                                *input_title = filtered_todos[selected].title.clone();
                                *input_content = filtered_todos[selected].content.clone();
                                *input_priority = match filtered_todos[selected].priority.as_str() {
                                    "low" => PrioritySelection::Low,
                                    "medium" => PrioritySelection::Medium,
                                    "high" => PrioritySelection::High,
                                    _ => PrioritySelection::Low,
                                };
                                *input_deadline = filtered_todos[selected].deadline.clone();
                            }
                        }
                    }
                    KeyCode::Char('l') => {
                        if let Some(selected) = state.selected() {
                            if !filtered_todos.is_empty() {
                                *input_mode = InputMode::ViewingDetails;
                            }
                        }
                    }
                    KeyCode::Char('j') => {
                        if let Some(selected) = state.selected() {
                            if selected < filtered_todos.len() - 1 {
                                state.select(Some(selected + 1));
                            }
                        }
                    }
                    KeyCode::Char('k') => {
                        if let Some(selected) = state.selected() {
                            if selected > 0 {
                                state.select(Some(selected - 1));
                            }
                        }
                    }
                    KeyCode::Char('s') => {
                        cycle_sort_mode(sort_mode);
                        sort_todos(filtered_todos, sort_mode.clone());
                    }
                    KeyCode::Enter => {
                        if let Some(selected) = state.selected() {
                            if !filtered_todos.is_empty() {
                                filtered_todos[selected].done = !filtered_todos[selected].done;
                                *todos = filtered_todos.clone();
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
        InputMode::ViewingDetails => {
            if key.code == KeyCode::Char('q') {
                *input_mode = InputMode::Normal;
            }
        }
    }
    Ok(())
}
