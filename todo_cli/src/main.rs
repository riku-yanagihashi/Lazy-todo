use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem, Paragraph};
use tui::Terminal;
use tui::widgets::ListState;
use chrono::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Todo {
    title: String,
    content: String,
    priority: String,
    date_time: String,
    deadline: String,
    done: bool,
}

impl Todo {
    fn new(title: String, content: String, priority: String, deadline: String) -> Self {
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

fn load_todos() -> Vec<Todo> {
    if Path::new(DB_FILE).exists() {
        let data = fs::read_to_string(DB_FILE).expect("Unable to read file");
        serde_json::from_str(&data).expect("Unable to parse JSON")
    } else {
        vec![]
    }
}

fn save_todos(todos: &Vec<Todo>) {
    let data = serde_json::to_string_pretty(todos).expect("Unable to serialize");
    fs::write(DB_FILE, data).expect("Unable to write file");
}

#[derive(PartialEq)]
enum InputMode {
    Normal,
    AddingTitle,
    AddingContent,
    AddingPriority,
    AddingDeadline,
    EditingTitle(usize),
    EditingContent(usize),
    EditingPriority(usize),
    EditingDeadline(usize),
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut todos = load_todos();
    let mut state = ListState::default();
    state.select(Some(0));

    let mut input_mode = InputMode::Normal;
    let mut input_title = String::new();
    let mut input_content = String::new();
    let mut input_priority = String::new();
    let mut input_deadline = String::new();

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(1),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(size);

            let block = Block::default()
                .borders(Borders::ALL)
                .title("Todo List");
            f.render_widget(block, size);

            let items: Vec<ListItem> = todos
                .iter()
                .map(|todo| {
                    let status = if todo.done {
                        Span::styled("✔", Style::default().fg(Color::Green))
                    } else {
                        Span::styled("✘", Style::default().fg(Color::Red))
                    };
                    let priority = match todo.priority.as_str() {
                        "low" => Span::styled(" ●", Style::default().fg(Color::Green)),
                        "medium" => Span::styled(" ●", Style::default().fg(Color::Yellow)),
                        "high" => Span::styled(" ●", Style::default().fg(Color::Red)),
                        _ => Span::raw(""),
                    };
                    let deadline = if todo.deadline.is_empty() {
                        Span::raw("")
                    } else {
                        Span::raw(format!(" | {}", todo.deadline))
                    };
                    let content = Spans::from(vec![
                        status,
                        Span::raw(": "),
                        Span::raw(&todo.title),
                        priority,
                        deadline,
                    ]);
                    ListItem::new(content).style(Style::default())
                })
                .collect();
            let todos_list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Todos"))
                .highlight_style(Style::default().bg(Color::Blue));
            f.render_stateful_widget(todos_list, chunks[1], &mut state);

            let instructions = match input_mode {
                InputMode::Normal => {
                    String::from("q: Quit | a: Add | d: Delete | e: Edit | j: Down | k: Up | Enter: Toggle Done")
                }
                InputMode::AddingTitle | InputMode::EditingTitle(_) => format!("Enter title: {}", input_title),
                InputMode::AddingContent | InputMode::EditingContent(_) => format!("Enter content: {}", input_content),
                InputMode::AddingPriority | InputMode::EditingPriority(_) => format!("Enter priority: {}", input_priority),
                InputMode::AddingDeadline | InputMode::EditingDeadline(_) => format!("Enter deadline: {}", input_deadline),
            };
            let instructions = Paragraph::new(instructions)
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .block(Block::default().borders(Borders::ALL).title("Instructions"));
            f.render_widget(instructions, chunks[2]);
        })?;

