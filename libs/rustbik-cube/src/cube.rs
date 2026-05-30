use crate::moves::{Scramble, SingleMove};
use std::fmt;

/// Represents a 3x3 Rubik's Cube using bitboards for edges and corners
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cube {
    // 12 edges, each taking 5 bits (1 bit for orientation, 4 bits for piece ID)
    pub(crate) edges: u64,
    // 8 corners, each taking 5 bits (2 bits for orientation, 3 bits for piece ID)
    pub(crate) corners: u64,
}

impl Cube {
    // Bitmasks representing the solved state for edges and corners
    const EDGES_SOLVED: u64 =
        0b01011_01010_01001_01000_00111_00110_00101_00100_00011_00010_00001_00000;
    const CORNERS_SOLVED: u64 = 0b00111_00110_00101_00100_00011_00010_00001_00000;

    // Color mapping for edge pieces
    const EDGE_COLORS: [[char; 2]; 12] = [
        ['W', 'R'],
        ['W', 'G'],
        ['W', 'O'],
        ['W', 'B'],
        ['Y', 'R'],
        ['Y', 'G'],
        ['Y', 'O'],
        ['Y', 'B'],
        ['G', 'R'],
        ['G', 'O'],
        ['B', 'O'],
        ['B', 'R'],
    ];

    // Color mapping for corner pieces
    const CORNER_COLORS: [[char; 3]; 8] = [
        ['W', 'G', 'R'],
        ['W', 'G', 'O'],
        ['W', 'B', 'O'],
        ['W', 'B', 'R'],
        ['Y', 'G', 'R'],
        ['Y', 'G', 'O'],
        ['Y', 'B', 'O'],
        ['Y', 'B', 'R'],
    ];

    /// Creates a new Cube in the solved state
    pub fn new() -> Cube {
        Self {
            edges: Self::EDGES_SOLVED,
            corners: Self::CORNERS_SOLVED,
        }
    }

    #[inline(always)]
    pub fn edges(&self) -> u64 {
        self.edges
    }

    #[inline(always)]
    pub fn corners(&self) -> u64 {
        self.corners
    }

    /// Creates a new Cube in a random state
    pub fn new_random(size: usize) -> Self {
        let mut cube = Self {
            edges: Self::EDGES_SOLVED,
            corners: Self::CORNERS_SOLVED,
        };
        let initial_scramble = if size == 0 {
            Scramble::random(25)
        } else {
            Scramble::random(size)
        };

        cube.apply(&initial_scramble);

        cube
    }

    /// Creates a new Cube with a specified scramble
    pub fn new_with(move_list: &Scramble) -> Self {
        let mut cube = Self {
            edges: Self::EDGES_SOLVED,
            corners: Self::CORNERS_SOLVED,
        };

        cube.apply(move_list);

        cube
    }

    pub fn new_from_minimal(representation: u128) -> Self {
        let edges = (representation >> 64) as u64;
        let corners = representation as u64;
        Self { edges, corners }
    }

    /// Checks if the cube is currently in its solved state
    pub fn is_solved(&self) -> bool {
        self.edges == Self::EDGES_SOLVED && self.corners == Self::CORNERS_SOLVED
    }

