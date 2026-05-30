//! TUI application entry point and main event loop
//! This module manages the terminal state, event handling, and screen transitions

use crossterm::{
    ExecutableCommand,
    event::{
        self, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{Result, stdout};
use std::time::Duration;

use crate::screens::{Screen, ScreenAction, main_menu::MainMenuScreen};

/// Represents the state of the TUI application
struct TuiApp {
    /// The currently active screen being rendered and handled
    current_screen: Box<dyn Screen>,
}

impl TuiApp {
    fn new() -> Self {
        Self {
            current_screen: Box::new(MainMenuScreen::new()),
        }
    }
}

/// It sets up the terminal, processes keyboard events, and manages screen switching
pub fn ui_loop() -> Result<()> {
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    let _ = stdout.execute(PushKeyboardEnhancementFlags(
        KeyboardEnhancementFlags::REPORT_EVENT_TYPES,
    ));

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    let mut app = TuiApp::new();

    loop {
        terminal.draw(|frame| app.current_screen.draw(frame))?;

        if event::poll(Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                match app.current_screen.handle_event(key) {
                    ScreenAction::Quit => break,
                    ScreenAction::ChangeScreen(new_screen) => {
                        app.current_screen = new_screen;
                    }
                    ScreenAction::None => {}
                }
            }
        }
    }

    let _ = terminal.backend_mut().execute(PopKeyboardEnhancementFlags);
    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    Ok(())
}
