use std::time::{Duration, Instant};

use rustbik_cube::kociemba::{solve, solve_max_moves, solve_time_limit};
use rustbik_cube::{Cube, Scramble};

fn main() -> std::io::Result<()> {
    println!("\nTesting scramble...");

    // Define a complex scramble sequence using Singmaster notation
    let scramble = Scramble::new("U R2 F B R B2 R U2 L B2 R U' D' R2 F R' L B2 U2 F2");
    // Initialize a new solved cube and apply the scramble
    let mut cube = Cube::new();
    cube.apply(&scramble);

    println!("Scramble: {}", scramble.to_str());
    println!("{}", cube); // Uses the Display implementation to show the 2D net map
    println!("Searching for solution...");

    // Invoke the Kociemba two-phase solver
    // This requires precomputed lookup tables to be present in the tables/ directory
    let start = Instant::now();
    match solve_max_moves(&cube, 20) {
        Some(path) => {
            let end = start.elapsed();
            println!(
                "Found solution in {}ms of {} moves:",
                end.as_millis(),
                path.len()
            );

            // Apply each move of the solution to the cube to verify it reaches the solved state
            for mv in path {
                cube.turn(&mv);
                print!("{} ", mv.to_string());
            }
            print!("\n");

            // Print the final cube state (should be solved)
            println!("{}", cube);
        }
        None => println!("No solution found within depth limit,"),
    }

    Ok(())
}
