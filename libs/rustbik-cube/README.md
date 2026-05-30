# rustbik-cube

`rustbik-cube` is a high-performance 3x3 Rubik's Cube simulation engine for Rust. It is designed to be lightweight, efficient, and easily extensible for solving algorithms and external bindings.

## Core Concepts

- **Bitboard Engine**: Represents the cube state using optimized bit manipulation for edges and corners.
- **Two-Phase Solver**: Native implementation of the Kociemba algorithm for finding solutions under 20 moves.
- **Singmaster Notation**: Full support for parsing and generating standard scramble strings.
- **Python Bindings**: Built with PyO3, allowing the library to be imported directly into Python environments.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
rustbik-cube = { path = "libs/rustbik-cube" }
```

### Basic Example

```rust
use rustbik_cube::{Cube, Scramble};

fn main() {
    let mut cube = Cube::new();
    let scramble = Scramble::new("R U R' U'");
    
    cube.apply(&scramble);
    
    if !cube.is_solved() {
        println!("Cube is scrambled!");
    }
}
```

## Module Structure

- `cube.rs`: Bitboard state and rotation logic.
- `moves.rs`: Scramble parsing and random generation.
- `solvers/`: Specialized solving algorithms (Kociemba).
