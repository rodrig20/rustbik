# Rustbik

Rustbik is a TUI (Terminal User Interface) utility for Rubik's Cube enthusiasts, focused on high performance and a minimalist design.

This project is developed as a modular monorepo using Rust and the `ratatui` library for terminal rendering.

## Project Structure

- `apps/rustbik-tui`: Terminal interface application (the main program).
- `libs/rustbik-cube`: Core cube logic, including state manipulation.


## How to Run

Ensure you have [Rust installed](https://www.rust-lang.org/tools/install).

1. Clone the repository.
2. From the project root, run:

```bash
cargo run -p rustbik-tui
```

## Controls

- **Menu**: Arrow keys or `j`/`k` to navigate, `Enter` to select, `q` to quit.
- **Timer**: Hold `Spacebar` to prepare, release to start. Press any key to stop.
- **Navigation**: `Esc` to return to the menu.