        if let Event::Key(key) = event::read()? {
            match input_mode {
                InputMode::Normal => {
                    match key.code {
                        KeyCode::Char('q') => {
                            disable_raw_mode()?;
                            terminal.backend_mut().execute(LeaveAlternateScreen)?;
                            terminal.show_cursor()?;
                            break;
                        }
                        KeyCode::Char('a') => {
                            input_mode = InputMode::AddingTitle;
                        }
                        KeyCode::Char('d') => {
                            if let Some(selected) = state.selected() {
                                if !todos.is_empty() {
                                    todos.remove(selected);
                                    if selected > 0 {
                                        state.select(Some(selected - 1));
                                    }
                                    save_todos(&todos);
                                }
                            }
                        }
                        KeyCode::Char('e') => {
                            if let Some(selected) = state.selected() {
                                if !todos.is_empty() {
                                    input_mode = InputMode::EditingTitle(selected);
                                    input_title = todos[selected].title.clone();
                                    input_content = todos[selected].content.clone();
                                    input_priority = todos[selected].priority.clone();
                                    input_deadline = todos[selected].deadline.clone();
                                }
                            }
                        }
                        KeyCode::Char('j') => {
                            if let Some(selected) = state.selected() {
                                if selected < todos.len() - 1 {
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
                        KeyCode::Enter => {
                            if let Some(selected) = state.selected() {
                                if !todos.is_empty() {
                                    todos[selected].done = !todos[selected].done;
                                    save_todos(&todos);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                InputMode::AddingTitle => match key.code {
                    KeyCode::Enter => {
                        if !input_title.is_empty() {
                            input_mode = InputMode::AddingContent;
                        }
                    }
                    KeyCode::Char(c) => {
                        input_title.push(c);
                    }
                    KeyCode::Backspace => {
                        input_title.pop();
                    }
                    KeyCode::Esc => {
                        input_mode = InputMode::Normal;
                        input_title.clear();
                    }
                    _ => {}
                },
                InputMode::AddingContent => match key.code {
                    KeyCode::Enter => {
                        input_mode = InputMode::AddingPriority;
                    }
                    KeyCode::Char(c) => {
                        input_content.push(c);
                    }
                    KeyCode::Backspace => {
                        input_content.pop();
                    }
                    KeyCode::Esc => {
                        input_mode = InputMode::Normal;
                        input_title.clear();
                        input_content.clear();
                    }
                    _ => {}
                },
                InputMode::AddingPriority => match key.code {
                    KeyCode::Enter => {
                        input_mode = InputMode::AddingDeadline;
                    }
                    KeyCode::Char(c) => {
                        input_priority.push(c);
                    }
                    KeyCode::Backspace => {
                        input_priority.pop();
                    }
                    KeyCode::Esc => {
                        input_mode = InputMode::Normal;
                        input_title.clear();
                        input_content.clear();
                        input_priority.clear();
                    }
                    _ => {}
                },
                InputMode::AddingDeadline => match key.code {
                    KeyCode::Enter => {
                        if !input_title.is_empty() {
                            let new_todo = Todo::new(input_title.clone(), input_content.clone(), input_priority.clone(), input_deadline.clone());
                            todos.push(new_todo);
                            save_todos(&todos);
                            input_mode = InputMode::Normal;
                            input_title.clear();
                            input_content.clear();
                            input_priority.clear();
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
                        input_mode = InputMode::Normal;
                        input_title.clear();
                        input_content.clear();
                        input_priority.clear();
                        input_deadline.clear();
                    }
                    _ => {}
                },
                InputMode::EditingTitle(index) => match key.code {
                    KeyCode::Enter => {
                        if !input_title.is_empty() {
                            todos[index].title = input_title.clone();
                            input_mode = InputMode::EditingContent(index);
                        }
                    }
                    KeyCode::Char(c) => {
                        input_title.push(c);
                    }
                    KeyCode::Backspace => {
                        input_title.pop();
                    }
                    KeyCode::Esc => {
                        input_mode = InputMode::Normal;
                        input_title.clear();
                        input_content.clear();
                        input_priority.clear();
                        input_deadline.clear();
                    }
                    _ => {}
                },
                InputMode::EditingContent(index) => match key.code {
                    KeyCode::Enter => {
                        todos[index].content = input_content.clone();
                        input_mode = InputMode::EditingPriority(index);
                    }
                    KeyCode::Char(c) => {
                        input_content.push(c);
                    }
                    KeyCode::Backspace => {
                        input_content.pop();
                    }
                    KeyCode::Esc => {
                        input_mode = InputMode::Normal;
                        input_title.clear();
                        input_content.clear();
                        input_priority.clear();
                        input_deadline.clear();
                    }
                    _ => {}
                },
                InputMode::EditingPriority(index) => match key.code {
                    KeyCode::Enter => {
                        todos[index].priority = input_priority.clone();
                        input_mode = InputMode::EditingDeadline(index);
                    }
                    KeyCode::Char(c) => {
                        input_priority.push(c);
                    }
                    KeyCode::Backspace => {
                        input_priority.pop();
                    }
                    KeyCode::Esc => {
                        input_mode = InputMode::Normal;
                        input_title.clear();
                        input_content.clear();
                        input_priority.clear();
                        input_deadline.clear();
                    }
                    _ => {}
                },
                InputMode::EditingDeadline(index) => match key.code {
                    KeyCode::Enter => {
                        todos[index].deadline = input_deadline.clone();
                        todos[index].date_time = Utc::now().to_rfc3339();
                        save_todos(&todos);
                        input_mode = InputMode::Normal;
                        input_title.clear();
                        input_content.clear();
                        input_priority.clear();
                        input_deadline.clear();
                    }
                    KeyCode::Char(c) => {
                        input_deadline.push(c);
                    }
                    KeyCode::Backspace => {
                        input_deadline.pop();
                    }
                    KeyCode::Esc => {
                        input_mode = InputMode::Normal;
                        input_title.clear();
                        input_content.clear();
                        input_priority.clear();
                        input_deadline.clear();
                    }
                    _ => {}
                },
            }
        }
    }

    Ok(())
}
