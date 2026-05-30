# Rustbik

Rustbik is a high-performance Rubik's Cube utility suite built with Rust, featuring a terminal-based simulator and a powerful solving engine.

## Key Components

- **Terminal UI**: A full-featured TUI for scrambling, timing, and visualizing moves.
- **Solving Engine**: High-speed implementation of the Kociemba algorithm.
- **Python Integration**: The core engine is available as a Python module for research and automation.

## Project Structure

- `apps/rustbik-tui`: The terminal interface (Ratatui).
- `libs/rustbik-cube`: The core simulation and solving library.

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
