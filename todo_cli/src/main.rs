use serde::{Serialize, Deserialize}; // SerdeライブラリのSerializeとDeserializeをインポート
use std::fs::File; // ファイル操作のための標準ライブラリ
use std::io::{self, Read, Write}; // 入出力操作のための標準ライブラリ
use serde_json::Result as SerdeResult; // Serde JSONの結果型をインポートして別名をつける
use crossterm::{ // Crosstermライブラリをインポート
    event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{stdout, Write as IoWrite}; // 標準出力と書き込み操作をインポート
use tui::{ // TUIライブラリをインポート
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

// Todo項目を定義する構造体。タスクID、タスク内容、完了状態を持つ
#[derive(Serialize, Deserialize, Debug)]
struct TodoItem {
    id: u32,
    task: String,
    done: bool,
}

const FILE_PATH: &str = "todo_list.json"; // JSONファイルのパス

// JSONファイルからTodo項目を読み込む関数
fn read_todos() -> SerdeResult<Vec<TodoItem>> {
    let mut file = match File::open(FILE_PATH) {
        Ok(file) => file, // ファイルが存在する場合は開く
        Err(_) => File::create(FILE_PATH).expect("Failed to create file"), // ファイルが存在しない場合は新しく作成する
    };
    let mut data = String::new();
    file.read_to_string(&mut data).expect("Failed to read file"); // ファイル内容を文字列に読み込む
    if data.is_empty() {
        Ok(vec![]) // ファイルが空の場合は空のベクタを返す
    } else {
        serde_json::from_str(&data) // JSON文字列をデシリアライズして返す
    }
}

// Todo項目をJSONファイルに書き込む関数
fn write_todos(todos: &Vec<TodoItem>) -> io::Result<()> {
    let data = serde_json::to_string(todos).expect("Failed to serialize todos"); // Todo項目をJSON文字列にシリアライズ
    let mut file = File::create(FILE_PATH)?; // ファイルを新しく作成
    file.write_all(data.as_bytes()) // ファイルに書き込む
}

// メイン関数
fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?; // ターミナルをrawモードに設定
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?; // ターミナルを拡張スクリーンモードに設定し、マウスキャプチャを有効化
    let backend = CrosstermBackend::new(stdout); // Crosstermのバックエンドを作成
    let mut terminal = Terminal::new(backend)?; // TUIターミナルを作成

    let res = run_app(&mut terminal); // アプリケーションを実行

    disable_raw_mode()?; // rawモードを無効化
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?; // 拡張スクリーンモードを終了し、マウスキャプチャを無効化
    terminal.show_cursor()?; // カーソルを表示

    if let Err(err) = res {
        println!("{:?}", err) // エラーが発生した場合は表示
    }

    Ok(())
}

// アプリケーションのモードを定義
enum Mode {
    Normal, // 通常モード
    Adding(String), // 追加モード
    Editing(u32, String), // 編集モード
    ConfirmDelete(u32), // 削除確認モード
    Help, // ヘルプモード
}

// アプリケーションのメインループ
fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let mut mode = Mode::Normal; // 初期モードは通常モード
    let mut cursor_pos = 0; // カーソル位置の初期値
    let mut tasks = read_todos().unwrap(); // Todo項目を読み込む
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(50), // 左右のレイアウトを50%ずつに分割
                        Constraint::Percentage(50),
                    ].as_ref()
                )
                .split(f.size());

            let block = Block::default()
                .title("Todo List")
                .borders(Borders::ALL); // Todoリストのブロックを作成
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
                        .borders(Borders::ALL); // 追加モードのブロックを作成
                    let paragraph = Paragraph::new(format!("title: {}", input)).block(block);
                    f.render_widget(paragraph, chunks[1]);
                }
                Mode::Editing(_, input) => {
                    let block = Block::default()
                        .title("Edit Task")
                        .borders(Borders::ALL); // 編集モードのブロックを作成
                    let paragraph = Paragraph::new(format!("title: {}", input)).block(block);
                    f.render_widget(paragraph, chunks[1]);
                }
                Mode::ConfirmDelete(id) => {
                    let block = Block::default()
                        .title("Delete Task")
                        .borders(Borders::ALL); // 削除確認モードのブロックを作成
                    let paragraph = Paragraph::new(format!("Are you sure you want to delete task with ID {}? (y/n): ", id)).block(block);
                    f.render_widget(paragraph, chunks[1]);
                }
                Mode::Help => {
                    let block = Block::default()
                        .title("Help")
                        .borders(Borders::ALL); // ヘルプモードのブロックを作成
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

        // キーイベントを処理
        if let event::Event::Key(KeyEvent { code, modifiers }) = event::read()? {
            match code {
                KeyCode::Char('q') if matches!(mode, Mode::Normal) => return Ok(()), // 通常モードでqを押すと終了
                KeyCode::Char('a') if matches!(mode, Mode::Normal) => mode = Mode::Adding(String::new()), // 通常モードでaを押すと追加モードに切り替え
                KeyCode::Char('e') if matches!(mode, Mode::Normal) => {
                    if let Some(todo) = tasks.get(cursor_pos) {
                        mode = Mode::Editing(todo.id, todo.task.clone()); // 通常モードでeを押すと編集モードに切り替え
                    }
                }
                KeyCode::Char('d') if matches!(mode, Mode::Normal) => {
                    if let Some(todo) = tasks.get(cursor_pos) {
                        mode = Mode::ConfirmDelete(todo.id); // 通常モードでdを押すと削除確認モードに切り替え
                    }
                }
                KeyCode::Char('?') if matches!(mode, Mode::Normal) => mode = Mode::Help, // 通常モードで?を押すとヘルプモードに切り替え
                KeyCode::Char('k') if matches!(mode, Mode::Normal) => {
                    if cursor_pos > 0 {
                        cursor_pos -= 1; // 通常モードでkを押すとカーソルを上に移動
                    }
                }
                KeyCode::Char('j') if matches!(mode, Mode::Normal) => {
                    if cursor_pos < tasks.len() - 1 {
                        cursor_pos += 1; // 通常モードでjを押すとカーソルを下に移動
                    }
                }
                KeyCode::Char('l') if matches!(mode, Mode::ConfirmDelete(_)) => {
                    if let Mode::ConfirmDelete(id) = mode {
                        delete_todo_by_id(id); // 削除確認モードでlを押すとタスクを削除
                        tasks = read_todos().unwrap();
                        mode = Mode::Normal; // 通常モードに戻る
                    }
                }
                KeyCode::Char('h') if !matches!(mode, Mode::Normal) => {
                    mode = Mode::Normal; // 他のモードでhを押すと通常モードに戻る
                }
                KeyCode::Char('y') if matches!(mode, Mode::ConfirmDelete(_)) => {
                    if let Mode::ConfirmDelete(id) = mode {
                        delete_todo_by_id(id); // 削除確認モードでyを押すとタスクを削除
                        tasks = read_todos().unwrap();
                        mode = Mode::Normal; // 通常モードに戻る
                    }
                }
                KeyCode::Char('n') if matches!(mode, Mode::ConfirmDelete(_)) => {
                    if let Mode::ConfirmDelete(_) = mode {
                        mode = Mode::Normal; // 削除確認モードでnを押すと通常モードに戻る
                    }
                }
                KeyCode::Enter => {
                    match &mode {
                        Mode::Adding(input) => {
                            create_todo(input.clone()); // 追加モードでEnterを押すとタスクを追加
                            tasks = read_todos().unwrap();
                            mode = Mode::Normal; // 通常モードに戻る
                        }
                        Mode::Editing(id, input) => {
                            update_todo(*id, Some(input.clone()), None); // 編集モードでEnterを押すとタスクを更新
                            tasks = read_todos().unwrap();
                            mode = Mode::Normal; // 通常モードに戻る
                        }
                        _ => {}
                    }
                }
                KeyCode::Char(c) if matches!(mode, Mode::Adding(_) | Mode::Editing(_, _)) => {
                    match &mut mode {
                        Mode::Adding(input) | Mode::Editing(_, input) => {
                            input.push(c); // 追加モードまたは編集モードで文字を入力
                        }
                        _ => {}
                    }
                }
                KeyCode::Backspace if matches!(mode, Mode::Adding(_) | Mode::Editing(_, _)) => {
                    match &mut mode {
                        Mode::Adding(input) | Mode::Editing(_, input) => {
                            input.pop(); // 追加モードまたは編集モードでBackspaceを押すと文字を削除
                        }
                        _ => {}
                    }
                }
                KeyCode::Esc if matches!(mode, Mode::Adding(_) | Mode::Editing(_, _) | Mode::ConfirmDelete(_)) => {
                    mode = Mode::Normal; // Escキーで通常モードに戻る
                }
                _ => {}
            }
        }
    }
}

