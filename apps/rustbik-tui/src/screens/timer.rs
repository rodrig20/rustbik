//! Timer screen functionality.
//! Handles Rubik's cube timing, scrambling, and visualization.

use crate::screens::{Screen, ScreenAction, main_menu::MainMenuScreen};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use rustbik_cube::{Cube, Scramble};
use std::time::{Duration, Instant};
use tui_big_text::{BigText, PixelSize};

const COLOR_HOLDING: Color = Color::Rgb(241, 196, 15);
const COLOR_READY: Color = Color::Rgb(46, 204, 113);
const COLOR_IDLE: Color = Color::Rgb(236, 240, 241);

/// Helper struct for timer formatting and UI utility calculations.
struct TimerUtils;

impl TimerUtils {
    /// Splits a duration into main (minutes:seconds) and fractional (cents) parts.
    fn split_duration(duration: Duration, precision: usize) -> (String, String) {
        let total_secs = duration.as_secs();
        let minutes = total_secs / 60;
        let seconds = total_secs % 60;
        let millis = duration.subsec_millis();
        let main = if minutes > 0 {
            format!("{}:{:02}", minutes, seconds)
        } else {
            format!("{:1}", seconds)
        };
        let cents = match precision {
            2 => format!(".{:02}", (millis / 10) % 100),
            _ => format!(".{:01}", millis / 100),
        };
        (main, cents)
    }

    /// Checks if the timer display can fit within the current screen area.
    fn can_fit_full_size(area: Rect, text_len: usize) -> bool {
        area.height >= 12 && area.width >= (text_len as u16 * 8) && area.width >= 50
    }

    /// Estimates the width of a text string in pixels based on character count.
    fn estimate_text_width(text: &str) -> u16 {
        (text.len() as u16) * 8
    }
}

/// The main timer screen displaying the cube state and timing interface.
pub struct TimerScreen {
    start_hold_time: Option<Instant>,
    running_since: Option<Instant>,
    last_duration: Duration,
    is_running: bool,
    cube: Cube,
    scramble_str: String,
}

impl TimerScreen {
    /// Creates a new `TimerScreen` with a fresh scramble.
    pub fn new() -> Self {
        let scramble = Scramble::random(25);
        let scramble_str = scramble.to_string();
        Self {
            start_hold_time: None,
            running_since: None,
            last_duration: Duration::ZERO,
            is_running: false,
            cube: Cube::new_with(scramble),
            scramble_str,
        }
    }

    /// Returns the elapsed duration if running, or the last completed time.
    fn get_current_duration(&self) -> Duration {
        if self.is_running {
            self.running_since
                .map(|s| s.elapsed())
                .unwrap_or(Duration::ZERO)
        } else {
            self.last_duration
        }
    }

    /// Determines the current display color based on timer state.
    fn get_timer_color(&self) -> Color {
        if !self.is_running {
            if let Some(start) = self.start_hold_time {
                return if start.elapsed() >= Duration::from_millis(300) {
                    COLOR_READY
                } else {
                    COLOR_HOLDING
                };
            }
        }
        COLOR_IDLE
    }

    /// Renders the timer display (big text) to the frame.
    /// Renders the timer display (big text) to the frame.
    fn draw_timer_display(
        &self,
        frame: &mut Frame,
        area: Rect,
        main_str: &str,
        cents_str: &str,
        color: Color,
    ) {
        // Calculate the vertical and horizontal centering for the timer
        let y_offset = (area.height.saturating_sub(8)) / 2;
        let centered_area = Rect::new(area.x, area.y + y_offset, area.width, 8);

        let main_width = TimerUtils::estimate_text_width(main_str);
        let cents_width = TimerUtils::estimate_text_width(cents_str);
        let total_width = main_width + cents_width;
        let x_offset = centered_area.x + (centered_area.width.saturating_sub(total_width)) / 2;

        // Split the area to position the main timer and fractional cents separately
        let h_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(x_offset),
                Constraint::Length(main_width),
                Constraint::Length(cents_width),
                Constraint::Min(0),
            ])
            .split(centered_area);

        // Render the main seconds/minutes in a large format
        frame.render_widget(
            BigText::builder()
                .pixel_size(PixelSize::Full)
                .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
                .lines(vec![main_str.into()])
                .build(),
            h_chunks[1],
        );

        // Render the fractional cents in a slightly smaller format if space allows
        if h_chunks.len() > 2 {
            frame.render_widget(
                BigText::builder()
                    .pixel_size(PixelSize::HalfHeight)
                    .style(Style::default().fg(color))
                    .lines(vec![cents_str.into()])
                    .build(),
                Rect::new(h_chunks[2].x, h_chunks[2].y + 3, h_chunks[2].width, 8),
            );
        }
    }

    /// Renders the 2D representation of the cube net.
    /// Renders the 2D representation of the cube net.
    fn draw_cube_net(&self, frame: &mut Frame, area: Rect) {
        // Retrieve the cube net structure as a flat string and prepare lines for rendering
        let raw = self.cube.net_map();
        let mut lines = Vec::new();

        // Helper closure to map cube sticker characters to terminal colors
        let color_for = |c| match c {
            'W' => Color::Rgb(255, 255, 255),
            'G' => Color::Rgb(0, 155, 72),
            'R' => Color::Rgb(183, 18, 52),
            'B' => Color::Rgb(0, 70, 173),
            'O' => Color::Rgb(255, 88, 0),
            'Y' => Color::Rgb(255, 213, 0),
            _ => Color::Black,
        };

        // Construct the 9x12 grid visualization, iterating through rows and columns
        for row in 0..9 {
            let mut line = Vec::new();
            for col in 0..12 {
                // Map logical grid coordinates to indices in the cube net data
                let ch = match (row, col) {
                    (0..3, 3..6) => raw.chars().nth(0 * 9 + ((row - 0) * 3) + (col - 3)), // U
                    (3..6, 0..3) => raw.chars().nth(1 * 9 + ((row - 3) * 3) + (col - 0)), // L
                    (3..6, 3..6) => raw.chars().nth(2 * 9 + ((row - 3) * 3) + (col - 3)), // F
                    (3..6, 6..9) => raw.chars().nth(3 * 9 + ((row - 3) * 3) + (col - 6)), // R
                    (3..6, 9..12) => raw.chars().nth(4 * 9 + ((row - 3) * 3) + (col - 9)), // B
                    (6..9, 3..6) => raw.chars().nth(5 * 9 + ((row - 6) * 3) + (col - 3)), // D
                    _ => None,
                };

                // Create the visual elements (boxes) for each sticker
                match ch {
                    Some(c) => line.push({
                        let border_style = Style::default().fg(Color::Black).bg(color_for(c));

                        match (row % 3, col % 3) {
                            (0, 0) => Span::styled("╔═", border_style),
                            (0, 1) | (2, 1) => Span::styled("══", border_style),
                            (0, 2) => Span::styled("═╗", border_style),

                            (1, 0) => Span::styled("║ ", border_style),
                            (1, 2) => Span::styled(" ║", border_style),

                            (2, 0) => Span::styled("╚═", border_style),
                            (2, 2) => Span::styled("═╝", border_style),

                            _ => Span::styled("  ", border_style),
                        }
                    }),
                    None => line.push(Span::raw("  ")), // Two spaces to maintain aspect ratio
                }
            }
            lines.push(Line::from(line));
        }

        // Finalize and render the cube net widget
        let net = Paragraph::new(lines).alignment(Alignment::Center);
        frame.render_widget(
            net.block(Block::default().title(" Cube ").borders(Borders::ALL)),
            area,
        );
    }
}

