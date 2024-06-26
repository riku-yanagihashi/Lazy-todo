use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem, Paragraph};
use tui::Terminal;
use tui::widgets::ListState;
use chrono::prelude::*;
use std::cmp::Ordering;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
    ViewingDetails,
    Searching,
    EditingTitle(usize),
    EditingContent(usize),
    EditingPriority(usize),
    EditingDeadline(usize),
}

#[derive(PartialEq, Copy, Clone)]
enum PrioritySelection {
    Low,
    Medium,
    High,
}

impl PrioritySelection {
    fn next(&self) -> Self {
        match *self {
            PrioritySelection::Low => PrioritySelection::Medium,
            PrioritySelection::Medium => PrioritySelection::High,
            PrioritySelection::High => PrioritySelection::Low,
        }
    }

    fn prev(&self) -> Self {
        match *self {
            PrioritySelection::Low => PrioritySelection::High,
            PrioritySelection::Medium => PrioritySelection::Low,
            PrioritySelection::High => PrioritySelection::Medium,
        }
    }

    fn to_str(&self) -> &'static str {
        match *self {
            PrioritySelection::Low => "low",
            PrioritySelection::Medium => "medium",
            PrioritySelection::High => "high",
        }
    }

    fn color(&self) -> Color {
        match *self {
            PrioritySelection::Low => Color::Green,
            PrioritySelection::Medium => Color::Yellow,
            PrioritySelection::High => Color::Red,
        }
    }
}

#[derive(Clone)]
enum SortMode {
    ByCompletion,
    ByDeadline,
    ByPriority,
}