    /// Applies a list of moves to the cube
    pub fn apply(&mut self, move_list: &Scramble) {
        for mv in move_list.iter() {
            self.turn(mv);
        }
    }
    /// Performs a single face rotation using bitboard manipulation
    /// This method extracts 4 edge and 4 corner pieces from their bitboard slots,
    /// updates their orientation based on precomputed look-up tables (LUTs),
    /// and permutes them into their new positions
    pub fn turn(&mut self, mv: &SingleMove) {
        let data = mv.get_data();

        // Extract current pieces (5 bits each) from the source slots
        let mut e0 = (self.edges >> data.e_shifts[0].0) & 0x1F;
        let mut e1 = (self.edges >> data.e_shifts[1].0) & 0x1F;
        let mut e2 = (self.edges >> data.e_shifts[2].0) & 0x1F;
        let mut e3 = (self.edges >> data.e_shifts[3].0) & 0x1F;

        let mut c0 = (self.corners >> data.c_shifts[0].0) & 0x1F;
        let mut c1 = (self.corners >> data.c_shifts[1].0) & 0x1F;
        let mut c2 = (self.corners >> data.c_shifts[2].0) & 0x1F;
        let mut c3 = (self.corners >> data.c_shifts[3].0) & 0x1F;

        // Clear affected slots using the precomputed masks
        self.edges &= !data.e_mask;
        self.corners &= !data.c_mask;

        // Apply edge orientation updates (flip bit if necessary)
        if data.e_flip {
            e0 ^= 0x10;
            e1 ^= 0x10;
            e2 ^= 0x10;
            e3 ^= 0x10;
        }

        // Apply corner orientation updates using the LUT
        let l = &data.c_ori_lut;
        c0 = (u64::from(l[(c0 >> 3) as usize]) << 3) | (c0 & 0x07);
        c1 = (u64::from(l[(c1 >> 3) as usize]) << 3) | (c1 & 0x07);
        c2 = (u64::from(l[(c2 >> 3) as usize]) << 3) | (c2 & 0x07);
        c3 = (u64::from(l[(c3 >> 3) as usize]) << 3) | (c3 & 0x07);

        // Permute and insert pieces back into their destination slots
        self.edges |= (e0 << data.e_shifts[0].1)
            | (e1 << data.e_shifts[1].1)
            | (e2 << data.e_shifts[2].1)
            | (e3 << data.e_shifts[3].1);

        self.corners |= (c0 << data.c_shifts[0].1)
            | (c1 << data.c_shifts[1].1)
            | (c2 << data.c_shifts[2].1)
            | (c3 << data.c_shifts[3].1);
    }

    /// Retrieves orientation and piece ID for an edge slot
    fn get_edge_slot(&self, i: usize) -> (u8, u8) {
        let val = (self.edges >> (5 * i)) & 0b11111;
        ((val >> 4) as u8, (val & 0b1111) as u8)
    }

    /// Retrieves orientation and piece ID for a corner slot
    fn get_corner_slot(&self, i: usize) -> (u8, u8) {
        let val = (self.corners >> (5 * i)) & 0b11111;
        (((val >> 3) & 0b11) as u8, (val & 0b111) as u8)
    }

    /// Returns the color of a specific edge sticker based on orientation
    fn get_edge_color(&self, slot: usize, sticker: usize) -> char {
        let (ori, pos) = self.get_edge_slot(slot);
        Self::EDGE_COLORS[pos as usize][(sticker ^ ori as usize) % 2]
    }

    /// Returns the color of a specific corner sticker
    /// Accounting for orientation and parity is complex because corner colors
    /// cycle depending on whether the piece is in an "even" or "odd" position relative to its home
    fn get_corner_color(&self, slot: usize, sticker: usize) -> char {
        let (ori, pos) = self.get_corner_slot(slot);

        // Determine if the piece is in a slot with matching parity to its identity
        let slot_parity = (slot % 2) ^ (slot / 4);
        let piece_parity = (pos as usize % 2) ^ (pos as usize / 4);
        let rel_parity = slot_parity ^ piece_parity;

        // Apply orientation transformation based on the relative parity and current orientation
        let idx = match (rel_parity, ori as usize) {
            (0, 1) => [2, 0, 1][sticker],
            (0, 2) => [1, 2, 0][sticker],
            (1, 0) => [0, 2, 1][sticker],
            (1, 1) => [1, 0, 2][sticker],
            (1, 2) => [2, 1, 0][sticker],
            _ => sticker,
        };
        Self::CORNER_COLORS[pos as usize][idx]
    }

    pub fn minimal_representation(&self) -> u128 {
        ((self.edges as u128) << 64) | (self.corners as u128)
    }

    /// Returns representation of the cube
    pub fn net_map(&self) -> String {
        let mut output = String::with_capacity(54); // Cube with 54 stickers
        let faces = self.get_face_data();

        // Logical order for standard net mapping: (U, L, F, R, B, D)
        for &face_idx in &[0, 4, 1, 2, 3, 5] {
            for r in 0..3 {
                for c in 0..3 {
                    output.push(faces[face_idx][r][c]);
                }
            }
        }
        output
    }

