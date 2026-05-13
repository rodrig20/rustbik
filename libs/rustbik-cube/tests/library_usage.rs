use rustbik_cube::{Cube, Scramble};

#[test]
fn test_scramble_and_is_solved() {
    let mut cube = Cube::new();

    // Scramble
    let scramble1 = "U R2 F B R B2 L U2 L' B2 R2 U2 D' B2 L2 F2 D' L2 U' B2";
    let moves1 = Scramble::new(scramble1);

    cube.apply_move(moves1);

    assert!(
        !cube.is_solved(),
        "The cube should not be solved after a scramble"
    );

    let scramble2 = "B2 U L2 D F2 L2 B2 D U2 R2 B2 L U2 L' B2 R' B' F' R2 U'";
    let moves2 = Scramble::new(scramble2);

    cube.apply_move(moves2);
    assert!(
        cube.is_solved(),
        "The cube should be solved after reversing the scramble"
    );
}

#[test]
fn test_standard_algorithms() {
    let mut cube = Cube::new();

    // T-Perm (A classic PLL algorithm that swaps specific corners and edges)
    // Applied twice, the cube should return to a solved state
    let t_perm = "R U R' U' R' F R2 U' R' U' R U R' F'";

    cube.apply_move(Scramble::new(t_perm));
    assert!(!cube.is_solved());

    cube.apply_move(Scramble::new(t_perm));
    assert!(
        cube.is_solved(),
        "T-Perm applied twice should solve the cube"
    );
}

#[test]
fn test_pll_sequence() {
    let mut cube = Cube::new();

    let t_perm = "R U R' U' R' F R2 U' R' U' R U R' F'";
    let j_perm = "R U R' F' R U R' U' R' F R2 U' R'";
    let a_perm = "R2 U R U R' U' R' U' R' U R'";

    // Apply T, then J, then A
    cube.apply_move(Scramble::new(t_perm));
    cube.apply_move(Scramble::new(j_perm));
    cube.apply_move(Scramble::new("U'"));
    cube.apply_move(Scramble::new(a_perm));

    assert!(
        cube.is_solved(),
        "Cube should be solved after PLL sequence"
    );
}