fn sort_todos(todos: &mut Vec<Todo>, mode: SortMode) {
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

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut todos = load_todos();
    let mut filtered_todos = todos.clone();
    let mut state = ListState::default();
    state.select(Some(0));

    let mut input_mode = InputMode::Normal;
    let mut input_title = String::new();
    let mut input_content = String::new();
    let mut input_priority = PrioritySelection::Low;
    let mut input_deadline = String::new();
    let mut search_query = String::new();
    let mut search_state = ListState::default();
    search_state.select(Some(0));
    let mut sort_mode = SortMode::ByCompletion;

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let (left_chunk, right_chunk) = if input_mode == InputMode::ViewingDetails {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(size);
                (chunks[0], chunks[1])
            } else {
                (size, size)
            };

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(1),
                        Constraint::Length(6),
                    ]
                    .as_ref(),
                )
                .split(left_chunk);

            // 現在のソートモードを取得
            let sort_mode_str = match sort_mode {
                SortMode::ByCompletion => "Completion",
                SortMode::ByDeadline => "Deadline",
                SortMode::ByPriority => "Priority",
            };

            let block = Block::default()
                .borders(Borders::ALL)
                .title(format!("Lazy Todo - Sort Mode: {}", sort_mode_str)); // タイトルにソートモードを追加
            f.render_widget(block, size);

            let items: Vec<ListItem> = filtered_todos
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

            if input_mode == InputMode::ViewingDetails {
                if let Some(selected) = state.selected() {
                    let todo = &filtered_todos[selected];
                    let status = if todo.done { "✔" } else { "✘" };
                    let priority = &todo.priority;
                    let deadline = if todo.deadline.is_empty() { "No Deadline" } else { &todo.deadline };
                    let details = vec![
                        Spans::from(vec![Span::styled("Title: ", Style::default().fg(Color::White)), Span::raw(&todo.title)]),
                        Spans::from(vec![Span::styled("Content: ", Style::default().fg(Color::White)), Span::raw(&todo.content)]),
                        Spans::from(vec![Span::styled("Priority: ", Style::default().fg(Color::White)), Span::raw(priority)]),
                        Spans::from(vec![Span::styled("Deadline: ", Style::default().fg(Color::White)), Span::raw(deadline)]),
                        Spans::from(vec![Span::styled("Status: ", Style::default().fg(Color::White)), Span::raw(status)]),
                    ];
                    let details_block = Paragraph::new(details)
                        .block(Block::default().borders(Borders::ALL).title("Details"))
                        .wrap(tui::widgets::Wrap { trim: true });
                    f.render_widget(details_block, right_chunk);
                }
            }

            let instructions_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
                .split(chunks[2]);

            let instructions = match input_mode {
                InputMode::Normal => {
                    String::from("q: Quit | a: Add | d: Delete | e: Edit | l: View Details | <space><space>: Search | s: Sort | j: Down | k: Up | Enter: Toggle Done")
                }
                InputMode::AddingTitle | InputMode::EditingTitle(_) => format!("Enter title: {}", input_title),
                InputMode::AddingContent | InputMode::EditingContent(_) => format!("Enter content: {}", input_content),
                InputMode::AddingPriority | InputMode::EditingPriority(_) => {
                    let priority_symbol = match input_priority {
                        PrioritySelection::Low => "●",
                        PrioritySelection::Medium => "●",
                        PrioritySelection::High => "●",
                    };
                    let priority_color = input_priority.color();
                    let priorities_list = Paragraph::new(Spans::from(vec![Span::styled(priority_symbol, Style::default().fg(priority_color))]))
                        .block(Block::default().borders(Borders::ALL).title("Select priority"));

                    f.render_widget(priorities_list, instructions_chunks[1]);
                    format!("Use j/k to change priority")
                },
                InputMode::AddingDeadline | InputMode::EditingDeadline(_) => format!("Enter deadline: {}", input_deadline),
                InputMode::ViewingDetails => String::from("Press q to go back"),
                InputMode::Searching => String::from("Type to search | Enter to filter | Esc to cancel"),
            };
            let instructions_paragraph = Paragraph::new(instructions)
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .block(Block::default().borders(Borders::ALL).title("Instructions"));
            f.render_widget(instructions_paragraph, instructions_chunks[0]);

            if input_mode == InputMode::Searching {
                let search_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Length(8)].as_ref())
                    .split(Rect {
                        x: size.width / 2 - 60, 
                        y: size.height / 2 - 20,
                        width: 120,
                        height: 25,
                    });

                let search_input = Paragraph::new(search_query.as_ref())
                    .style(Style::default().fg(Color::Yellow))
                    .block(Block::default().borders(Borders::ALL).title("Search"));

                let search_items: Vec<ListItem> = todos
                    .iter()
                    .filter(|todo| {
                        todo.title.contains(&search_query) || todo.content.contains(&search_query)
                    })
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
                let search_list = List::new(search_items)
                    .block(Block::default().borders(Borders::ALL).title("Search Results"))
                    .highlight_style(Style::default().bg(Color::Blue));
                f.render_stateful_widget(search_list, search_chunks[1], &mut search_state);

                f.render_widget(search_input, search_chunks[0]);
            }
        })?;

        if let Event::Key(key) = event::read()? {
            match input_mode {
                InputMode::Normal => {
                    static mut SPACE_COUNT: u8 = 0;
                    if key.code == KeyCode::Char(' ') {
                        unsafe {
                            SPACE_COUNT += 1;
                            if SPACE_COUNT >= 2 {
                                input_mode = InputMode::Searching;
                                terminal.backend_mut().execute(Hide)?; // ここを追加
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
                                break;
                            }
                            KeyCode::Char('a') => {
                                input_mode = InputMode::AddingTitle;
                            }
                            KeyCode::Char('d') => {
                                if let Some(_selected) = state.selected() {
                                    if !filtered_todos.is_empty() {
                                        filtered_todos.remove(_selected);
                                        todos = filtered_todos.clone();
                                        if _selected > 0 {
                                            state.select(Some(_selected - 1));
                                        }
                                        save_todos(&todos);
                                    }
                                }
                            }
                            KeyCode::Char('e') => {
                                if let Some(_selected) = state.selected() {
                                    if !filtered_todos.is_empty() {
                                        input_mode = InputMode::EditingTitle(_selected);
                                        input_title = filtered_todos[_selected].title.clone();
                                        input_content = filtered_todos[_selected].content.clone();
                                        input_priority = match filtered_todos[_selected].priority.as_str() {
                                            "low" => PrioritySelection::Low,
                                            "medium" => PrioritySelection::Medium,
                                            "high" => PrioritySelection::High,
                                            _ => PrioritySelection::Low,
                                        };
                                        input_deadline = filtered_todos[_selected].deadline.clone();
                                    }
                                }
                            }
                            KeyCode::Char('l') => {
                                if let Some(_selected) = state.selected() {
                                    if !filtered_todos.is_empty() {
                                        input_mode = InputMode::ViewingDetails;
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
                                match sort_mode {
                                    SortMode::ByCompletion => sort_mode = SortMode::ByDeadline,
                                    SortMode::ByDeadline => sort_mode = SortMode::ByPriority,
                                    SortMode::ByPriority => sort_mode = SortMode::ByCompletion,
                                }
                                sort_todos(&mut filtered_todos, sort_mode.clone());
                            }
                            KeyCode::Enter => {
                                if let Some(selected) = state.selected() {
                                    if !filtered_todos.is_empty() {
                                        filtered_todos[selected].done = !filtered_todos[selected].done;
                                        todos = filtered_todos.clone();
                                        save_todos(&todos);
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
                    KeyCode::Char('j') => {
                        input_priority = input_priority.next();
                    }
                    KeyCode::Char('k') => {
                        input_priority = input_priority.prev();
                    }
                    KeyCode::Esc => {
                        input_mode = InputMode::Normal;
                        input_title.clear();
                        input_content.clear();
                        input_deadline.clear();
                    }
                    _ => {}
                },
                InputMode::AddingDeadline => match key.code {
                    KeyCode::Enter => {
                        if !input_title.is_empty() {
                            let priority_str = input_priority.to_str().to_string();
                            let new_todo = Todo::new(input_title.clone(), input_content.clone(), priority_str, input_deadline.clone());
                            filtered_todos.push(new_todo);
                            todos = filtered_todos.clone();
                            save_todos(&todos);
                            input_mode = InputMode::Normal;
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
                        input_mode = InputMode::Normal;
                        input_title.clear();
                        input_content.clear();
                        input_deadline.clear();
                    }
                    _ => {}
                },
                InputMode::EditingTitle(index) => match key.code {
                    KeyCode::Enter => {
                        if !input_title.is_empty() {
                            filtered_todos[index].title = input_title.clone();
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
                        input_deadline.clear();
                    }
                    _ => {}
                },
                InputMode::EditingContent(index) => match key.code {
                    KeyCode::Enter => {
                        filtered_todos[index].content = input_content.clone();
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
                        input_deadline.clear();
                    }
                    _ => {}
                },
                InputMode::EditingPriority(index) => match key.code {
                    KeyCode::Enter => {
                        input_mode = InputMode::EditingDeadline(index);
                    }
                    KeyCode::Char('j') => {
                        input_priority = input_priority.next();
                    }
                    KeyCode::Char('k') => {
                        input_priority = input_priority.prev();
                    }
                    KeyCode::Esc => {
                        input_mode = InputMode::Normal;
                        input_title.clear();
                        input_content.clear();
                        input_deadline.clear();
                    }
                    _ => {}
                },
                InputMode::EditingDeadline(index) => match key.code {
                    KeyCode::Enter => {
                        filtered_todos[index].priority = input_priority.to_str().to_string();
                        filtered_todos[index].deadline = input_deadline.clone();
                        filtered_todos[index].date_time = Utc::now().to_rfc3339();
                        todos = filtered_todos.clone();
                        save_todos(&todos);
                        input_mode = InputMode::Normal;
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
                        input_mode = InputMode::Normal;
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
                        filtered_todos = todos
                            .iter()
                            .filter(|todo| {
                                todo.title.contains(&search_query) || todo.content.contains(&search_query)
                            })
                            .cloned()
                            .collect();
                        input_mode = InputMode::Normal;
                        state.select(Some(0));
                        terminal.backend_mut().execute(Show)?; // ここを追加
                    }
                    KeyCode::Esc => {
                        input_mode = InputMode::Normal;
                        search_query.clear();
                        terminal.backend_mut().execute(Show)?; // ここを追加
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
                        input_mode = InputMode::Normal;
                    }
                }
            }
        }
    }

    Ok(())
}
