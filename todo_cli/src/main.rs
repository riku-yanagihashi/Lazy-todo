use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{self, Read, Write};
use serde_json::Result as SerdeResult;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{stdout, Write as IoWrite};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

#[derive(Serialize, Deserialize, Debug)]
struct TodoItem {
    id: u32,
    task: String,
    done: bool,
}

const FILE_PATH: &str = "todo_list.json";

fn read_todos() -> SerdeResult<Vec<TodoItem>> {
    let mut file = match File::open(FILE_PATH) {
        Ok(file) => file,
        Err(_) => File::create(FILE_PATH).expect("Failed to create file"),
    };
    let mut data = String::new();
    file.read_to_string(&mut data).expect("Failed to read file");
    if data.is_empty() {
        Ok(vec![])
    } else {
        serde_json::from_str(&data)
    }
}

fn write_todos(todos: &Vec<TodoItem>) -> io::Result<()> {
    let data = serde_json::to_string(todos).expect("Failed to serialize todos");
    let mut file = File::create(FILE_PATH)?;
    file.write_all(data.as_bytes())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

enum Mode {
    Normal,
    Adding(String),
    Editing(u32, String),
    ConfirmDelete(u32),
    Help,
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let mut mode = Mode::Normal;
    let mut cursor_pos = 0;
    let mut tasks = read_todos().unwrap();
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(50),
                        Constraint::Percentage(50),
                    ].as_ref()
                )
                .split(f.size());

            let block = Block::default()
                .title("Todo List")
                .borders(Borders::ALL);
            f.render_widget(block, chunks[0]);

            let items: Vec<ListItem> = tasks.iter().map(|todo| {
                let lines = vec![
                    Spans::from(Span::styled(todo.task.clone(), Style::default().add_modifier(Modifier::BOLD))),
                ];
                ListItem::new(lines).style(Style::default().fg(if todo.done { Color::Green } else { Color::Red }))
            }).collect();

            let mut state = tui::widgets::ListState::default();
            state.select(Some(cursor_pos));

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Todos"))
                .highlight_style(Style::default().bg(Color::LightGreen).fg(Color::Black))
                .highlight_symbol(">> ");
            f.render_stateful_widget(list, chunks[0], &mut state);

            match &mode {
                Mode::Adding(input) => {
                    let block = Block::default()
                        .title("Add Task")
                        .borders(Borders::ALL);
                    let paragraph = Paragraph::new(format!("title: {}", input)).block(block);
                    f.render_widget(paragraph, chunks[1]);
                }
                Mode::Editing(_, input) => {
                    let block = Block::default()
                        .title("Edit Task")
                        .borders(Borders::ALL);
                    let paragraph = Paragraph::new(format!("title: {}", input)).block(block);
                    f.render_widget(paragraph, chunks[1]);
                }
                Mode::ConfirmDelete(id) => {
                    let block = Block::default()
                        .title("Delete Task")
                        .borders(Borders::ALL);
                    let paragraph = Paragraph::new(format!("Are you sure you want to delete task with ID {}? (y/n): ", id)).block(block);
                    f.render_widget(paragraph, chunks[1]);
                }
                Mode::Help => {
                    let block = Block::default()
                        .title("Help")
                        .borders(Borders::ALL);
                    let text = vec![
                        Spans::from("a - Add a new task"),
                        Spans::from("e - Edit a task"),
                        Spans::from("d - Delete a task"),
                        Spans::from("q - Quit"),
                        Spans::from("k - Move cursor up"),
                        Spans::from("j - Move cursor down"),
                        Spans::from("l - Select task"),
                        Spans::from("h - Close task"),
                    ];
                    let paragraph = Paragraph::new(text).block(block).wrap(tui::widgets::Wrap { trim: true });
                    f.render_widget(paragraph, chunks[1]);
                }
                _ => {}
            }
        })?;

        if let event::Event::Key(KeyEvent { code, modifiers }) = event::read()? {
            match code {
                KeyCode::Char('q') if matches!(mode, Mode::Normal) => return Ok(()),
                KeyCode::Char('a') if matches!(mode, Mode::Normal) => mode = Mode::Adding(String::new()),
                KeyCode::Char('e') if matches!(mode, Mode::Normal) => {
                    if let Some(todo) = tasks.get(cursor_pos) {
                        mode = Mode::Editing(todo.id, todo.task.clone());
                    }
                }
                KeyCode::Char('d') if matches!(mode, Mode::Normal) => {
                    if let Some(todo) = tasks.get(cursor_pos) {
                        mode = Mode::ConfirmDelete(todo.id);
                    }
                }
                KeyCode::Char('?') if matches!(mode, Mode::Normal) => mode = Mode::Help,
                KeyCode::Char('k') if matches!(mode, Mode::Normal) => {
                    if cursor_pos > 0 {
                        cursor_pos -= 1;
                    }
                }
                KeyCode::Char('j') if matches!(mode, Mode::Normal) => {
                    if cursor_pos < tasks.len() - 1 {
                        cursor_pos += 1;
                    }
                }
                KeyCode::Char('l') if matches!(mode, Mode::ConfirmDelete(_)) => {
                    if let Mode::ConfirmDelete(id) = mode {
                        delete_todo_by_id(id);
                        tasks = read_todos().unwrap();
                        mode = Mode::Normal;
                    }
                }
                KeyCode::Char('h') if !matches!(mode, Mode::Normal) => {
                    mode = Mode::Normal;
                }
                KeyCode::Char('y') if matches!(mode, Mode::ConfirmDelete(_)) => {
                    if let Mode::ConfirmDelete(id) = mode {
                        delete_todo_by_id(id);
                        tasks = read_todos().unwrap();
                        mode = Mode::Normal;
                    }
                }
                KeyCode::Char('n') if matches!(mode, Mode::ConfirmDelete(_)) => {
                    if let Mode::ConfirmDelete(_) = mode {
                        mode = Mode::Normal;
                    }
                }
                KeyCode::Enter => {
                    match &mode {
                        Mode::Adding(input) => {
                            create_todo(input.clone());
                            tasks = read_todos().unwrap();
                            mode = Mode::Normal;
                        }
                        Mode::Editing(id, input) => {
                            update_todo(*id, Some(input.clone()), None);
                            tasks = read_todos().unwrap();
                            mode = Mode::Normal;
                        }
                        _ => {}
                    }
                }
                KeyCode::Char(c) if matches!(mode, Mode::Adding(_) | Mode::Editing(_, _)) => {
                    match &mut mode {
                        Mode::Adding(input) | Mode::Editing(_, input) => {
                            input.push(c);
                        }
                        _ => {}
                    }
                }
                KeyCode::Backspace if matches!(mode, Mode::Adding(_) | Mode::Editing(_, _)) => {
                    match &mut mode {
                        Mode::Adding(input) | Mode::Editing(_, input) => {
                            input.pop();
                        }
                        _ => {}
                    }
                }
                KeyCode::Esc if matches!(mode, Mode::Adding(_) | Mode::Editing(_, _) | Mode::ConfirmDelete(_)) => {
                    mode = Mode::Normal;
                }
                _ => {}
            }
        }
    }
}

fn delete_todo_by_id(id: u32) {
    let mut todos = read_todos().expect("Failed to read todos");
    todos.retain(|todo| todo.id != id);
    write_todos(&todos).expect("Failed to write todos");
}

fn create_todo(task: String) {
    let mut todos = read_todos().expect("Failed to read todos");
    let id = if let Some(last) = todos.last() { last.id + 1 } else { 1 };
    let todo = TodoItem { id, task, done: false };
    todos.push(todo);
    write_todos(&todos).expect("Failed to write todos");
}

fn update_todo(id: u32, task: Option<String>, done: Option<bool>) {
    let mut todos = read_todos().expect("Failed to read todos");
    if let Some(todo) = todos.iter_mut().find(|todo| todo.id == id) {
        if let Some(task) = task {
            todo.task = task;
        }
        if let Some(done) = done {
            todo.done = done;
        }
    }
    write_todos(&todos).expect("Failed to write todos");
}