impl Screen for TimerScreen {
    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let display_duration = self.get_current_duration();
        let (main_str, cents_str) =
            TimerUtils::split_duration(display_duration, if self.is_running { 1 } else { 2 });
        let full_text = format!("{}{}", main_str, cents_str);

        if !TimerUtils::can_fit_full_size(area, full_text.len()) {
            if self.is_running {
                self.last_duration = self.get_current_duration();
                self.is_running = false;
                self.running_since = None;

                let scramble = Scramble::random(25);
                let scramble_str = scramble.to_string();

                self.cube = Cube::new_with(scramble);
                self.scramble_str = scramble_str;
            }
            let warning_text = Paragraph::new("Window too small").alignment(Alignment::Center);
            frame.render_widget(
                warning_text,
                Rect::new(area.x, area.y + area.height / 2, area.width, 1),
            );
            return;
        }

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Min(8),
                Constraint::Length(1),
            ])
            .split(area);

        if !self.is_running
            && !self
                .start_hold_time
                .map(|s: Instant| s.elapsed() >= Duration::from_millis(300))
                .unwrap_or(false)
        {
            frame.render_widget(
                Paragraph::new(self.scramble_str.as_str())
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Gray)),
                main_chunks[0],
            );
        }

        self.draw_timer_display(
            frame,
            main_chunks[1],
            &main_str,
            &cents_str,
            self.get_timer_color(),
        );

        if !self.is_running
            && !self
                .start_hold_time
                .map(|s: Instant| s.elapsed() >= Duration::from_millis(300))
                .unwrap_or(false)
        {
            let cube_area = Rect::new(
                area.width.saturating_sub(28),
                area.height.saturating_sub(12),
                26,
                11,
            );
            self.draw_cube_net(frame, cube_area);
        }

        let footer_text = if self.is_running {
            "Press any key to stop"
        } else if self.start_hold_time.is_some() {
            if self
                .start_hold_time
                .map(|s| s.elapsed() >= Duration::from_millis(300))
                .unwrap_or(false)
            {
                "RELEASE TO START!"
            } else {
                "HOLDING..."
            }
        } else {
            "Hold 'Space' to prepare | 'Esc' to go back"
        };
        frame.render_widget(
            Paragraph::new(footer_text)
                .alignment(Alignment::Center)
                .style(Style::default().bg(Color::Rgb(44, 62, 80)).fg(Color::White)),
            main_chunks[2],
        );
    }

    fn handle_event(&mut self, key: KeyEvent) -> ScreenAction {
        if key.kind != KeyEventKind::Press && key.kind != KeyEventKind::Release {
            return ScreenAction::None;
        }
        if self.is_running {
            if key.kind == KeyEventKind::Press {
                self.last_duration = self.get_current_duration();
                self.is_running = false;
                self.running_since = None;

                let scramble = Scramble::random(25);
                let scramble_str = scramble.to_string();

                self.cube = Cube::new_with(scramble);
                self.scramble_str = scramble_str;
            }
            return ScreenAction::None;
        }
        match key.code {
            KeyCode::Char(' ') => match key.kind {
                KeyEventKind::Press => {
                    if self.start_hold_time.is_none() {
                        self.start_hold_time = Some(Instant::now());
                    }
                }
                KeyEventKind::Release => {
                    if self
                        .start_hold_time
                        .map(|s| s.elapsed() >= Duration::from_millis(300))
                        .unwrap_or(false)
                    {
                        self.is_running = true;
                        self.running_since = Some(Instant::now());
                        self.last_duration = Duration::ZERO;
                    }
                    self.start_hold_time = None;
                }
                _ => {}
            },
            KeyCode::Esc => {
                if !self.is_running {
                    return ScreenAction::ChangeScreen(Box::new(MainMenuScreen::new()));
                }
            }
            _ => {}
        }
        ScreenAction::None
    }
}
