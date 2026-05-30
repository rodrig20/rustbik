//! Main menu screen functionality
//! Displays the main menu for the application

use crate::screens::{Screen, ScreenAction, timer::TimerScreen};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    Frame,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
};

const HIGHLIGHT_COLOR: Color = Color::Rgb(52, 152, 219); // Peter River Blue
const TEXT_COLOR: Color = Color::Rgb(236, 240, 241); // Cloud White

/// The main menu screen
pub struct MainMenuScreen {
    items: Vec<String>,
    state: ListState,
}

impl MainMenuScreen {
    /// Creates a new `MainMenuScreen` with default menu items
    pub fn new() -> Self {
        let items = vec!["Timer".to_string(), "Quit (q)".to_string()];
        let mut state = ListState::default();
        state.select(Some(0));
        Self { items, state }
    }

    /// Moves the selection to the next item in the list
    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => (i + 1) % self.items.len(),
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Moves the selection to the previous item in the list
    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => (i + self.items.len() - 1) % self.items.len(),
            None => 0,
        };
        self.state.select(Some(i));
    }
}

impl Screen for MainMenuScreen {
    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|i| ListItem::new(i.as_str()))
            .collect();

        let menu = List::new(items)
            .block(
                Block::default()
                    .title(" Rustbik ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(HIGHLIGHT_COLOR)),
            )
            .style(Style::default().fg(TEXT_COLOR))
            .highlight_style(
                Style::default()
                    .bg(HIGHLIGHT_COLOR)
                    .fg(Color::Rgb(44, 62, 80)) // Midnight Blue for selected text
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("  ➜ ");

        frame.render_stateful_widget(menu, area, &mut self.state);
    }

    fn handle_event(&mut self, key: KeyEvent) -> ScreenAction {
        if key.kind != KeyEventKind::Press {
            return ScreenAction::None;
        }

        match key.code {
            KeyCode::Char('q') => ScreenAction::Quit,
            KeyCode::Down | KeyCode::Char('j') => {
                self.next();
                ScreenAction::None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.previous();
                ScreenAction::None
            }
            KeyCode::Enter => {
                if let Some(selected) = self.state.selected() {
                    match selected {
                        0 => ScreenAction::ChangeScreen(Box::new(TimerScreen::new())),
                        1 => ScreenAction::Quit,
                        _ => ScreenAction::None,
                    }
                } else {
                    ScreenAction::None
                }
            }
            _ => ScreenAction::None,
        }
    }
}
