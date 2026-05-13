use rustbik_cube::{Cube, Scramble};

fn main() {
    // Initialize a new solved cube
    let mut cube = Cube::new();

    // A standard scramble or move sequence in Singmaster notation
    let mv_list = Scramble::new("D L' B' D2 B L2 R2 U' F' R U B2 D F L' R' U' F2 R U2 D' B' R U' R2 F2 B' U' L' R B ");
    println!("Scramble Size: {}\n", mv_list.len());

    // Display the initial state (solved)
    println!("{}", cube);

    // Apply the sequence of moves
    cube.apply_move(mv_list);

    // Display the resulting scrambled state
    println!("{}", cube);
}