// タスクIDでTodo項目を削除する関数
fn delete_todo_by_id(id: u32) {
    let mut todos = read_todos().expect("Failed to read todos");
    todos.retain(|todo| todo.id != id); // 指定されたIDのタスクを除去
    write_todos(&todos).expect("Failed to write todos"); // 更新されたTodoリストをファイルに書き込む
}

// 新しいTodo項目を作成する関数
fn create_todo(task: String) {
    let mut todos = read_todos().expect("Failed to read todos");
    let id = if let Some(last) = todos.last() { last.id + 1 } else { 1 }; // 新しいIDを生成
    let todo = TodoItem { id, task, done: false }; // 新しいTodo項目を作成
    todos.push(todo); // Todoリストに追加
    write_todos(&todos).expect("Failed to write todos"); // 更新されたTodoリストをファイルに書き込む
}

// Todo項目を更新する関数
fn update_todo(id: u32, task: Option<String>, done: Option<bool>) {
    let mut todos = read_todos().expect("Failed to read todos");
    if let Some(todo) = todos.iter_mut().find(|todo| todo.id == id) {
        if let Some(task) = task {
            todo.task = task; // タスク内容を更新
        }
        if let Some(done) = done {
            todo.done = done; // 完了状態を更新
        }
    }
    write_todos(&todos).expect("Failed to write todos"); // 更新されたTodoリストをファイルに書き込む
}
