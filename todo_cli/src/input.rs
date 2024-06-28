#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    AddingTitle,
    AddingContent,
    AddingPriority,
    AddingDeadline,
    ViewingDetails(usize),
    ViewingSubtaskDetails(usize, usize), // サブタスク詳細表示のモードを追加
    Searching,
    EditingTitle(usize),
    EditingContent(usize),
    EditingPriority(usize),
    EditingDeadline(usize),
    AddingSubtask(usize),
}

#[derive(PartialEq, Copy, Clone)]
pub enum PrioritySelection {
    Low,
    Medium,
    High,
}

impl PrioritySelection {
    pub fn next(&self) -> Self {
        match *self {
            PrioritySelection::Low => PrioritySelection::Medium,
            PrioritySelection::Medium => PrioritySelection::High,
            PrioritySelection::High => PrioritySelection::Low,
        }
    }

    pub fn prev(&self) -> Self {
        match *self {
            PrioritySelection::Low => PrioritySelection::High,
            PrioritySelection::Medium => PrioritySelection::Low,
            PrioritySelection::High => PrioritySelection::Medium,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match *self {
            PrioritySelection::Low => "low",
            PrioritySelection::Medium => "medium",
            PrioritySelection::High => "high",
        }
    }

    pub fn color(&self) -> tui::style::Color {
        match *self {
            PrioritySelection::Low => tui::style::Color::Green,
            PrioritySelection::Medium => tui::style::Color::Yellow,
            PrioritySelection::High => tui::style::Color::Red,
        }
    }
}