    /// Translates a color character to an ANSI-colored block in the terminal
    fn format_sticker(&self, c: char) -> String {
        let code = match c {
            'W' => "\x1b[48;2;255;255;255;38;2;0;0;0m",
            'G' => "\x1b[48;2;0;155;72;38;2;255;255;255m",
            'R' => "\x1b[48;2;183;18;52;38;2;255;255;255m",
            'B' => "\x1b[48;2;0;70;173;38;2;255;255;255m",
            'O' => "\x1b[48;2;255;88;0;38;2;0;0;0m",
            'Y' => "\x1b[48;2;255;213;0;38;2;0;0;0m",
            _ => "\x1b[0m",
        };
        format!("{} {} \x1b[0m", code, c)
    }

    /// Generates a 3D array representing the 6 faces (3x3 each) from the bitboard state
    fn get_face_data(&self) -> [[[char; 3]; 3]; 6] {
        let mut f = [[[' '; 3]; 3]; 6];

        // Map current cube state to a 2D face array
        // U face
        f[0][0][0] = self.get_corner_color(2, 0);
        f[0][0][1] = self.get_edge_color(3, 0);
        f[0][0][2] = self.get_corner_color(3, 0);
        f[0][1][0] = self.get_edge_color(2, 0);
        f[0][1][1] = 'W';
        f[0][1][2] = self.get_edge_color(0, 0);
        f[0][2][0] = self.get_corner_color(1, 0);
        f[0][2][1] = self.get_edge_color(1, 0);
        f[0][2][2] = self.get_corner_color(0, 0);

        // F face
        f[1][0][0] = self.get_corner_color(1, 1);
        f[1][0][1] = self.get_edge_color(1, 1);
        f[1][0][2] = self.get_corner_color(0, 1);
        f[1][1][0] = self.get_edge_color(9, 0);
        f[1][1][1] = 'G';
        f[1][1][2] = self.get_edge_color(8, 0);
        f[1][2][0] = self.get_corner_color(5, 1);
        f[1][2][1] = self.get_edge_color(5, 1);
        f[1][2][2] = self.get_corner_color(4, 1);

        // R face
        f[2][0][0] = self.get_corner_color(0, 2);
        f[2][0][1] = self.get_edge_color(0, 1);
        f[2][0][2] = self.get_corner_color(3, 2);
        f[2][1][0] = self.get_edge_color(8, 1);
        f[2][1][1] = 'R';
        f[2][1][2] = self.get_edge_color(11, 1);
        f[2][2][0] = self.get_corner_color(4, 2);
        f[2][2][1] = self.get_edge_color(4, 1);
        f[2][2][2] = self.get_corner_color(7, 2);

        // B face
        f[3][0][0] = self.get_corner_color(3, 1);
        f[3][0][1] = self.get_edge_color(3, 1);
        f[3][0][2] = self.get_corner_color(2, 1);
        f[3][1][0] = self.get_edge_color(11, 0);
        f[3][1][1] = 'B';
        f[3][1][2] = self.get_edge_color(10, 0);
        f[3][2][0] = self.get_corner_color(7, 1);
        f[3][2][1] = self.get_edge_color(7, 1);
        f[3][2][2] = self.get_corner_color(6, 1);

        // L face
        f[4][0][0] = self.get_corner_color(2, 2);
        f[4][0][1] = self.get_edge_color(2, 1);
        f[4][0][2] = self.get_corner_color(1, 2);
        f[4][1][0] = self.get_edge_color(10, 1);
        f[4][1][1] = 'O';
        f[4][1][2] = self.get_edge_color(9, 1);
        f[4][2][0] = self.get_corner_color(6, 2);
        f[4][2][1] = self.get_edge_color(6, 1);
        f[4][2][2] = self.get_corner_color(5, 2);

        // D face
        f[5][0][0] = self.get_corner_color(5, 0);
        f[5][0][1] = self.get_edge_color(5, 0);
        f[5][0][2] = self.get_corner_color(4, 0);
        f[5][1][0] = self.get_edge_color(6, 0);
        f[5][1][1] = 'Y';
        f[5][1][2] = self.get_edge_color(4, 0);
        f[5][2][0] = self.get_corner_color(6, 0);
        f[5][2][1] = self.get_edge_color(7, 0);
        f[5][2][2] = self.get_corner_color(7, 0);

        f
    }
}

