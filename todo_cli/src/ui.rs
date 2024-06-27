use crate::input::{InputMode, PrioritySelection};
use crate::sort::SortMode;
use crate::todo::Todo;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem, Paragraph};
use tui::Frame;
use tui::widgets::ListState;

pub fn draw_ui<B: Backend>(
    f: &mut Frame<B>,
    state: &mut ListState,
    search_state: &mut ListState,
    filtered_todos: &Vec<Todo>,
    input_mode: &InputMode,
    input_title: &String,
    input_content: &String,
    input_priority: &PrioritySelection,
    input_deadline: &String,
    search_query: &String,
    sort_mode: &SortMode,
) {
    let size = f.size();
    let (left_chunk, right_chunk) = if *input_mode == InputMode::ViewingDetails {
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
    f.render_stateful_widget(todos_list, chunks[1], state);

    if *input_mode == InputMode::ViewingDetails {
        if let Some(selected) = state.selected() {
            let todo = &filtered_todos[selected];
            let status = if todo.done { "✔ Completed" } else { "✘ Not Completed" };
            let priority = match todo.priority.as_str() {
                "low" => Span::styled("Low", Style::default().fg(Color::Green)),
                "medium" => Span::styled("Medium", Style::default().fg(Color::Yellow)),
                "high" => Span::styled("High", Style::default().fg(Color::Red)),
                _ => Span::raw("Unknown"),
            };
            let deadline = if todo.deadline.is_empty() { "No Deadline" } else { &todo.deadline };
            let details = vec![
                Spans::from(vec![
                    Span::styled("Title: ", Style::default().fg(Color::Cyan)),
                    Span::raw(&todo.title),
                ]),
                Spans::from(vec![
                    Span::styled("Content: ", Style::default().fg(Color::Cyan)),
                    Span::raw(&todo.content),
                ]),
                Spans::from(vec![
                    Span::styled("Priority: ", Style::default().fg(Color::Cyan)),
                    priority,
                ]),
                Spans::from(vec![
                    Span::styled("Deadline: ", Style::default().fg(Color::Cyan)),
                    Span::raw(deadline),
                ]),
                Spans::from(vec![
                    Span::styled("Status: ", Style::default().fg(Color::Cyan)),
                    Span::raw(status),
                ]),
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

    let instructions = match *input_mode {
        InputMode::Normal => {
            String::from("q: Quit | a: Add | d: Delete | e: Edit | l: View Details | <space><space>: Search | s: Sort | j: Down | k: Up | Enter: Toggle Done")
        }
        InputMode::AddingTitle | InputMode::EditingTitle(_) => format!("Enter title: {}", input_title),
        InputMode::AddingContent | InputMode::EditingContent(_) => format!("Enter content: {}", input_content),
        InputMode::AddingPriority | InputMode::EditingPriority(_) => {
            let priority_symbol = match *input_priority {
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

    if *input_mode == InputMode::Searching {
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

        let search_items: Vec<ListItem> = filtered_todos
            .iter()
            .filter(|todo| {
                todo.title.contains(search_query) || todo.content.contains(search_query)
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
        f.render_stateful_widget(search_list, search_chunks[1], search_state);

        f.render_widget(search_input, search_chunks[0]);
    }
}

