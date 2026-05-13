//! Screen management module
//! Defines the traits and actions for managing different UI screens in the application

use crossterm::event::KeyEvent;
use ratatui::Frame;

/// Represents actions a screen can request the application to take
pub enum ScreenAction {
    /// No action to take.
    None,
    /// Request to change to a new screen.
    ChangeScreen(Box<dyn Screen>),
    /// Request to quit the application.
    Quit,
}

/// A trait defining the requirements for a screen in the TUI application
pub trait Screen {
    /// Draws the screen content on the terminal frame.
    fn draw(&mut self, frame: &mut Frame);
    /// Handles keyboard events for the screen and returns the resulting action.
    fn handle_event(&mut self, key: KeyEvent) -> ScreenAction;
}

pub mod main_menu;
pub mod timer;