impl fmt::Display for Cube {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let raw_str = self.net_map();
        let raw: Vec<char> = raw_str.chars().collect();

        // Helper to get a specific face (9 stickers) from the raw string
        let get_face = |i: usize| &raw[(i * 9)..(i * 9 + 9)];

        // UP - face index 0
        let u = get_face(0);
        for r in 0..3 {
            write!(f, "         ")?; // Padding for center alignment
            for c in 0..3 {
                write!(f, "{}", self.format_sticker(u[r * 3 + c]))?;
            }
            writeln!(f)?;
        }

        // LEFT, FRONT, RIGHT, BACK - faces 1, 2, 3, 4
        let l = get_face(1);
        let front = get_face(2);
        let r_face = get_face(3);
        let b = get_face(4);

        for r in 0..3 {
            for c in 0..3 {
                write!(f, "{}", self.format_sticker(l[r * 3 + c]))?;
            }
            for c in 0..3 {
                write!(f, "{}", self.format_sticker(front[r * 3 + c]))?;
            }
            for c in 0..3 {
                write!(f, "{}", self.format_sticker(r_face[r * 3 + c]))?;
            }
            for c in 0..3 {
                write!(f, "{}", self.format_sticker(b[r * 3 + c]))?;
            }
            writeln!(f)?;
        }

        // DOWN - face index 5
        let d = get_face(5);
        for r in 0..3 {
            write!(f, "         ")?;
            for c in 0..3 {
                write!(f, "{}", self.format_sticker(d[r * 3 + c]))?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cube_is_solved() {
        let cube = Cube::new();
        assert!(cube.is_solved());
    }

    #[test]
    fn test_single_move_unsolves() {
        let mut cube = Cube::new();
        cube.apply(&Scramble::new("R"));
        assert!(!cube.is_solved());
    }

    #[test]
    fn test_move_inverse() {
        let mut cube = Cube::new();
        cube.apply(&Scramble::new("R R'"));
        assert!(cube.is_solved());

        cube.apply(&Scramble::new("U U'"));
        assert!(cube.is_solved());

        cube.apply(&Scramble::new("F F'"));
        assert!(cube.is_solved());

        cube.apply(&Scramble::new("L L'"));
        assert!(cube.is_solved());

        cube.apply(&Scramble::new("B B'"));
        assert!(cube.is_solved());

        cube.apply(&Scramble::new("D D'"));
        assert!(cube.is_solved());
    }

    #[test]
    fn test_double_move() {
        let mut cube = Cube::new();
        cube.apply(&Scramble::new("R2 R2"));
        assert!(cube.is_solved());

        cube.apply(&Scramble::new("U2 U2"));
        assert!(cube.is_solved());

        cube.apply(&Scramble::new("F2 F2"));
        assert!(cube.is_solved());

        cube.apply(&Scramble::new("L2 L2"));
        assert!(cube.is_solved());

        cube.apply(&Scramble::new("B2 B2"));
        assert!(cube.is_solved());

        cube.apply(&Scramble::new("D2 D2"));
        assert!(cube.is_solved());
    }

    #[test]
    fn test_sexy_move_cycle() {
        let mut cube = Cube::new();
        // The "sexy move" (R U R' U') repeated 6 times returns the cube to solved
        for _ in 0..6 {
            cube.apply(&Scramble::new("R U R' U'"));
        }
        assert!(cube.is_solved());
    }

    #[test]
    fn test_minimal_representation_cycle() {
        let cube = Cube::new_random(10);
        let min_rep = cube.minimal_representation();
        let cube2 = Cube::new_from_minimal(min_rep);
        assert_eq!(cube, cube2);
    }

    #[test]
    fn test_new_random_not_solved() {
        let cube = Cube::new_random(10);
        assert!(!cube.is_solved());
    }
}
