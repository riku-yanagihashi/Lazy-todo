use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{self, Read, Write};
use serde_json::Result as SerdeResult;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{stdout, Write as IoWrite};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem},
    Terminal,
};

// Todoアイテムを表す構造体
#[derive(Serialize, Deserialize, Debug)]
struct TodoItem {
    id: u32,
    task: String,
    done: bool,
}

// Todoリストのデータファイル
const FILE_PATH: &str = "todo_list.json";

// Todoアイテムをファイルから読み込む関数
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

// Todoアイテムをファイルに書き込む関数
fn write_todos(todos: &Vec<TodoItem>) -> io::Result<()> {
    let data = serde_json::to_string(todos).expect("Failed to serialize todos");
    let mut file = File::create(FILE_PATH)?;
    file.write_all(data.as_bytes())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ターミナルをセットアップ
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // メインアプリケーションの実行
    let res = run_app(&mut terminal);

    // 終了処理
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

fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let size = f.size();

            let block = Block::default()
                .title("Todo List")
                .borders(Borders::ALL);
            f.render_widget(block, size);

            let todos = read_todos().unwrap();
            let items: Vec<ListItem> = todos.iter().map(|todo| {
                let lines = vec![
                    Spans::from(Span::styled(todo.task.clone(), Style::default().add_modifier(Modifier::BOLD))),
                ];
                ListItem::new(lines).style(Style::default().fg(if todo.done { Color::Green } else { Color::Red }))
            }).collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Todos"))
                .highlight_style(Style::default().bg(Color::LightGreen).fg(Color::Black))
                .highlight_symbol(">> ");
            f.render_widget(list, size);
        })?;

        if let event::Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                _ => {}
            }
        }
    }
}
