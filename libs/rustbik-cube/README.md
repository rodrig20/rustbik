# rustbik-cube

`rustbik-cube` is the core library for the Rustbik project, providing a high-performance 3x3 Rubik's Cube simulation engine. It uses bitboards to represent the cube state efficiently and supports arbitrary scramble sequences.

## Features

- **Efficient State Representation**: Uses bitboards for edge and corner tracking.
- **Scramble Support**: Includes a parser for Singmaster notation and random scramble generation.
- **Move Validation**: Fully functional face rotation logic.
- **Zero-Dependency Core**: Lightweight design.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
rustbik-cube = { path = "path/to/libs/rustbik-cube" }
```

### Basic Example

```rust
use rustbik_cube::{Cube, Scramble};

fn main() {
    // Create a new solved cube
    let mut cube = Cube::new();

    // Apply a scramble
    let scramble = Scramble::new("R U R' U'");
    cube.apply_move(scramble);

    // Check state
    if !cube.is_solved() {
        println!("Cube is scrambled!");
    }
}
```

## Architecture

- `cube.rs`: Implements the bitboard-based state and rotation logic.
- `moves.rs`: Handles parsing and random generation of scramble sequences.

