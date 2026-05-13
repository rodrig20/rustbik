use crate::moves::{MoveAxis, MoveDirection, Scramble, SingleMove};
use std::fmt;

/// Represents a 3x3 Rubik's Cube using bitboards for edges and corners
pub struct Cube {
    // 12 edges, each taking 5 bits (1 bit for orientation, 4 bits for piece ID)
    edges: u64,
    // 8 corners, each taking 5 bits (2 bits for orientation, 3 bits for piece ID)
    corners: u64,
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
        ['G', 'R'],
        ['G', 'O'],
        ['B', 'O'],
        ['B', 'R'],
        ['Y', 'R'],
        ['Y', 'G'],
        ['Y', 'O'],
        ['Y', 'B'],
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

        cube.apply_move(initial_scramble);

        cube
    }

    /// Creates a new Cube with a specified scramble
    pub fn new_with(move_list: Scramble) -> Self {
        let mut cube = Self {
            edges: Self::EDGES_SOLVED,
            corners: Self::CORNERS_SOLVED,
        };

        cube.apply_move(move_list);

        cube
    }

    /// Checks if the cube is currently in its solved state
    pub fn is_solved(&self) -> bool {
        self.edges == Self::EDGES_SOLVED && self.corners == Self::CORNERS_SOLVED
    }

    /// Applies a list of moves to the cube
    pub fn apply_move(&mut self, move_list: Scramble) {
        for mv in move_list.iter() {
            self.move_side(mv);
        }
    }

    /// Performs a single face rotation
    fn move_side(&mut self, mv: &SingleMove) {
        let (edge_mask_idxs, corner_mask_idxs, corner_ori_swap) = mv.mask();

        match mv.dir {
            MoveDirection::Clk => {
                let mut corner_list_of_values = [(0u8, 0u8); 4];
                let mut edge_list_of_values = [(0u8, 0u8); 4];

                // Extract pieces currently at the affected slots
                for i in 0..4 {
                    corner_list_of_values[i] = self.get_corner_slot(corner_mask_idxs[i]);
                    edge_list_of_values[i] = self.get_edge_slot(edge_mask_idxs[i]);
                }

                // Cycle pieces and update orientations
                for i in 0..4 {
                    // Corners
                    let (corner_old_ori, corner_id) = corner_list_of_values[i];

                    let corner_new_ori = if u64::from(corner_old_ori) == corner_ori_swap[0] {
                        corner_ori_swap[1] as u8
                    } else if u64::from(corner_old_ori) == corner_ori_swap[1] {
                        corner_ori_swap[0] as u8
                    } else {
                        corner_old_ori
                    };

                    self.set_corner_slot(corner_mask_idxs[(i + 1) % 4], corner_new_ori, corner_id);

                    // Edges
                    let edge_old_ori = edge_list_of_values[i].0;
                    let edge_new_ori = match mv.axis {
                        MoveAxis::F | MoveAxis::B => (edge_old_ori == 0) as u8,
                        _ => edge_old_ori,
                    };
                    self.set_edge_slot(
                        edge_mask_idxs[(i + 1) % 4],
                        edge_new_ori,
                        edge_list_of_values[i].1,
                    );
                }
            }
            MoveDirection::CCw => {
                let mut corner_list_of_values = [(0u8, 0u8); 4];
                let mut edge_list_of_values = [(0u8, 0u8); 4];

                // Extract pieces currently at the affected slots
                for i in 0..4 {
                    corner_list_of_values[i] = self.get_corner_slot(corner_mask_idxs[i]);
                    edge_list_of_values[i] = self.get_edge_slot(edge_mask_idxs[i]);
                }

                // Cycle pieces and update orientations
                for i in 0..4 {
                    // Corners
                    let (corner_old_ori, corner_id) = corner_list_of_values[i];

                    let corner_new_ori = if u64::from(corner_old_ori) == corner_ori_swap[0] {
                        corner_ori_swap[1] as u8
                    } else if u64::from(corner_old_ori) == corner_ori_swap[1] {
                        corner_ori_swap[0] as u8
                    } else {
                        corner_old_ori
                    };

                    self.set_corner_slot(corner_mask_idxs[(i + 3) % 4], corner_new_ori, corner_id);

                    // Edges
                    let edge_old_ori = edge_list_of_values[i].0;
                    let edge_new_ori = match mv.axis {
                        MoveAxis::F | MoveAxis::B => (edge_old_ori == 0) as u8,
                        _ => edge_old_ori,
                    };
                    self.set_edge_slot(
                        edge_mask_idxs[(i + 3) % 4],
                        edge_new_ori,
                        edge_list_of_values[i].1,
                    );
                }
            }
            MoveDirection::Dbl => {
                for i in 0..2 {
                    // Corners
                    let (ori1, id1) = self.get_corner_slot(corner_mask_idxs[i]);
                    let (ori2, id2) = self.get_corner_slot(corner_mask_idxs[i + 2]);
                    self.set_corner_slot(corner_mask_idxs[i], ori2, id2);
                    self.set_corner_slot(corner_mask_idxs[i + 2], ori1, id1);

                    // Edges
                    let (ori1, id1) = self.get_edge_slot(edge_mask_idxs[i]);
                    let (ori2, id2) = self.get_edge_slot(edge_mask_idxs[i + 2]);
                    self.set_edge_slot(edge_mask_idxs[i], ori2, id2);
                    self.set_edge_slot(edge_mask_idxs[i + 2], ori1, id1);
                }
            }
        }
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

    /// Sets orientation and piece ID for an edge slot
    fn set_edge_slot(&mut self, i: usize, ori: u8, piece_id: u8) {
        let shift = 5 * i;
        let slot = (u64::from(ori) << 4 | u64::from(piece_id)) << shift;
        self.edges = self.edges & !(0b11111 << shift) | slot;
    }

    /// Sets orientation and piece ID for a corner slot
    fn set_corner_slot(&mut self, i: usize, ori: u8, piece_id: u8) {
        let shift = 5 * i;
        let slot = (u64::from(ori) << 3 | u64::from(piece_id)) << shift;
        self.corners = self.corners & !(0b11111 << shift) | slot;
    }

    /// Returns the color of a specific edge sticker based on orientation
    fn get_edge_color(&self, slot: usize, sticker: usize) -> char {
        let (ori, pos) = self.get_edge_slot(slot);
        Self::EDGE_COLORS[pos as usize][(sticker ^ ori as usize) % 2]
    }

    /// Returns the color of a specific corner sticker, accounting for orientation and parity
    fn get_corner_color(&self, slot: usize, sticker: usize) -> char {
        let (ori, pos) = self.get_corner_slot(slot);

        let slot_parity = (slot % 2) ^ (slot / 4);
        let piece_parity = (pos as usize % 2) ^ (pos as usize / 4);
        let rel_parity = slot_parity ^ piece_parity;

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

    /// Returns representation of the cube
    pub fn net_map(&self) -> String {
        let mut output = String::with_capacity(54); // Cubo with 54 stickers
        let faces = self.get_face_data();

        // Logical order (U, L, F, R, B, D)
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
        f[1][1][0] = self.get_edge_color(5, 0);
        f[1][1][1] = 'G';
        f[1][1][2] = self.get_edge_color(4, 0);
        f[1][2][0] = self.get_corner_color(5, 1);
        f[1][2][1] = self.get_edge_color(9, 1);
        f[1][2][2] = self.get_corner_color(4, 1);

        // R face
        f[2][0][0] = self.get_corner_color(0, 2);
        f[2][0][1] = self.get_edge_color(0, 1);
        f[2][0][2] = self.get_corner_color(3, 2);
        f[2][1][0] = self.get_edge_color(4, 1);
        f[2][1][1] = 'R';
        f[2][1][2] = self.get_edge_color(7, 1);
        f[2][2][0] = self.get_corner_color(4, 2);
        f[2][2][1] = self.get_edge_color(8, 1);
        f[2][2][2] = self.get_corner_color(7, 2);

        // B face
        f[3][0][0] = self.get_corner_color(3, 1);
        f[3][0][1] = self.get_edge_color(3, 1);
        f[3][0][2] = self.get_corner_color(2, 1);
        f[3][1][0] = self.get_edge_color(7, 0);
        f[3][1][1] = 'B';
        f[3][1][2] = self.get_edge_color(6, 0);
        f[3][2][0] = self.get_corner_color(7, 1);
        f[3][2][1] = self.get_edge_color(11, 1);
        f[3][2][2] = self.get_corner_color(6, 1);

        // L face
        f[4][0][0] = self.get_corner_color(2, 2);
        f[4][0][1] = self.get_edge_color(2, 1);
        f[4][0][2] = self.get_corner_color(1, 2);
        f[4][1][0] = self.get_edge_color(6, 1);
        f[4][1][1] = 'O';
        f[4][1][2] = self.get_edge_color(5, 1);
        f[4][2][0] = self.get_corner_color(6, 2);
        f[4][2][1] = self.get_edge_color(10, 1);
        f[4][2][2] = self.get_corner_color(5, 2);

        // D face
        f[5][0][0] = self.get_corner_color(5, 0);
        f[5][0][1] = self.get_edge_color(9, 0);
        f[5][0][2] = self.get_corner_color(4, 0);
        f[5][1][0] = self.get_edge_color(10, 0);
        f[5][1][1] = 'Y';
        f[5][1][2] = self.get_edge_color(8, 0);
        f[5][2][0] = self.get_corner_color(6, 0);
        (&mut f[5][2])[1] = self.get_edge_color(11, 0);
        f[5][2][2] = self.get_corner_color(7, 0);

        f
    }
}

impl fmt::Display for Cube {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let raw_str = self.net_map();
        let raw: Vec<char> = raw_str.chars().collect();

        // Helper para obter uma face específica (9 caracteres)
        let get_face = |i: usize| &raw[(i * 9)..(i * 9 + 9)];

        // UP - 0
        let u = get_face(0);
        for r in 0..3 {
            write!(f, "         ")?; // Indentação para alinhar com o meio
            for c in 0..3 {
                write!(f, "{}", self.format_sticker(u[r * 3 + c]))?;
            }
            writeln!(f)?;
        }

        // LEFT, FRONT, RIGHT, BACK - 1, 2, 3, 4
        let l = get_face(1);
        let front = get_face(2);
        let r_face = get_face(3);
        let b = get_face(4);

        for r in 0..3 {
            // Desenha a linha 'r' de cada face lateral lado a lado
            for c in 0..3 { write!(f, "{}", self.format_sticker(l[r * 3 + c]))?; }
            for c in 0..3 { write!(f, "{}", self.format_sticker(front[r * 3 + c]))?; }
            for c in 0..3 { write!(f, "{}", self.format_sticker(r_face[r * 3 + c]))?; }
            for c in 0..3 { write!(f, "{}", self.format_sticker(b[r * 3 + c]))?; }
            writeln!(f)?;
        }

        // DOWN - 5
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
        cube.apply_move(Scramble::new("R"));
        assert!(!cube.is_solved());
    }

    #[test]
    fn test_move_inverse() {
        let mut cube = Cube::new();
        cube.apply_move(Scramble::new("R R'"));
        assert!(cube.is_solved());

        cube.apply_move(Scramble::new("U U'"));
        assert!(cube.is_solved());

        cube.apply_move(Scramble::new("F F'"));
        assert!(cube.is_solved());

        cube.apply_move(Scramble::new("L L'"));
        assert!(cube.is_solved());

        cube.apply_move(Scramble::new("B B'"));
        assert!(cube.is_solved());

        cube.apply_move(Scramble::new("D D'"));
        assert!(cube.is_solved());
    }

    #[test]
    fn test_double_move() {
        let mut cube = Cube::new();
        cube.apply_move(Scramble::new("R2 R2"));
        assert!(cube.is_solved());

        cube.apply_move(Scramble::new("U2 U2"));
        assert!(cube.is_solved());

        cube.apply_move(Scramble::new("F2 F2"));
        assert!(cube.is_solved());

        cube.apply_move(Scramble::new("L2 L2"));
        assert!(cube.is_solved());

        cube.apply_move(Scramble::new("B2 B2"));
        assert!(cube.is_solved());

        cube.apply_move(Scramble::new("D2 D2"));
        assert!(cube.is_solved());
    }

    #[test]
    fn test_sexy_move_cycle() {
        let mut cube = Cube::new();
        // The "sexy move" (R U R' U') repeated 6 times returns the cube to solved
        for _ in 0..6 {
            cube.apply_move(Scramble::new("R U R' U'"));
        }
        assert!(cube.is_solved());
    }

    #[test]
    fn test_sune_cycle() {
        let mut cube = Cube::new();
        // Sune (R U R' U R U2 R') repeated 6 times returns it to solved (for corners/edges cycle)
        for _ in 0..6 {
            cube.apply_move(Scramble::new("R U R' U R U2 R'"));
        }
        assert!(cube.is_solved());
    }

    #[test]
    fn test_solved_face_colors() {
        let cube = Cube::new();
        let faces = cube.get_face_data();

        // Check centers and some stickers
        assert_eq!(faces[0][1][1], 'W'); // U
        assert_eq!(faces[1][1][1], 'G'); // F
        assert_eq!(faces[2][1][1], 'R'); // R
        assert_eq!(faces[3][1][1], 'B'); // B
        assert_eq!(faces[4][1][1], 'O'); // L
        assert_eq!(faces[5][1][1], 'Y'); // D

        // Check all stickers on U face are White
        for r in 0..3 {
            for c in 0..3 {
                assert_eq!(faces[0][r][c], 'W');
            }
        }
    }
}
