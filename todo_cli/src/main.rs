mod add_task;
mod delete_task;
mod edit_task;
mod handle_input;
mod input;
mod search;
mod sort;
mod todo;
mod ui;

use crate::handle_input::handle_input;
use crate::input::{InputMode, PrioritySelection};
use crate::sort::SortMode;
use crate::todo::{load_todos, save_todos, Todo};
use crate::ui::draw_ui;

use crossterm::cursor::Hide;
use crossterm::event::{self, Event};
use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen};
use crossterm::ExecutableCommand;
use std::io;
use tui::backend::CrosstermBackend;
use tui::Terminal;
use tui::widgets::ListState;

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
            draw_ui(
                f,
                &mut state,
                &mut search_state,
                &filtered_todos,
                &input_mode,
                &input_title,
                &input_content,
                &input_priority,
                &input_deadline,
                &search_query,
                &sort_mode,
            );
        })?;

        if let Event::Key(key) = event::read()? {
            handle_input(
                key,
                &mut todos,
                &mut filtered_todos,
                &mut state,
                &mut input_mode,
                &mut input_title,
                &mut input_content,
                &mut input_priority,
                &mut input_deadline,
                &mut search_query,
                &mut search_state,
                &mut sort_mode,
                &mut terminal,
            )?;
        }
    }
}
