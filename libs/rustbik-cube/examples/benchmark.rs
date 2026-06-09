use std::time::Instant;

use rustbik_cube::Cube;
use rustbik_cube::kociemba::solve;

const N_CUBES: usize = 10_000;
const SCRAMBLE_SIZE: usize = 100;
const PRINT_EVERY: usize = 100;

fn main() -> std::io::Result<()> {
    println!(
        "Running benchmark with {} random cubes (scramble size = {})...\n",
        N_CUBES, SCRAMBLE_SIZE
    );

    let mut times_us: Vec<u128> = Vec::with_capacity(N_CUBES);
    let mut move_counts: Vec<usize> = Vec::with_capacity(N_CUBES);
    let mut failures: usize = 0;

    for i in 0..N_CUBES {
        let cube = Cube::new_random(SCRAMBLE_SIZE);

        let start = Instant::now();
        let solution = solve(&cube);
        let elapsed = start.elapsed();

        match solution {
            Some(path) => {
                let moves = path.len();
                times_us.push(elapsed.as_micros());
                move_counts.push(moves);
                if (i + 1) % PRINT_EVERY == 0 || i == 0 {
                    println!(
                        "[{:>4}/{:>4}]: {:>8.3} ms, {:>2} moves",
                        i + 1,
                        N_CUBES,
                        elapsed.as_secs_f64() * 1000.0,
                        moves,
                    );
                }
            }
            None => {
                failures += 1;
                println!("[{:>4}/{:>4}]: no solution found", i + 1, N_CUBES);
            }
        }
    }

    if times_us.is_empty() {
        eprintln!("No solutions were recorded; nothing to summarize.");
        return Ok(());
    }

    let total = times_us.len();

    let (min_t, max_t, sum_t) = times_us
        .iter()
        .fold((u128::MAX, 0u128, 0u128), |(mn, mx, sm), &t| {
            (mn.min(t), mx.max(t), sm + t)
        });
    let avg_t = sum_t as f64 / total as f64;

    let (min_m, max_m, sum_m) = move_counts
        .iter()
        .fold((usize::MAX, 0usize, 0usize), |(mn, mx, sm), &m| {
            (mn.min(m), mx.max(m), sm + m)
        });
    let avg_m = sum_m as f64 / total as f64;

    println!("\n--- Benchmark summary ---");
    println!("Total cubes:    {}", N_CUBES);
    println!("Solved:         {}  (failed: {})", total, failures);
    println!(
        "Time (us):      min = {:>10}  max = {:>10}  avg = {:>12.2}",
        min_t, max_t, avg_t
    );
    println!(
        "Time (ms):      min = {:>10.3}  max = {:>10.3}  avg = {:>12.3}",
        min_t as f64 / 1000.0,
        max_t as f64 / 1000.0,
        avg_t / 1000.0
    );
    println!(
        "Move count:     min = {:>10}  max = {:>10}  avg = {:>12.2}",
        min_m, max_m, avg_m
    );

    Ok(())
}
