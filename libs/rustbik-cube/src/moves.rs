use rand::prelude::*;
use std::fmt;

/// Represents the faces of the same axis (different directions)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxisGroup {
    UD, // Up Down
    FB, // Front Back
    RL, // Right Left
}

/// Represents the face or axis around which a move is performed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveAxis {
    U, // Up
    F, // Front
    R, // Right
    B, // Back
    L, // Left
    D, // Down
}

impl MoveAxis {
    /// Returns the group the axis belongs to
    pub fn group(&self) -> AxisGroup {
        match self {
            MoveAxis::U | MoveAxis::D => AxisGroup::UD,
            MoveAxis::F | MoveAxis::B => AxisGroup::FB,
            MoveAxis::R | MoveAxis::L => AxisGroup::RL,
        }
    }
}

/// Represents the direction and count of a single move
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveDirection {
    Clk, // Clockwise
    CCw, // Anti-clockwise
    Dbl, // Double move (180 degrees)
}

/// A single atomic move in a sequence
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SingleMove {
    pub axis: MoveAxis,
    pub dir: MoveDirection,
}

impl SingleMove {
    /// Parses a string representation (e.g., "U", "R'", "F2") into a `SingleMove`
    pub fn new(move_str: &str) -> Option<Self> {
        let mut chars = move_str.chars();
        // Determine the move axis
        let axis = match chars.next() {
            Some('U') => Some(MoveAxis::U),
            Some('F') => Some(MoveAxis::F),
            Some('R') => Some(MoveAxis::R),
            Some('B') => Some(MoveAxis::B),
            Some('L') => Some(MoveAxis::L),
            Some('D') => Some(MoveAxis::D),
            _ => None,
        };

        // Determine the move direction
        let dir = match chars.next() {
            Some('\'') => Some(MoveDirection::CCw),
            Some('2') => Some(MoveDirection::Dbl),
            None => Some(MoveDirection::Clk),
            _ => None,
        };

        if let (Some(axis), Some(dir)) = (axis, dir) {
            Some(SingleMove { axis, dir })
        } else {
            None
        }
    }

    /// Returns the inverted move
    pub fn invert(&self) -> SingleMove {
        let new_dir = match self.dir {
            MoveDirection::Clk => MoveDirection::CCw,
            MoveDirection::CCw => MoveDirection::Clk,
            MoveDirection::Dbl => MoveDirection::Dbl,
        };

        SingleMove {
            axis: self.axis,
            dir: new_dir,
        }
    }
}

/// Precomputed movement data for applying rotations
#[derive(Debug, Clone, Copy)]
pub(crate) struct MoveData {
    pub e_mask: u64,
    pub c_mask: u64,
    pub e_shifts: [(u32, u32); 4], // (src, dst)
    pub c_shifts: [(u32, u32); 4], // (src, dst)
    pub e_flip: bool,
    pub c_ori_lut: [u8; 3],
}

/// Precomputed data for all 18 possible moves
pub(crate) const MOVES_LUT: [MoveData; 18] = {
    let mut lut = [MoveData {
        e_mask: 0,
        c_mask: 0,
        e_shifts: [(0, 0); 4],
        c_shifts: [(0, 0); 4],
        e_flip: false,
        c_ori_lut: [0, 1, 2],
    }; 18];

    macro_rules! fill_axis {
        ($axis:expr, $e:expr, $c:expr, $o:expr, $flip:expr) => {
            let axis_idx = $axis as usize * 3;
            let e_idx: [u32; 4] = [$e[0] * 5, $e[1] * 5, $e[2] * 5, $e[3] * 5];
            let c_idx: [u32; 4] = [$c[0] * 5, $c[1] * 5, $c[2] * 5, $c[3] * 5];
            let e_mask = (0x1F << e_idx[0]) | (0x1F << e_idx[1]) | (0x1F << e_idx[2]) | (0x1F << e_idx[3]);
            let c_mask = (0x1F << c_idx[0]) | (0x1F << c_idx[1]) | (0x1F << c_idx[2]) | (0x1F << c_idx[3]);

            let c_ori_lut = if $o[0] == 1 && $o[1] == 2 { [0, 2, 1] }
                           else if $o[0] == 0 && $o[1] == 2 { [2, 1, 0] }
                           else { [1, 0, 2] };

            // Clk
            lut[axis_idx] = MoveData {
                e_mask, c_mask, e_flip: $flip, c_ori_lut,
                e_shifts: [(e_idx[0], e_idx[1]), (e_idx[1], e_idx[2]), (e_idx[2], e_idx[3]), (e_idx[3], e_idx[0])],
                c_shifts: [(c_idx[0], c_idx[1]), (c_idx[1], c_idx[2]), (c_idx[2], c_idx[3]), (c_idx[3], c_idx[0])],
            };
            // CCw
            lut[axis_idx + 1] = MoveData {
                e_mask, c_mask, e_flip: $flip, c_ori_lut,
                e_shifts: [(e_idx[0], e_idx[3]), (e_idx[1], e_idx[0]), (e_idx[2], e_idx[1]), (e_idx[3], e_idx[2])],
                c_shifts: [(c_idx[0], c_idx[3]), (c_idx[1], c_idx[0]), (c_idx[2], c_idx[1]), (c_idx[3], c_idx[2])],
            };
            // Dbl
            lut[axis_idx + 2] = MoveData {
                e_mask, c_mask, e_flip: false, c_ori_lut: [0, 1, 2],
                e_shifts: [(e_idx[0], e_idx[2]), (e_idx[1], e_idx[3]), (e_idx[2], e_idx[0]), (e_idx[3], e_idx[1])],
                c_shifts: [(c_idx[0], c_idx[2]), (c_idx[1], c_idx[3]), (c_idx[2], c_idx[0]), (c_idx[3], c_idx[1])],
            };
        }
    }

    fill_axis!(MoveAxis::U, [0, 1, 2, 3], [0, 1, 2, 3], [1, 2], false);
    fill_axis!(MoveAxis::F, [1, 8, 5, 9], [0, 4, 5, 1], [0, 2], true);
    fill_axis!(MoveAxis::R, [0, 11, 4, 8], [0, 3, 7, 4], [0, 1], false);
    fill_axis!(MoveAxis::B, [3, 10, 7, 11], [2, 6, 7, 3], [0, 2], true);
    fill_axis!(MoveAxis::L, [2, 9, 6, 10], [1, 5, 6, 2], [0, 1], false);
    fill_axis!(MoveAxis::D, [4, 7, 6, 5], [4, 7, 6, 5], [1, 2], false);

    lut
};

impl SingleMove {
    pub(crate) fn get_data(&self) -> &'static MoveData {
        &MOVES_LUT[self.axis as usize * 3 + self.dir as usize]
    }

    /// Converts the move to its standard string representation
    pub fn to_string(&self) -> String {
        let axis = match self.axis {
            MoveAxis::U => 'U',
            MoveAxis::F => 'F',
            MoveAxis::R => 'R',
            MoveAxis::B => 'B',
            MoveAxis::L => 'L',
            MoveAxis::D => 'D',
        };

        let dir = match self.dir {
            MoveDirection::CCw => "'",
            MoveDirection::Dbl => "2",
            MoveDirection::Clk => "",
        };

        format!("{}{}", axis, dir)
    }

    /// Helper to get the number of quarter turns (1-3)
    fn quarter_turns(&self) -> i8 {
        match self.dir {
            MoveDirection::Clk => 1,
            MoveDirection::Dbl => 2,
            MoveDirection::CCw => 3,
        }
    }

    /// Constructs a SingleMove from a number of quarter turns (1, 2, or 3)
    fn from_turns(axis: MoveAxis, turns: i8) -> Option<Self> {
        match turns.rem_euclid(4) {
            0 => None,
            1 => Some(Self {
                axis,
                dir: MoveDirection::Clk,
            }),
            2 => Some(Self {
                axis,
                dir: MoveDirection::Dbl,
            }),
            3 => Some(Self {
                axis,
                dir: MoveDirection::CCw,
            }),
            _ => unreachable!(),
        }
    }

    /// Combines two moves on the same axis into a single one (e.g., R + R = R2)
    pub fn combine(a: SingleMove, b: SingleMove) -> Option<Self> {
        if a.axis != b.axis {
            return None;
        }

        let total = a.quarter_turns() + b.quarter_turns();
        Self::from_turns(a.axis, total)
    }
}

/// A list of moves parsed from a string (e.g., "R U R' U'")
pub struct Scramble {
    pub move_list: Vec<SingleMove>,
}

impl Scramble {
    /// Parses a space-separated string of moves into a `Scramble`
    pub fn new(move_string: &str) -> Scramble {
        let mut list: Vec<SingleMove> = vec![];
        for mv_str in move_string.split_whitespace() {
            if let Some(mv) = SingleMove::new(mv_str) {
                list.push(mv);
            }
        }
        Self { move_list: list }
    }

    /// Creates a Scramble from a list of moves, applying "stitching" optimizations:
    /// 1. Combines consecutive moves on the same axis (e.g., R + R -> R2, R + R' -> Cancel)
    /// 2. Combines commuting moves across an opposite face (e.g., U D U -> U2 D)
    pub fn from_moves(moves: Vec<SingleMove>) -> Self {
        let mut list: Vec<SingleMove> = Vec::with_capacity(moves.len());

        for new_move in moves {
            if list.is_empty() {
                list.push(new_move);
                continue;
            }

            // Optimization 1: Combine consecutive moves on the same axis
            let last = list.last().copied().unwrap();
            if last.axis == new_move.axis {
                list.pop();
                if let Some(combined) = SingleMove::combine(last, new_move) {
                    list.push(combined);
                }
                continue;
            }

            // Optimization 2: Combine commuting moves (e.g., U D U -> U2 D)
            // If the last two moves were 'a' and 'b', and 'a' has the same axis as 'new_move'
            // while 'b' is in the same axis group (opposite face), we combine 'a' and 'new_move'
            if list.len() >= 2 {
                let a = list[list.len() - 2];
                let b = list[list.len() - 1];

                if a.axis == new_move.axis && a.axis.group() == b.axis.group() {
                    list.remove(list.len() - 2);
                    if let Some(combined) = SingleMove::combine(a, new_move) {
                        list.insert(list.len() - 1, combined);
                    }
                    continue;
                }
            }

            list.push(new_move);
        }

        Self { move_list: list }
    }

    /// Generates a random scramble of a given length
    /// It ensures the scramble is "clean" by:
    /// 1. Not having two consecutive moves on the same axis (they are merged)
    /// 2. Avoiding redundant triples on the same axis group (e.g., "U D U" is simplified to "U2 D")
    pub fn random(size: usize) -> Self {
        let mut rng = rand::rng();
        let mut list: Vec<SingleMove> = Vec::with_capacity(size);

        while list.len() < size {
            // Select a random axis and direction
            let axis = match rng.random_range(0..6) {
                0 => MoveAxis::U,
                1 => MoveAxis::F,
                2 => MoveAxis::R,
                3 => MoveAxis::B,
                4 => MoveAxis::L,
                _ => MoveAxis::D,
            };

            let dir = match rng.random_range(0..3) {
                0 => MoveDirection::Clk,
                1 => MoveDirection::CCw,
                _ => MoveDirection::Dbl,
            };

            let new_move = SingleMove { axis, dir };

            // Optimization 1: Combine consecutive moves on the same axis (e.g., R + R -> R2)
            if let Some(last) = list.last().copied() {
                if last.axis == new_move.axis {
                    list.pop();

                    if let Some(combined) = SingleMove::combine(last, new_move) {
                        list.push(combined);
                    }

                    continue;
                }
            }

            // Optimization 2: Avoid redundant sequences in the same axis group
            // For example, "U D U" is redundant because U and D commute
            // If the last two moves were 'a' and 'b', and 'a' has the same axis as 'new_move'
            // while 'b' is in the same axis group (opposite face), the sequence is redundant
            if list.len() >= 2 {
                let a = list[list.len() - 2];
                let b = list[list.len() - 1];

                if a.axis == new_move.axis && a.axis.group() == b.axis.group() {
                    continue;
                }
            }

            list.push(new_move);
        }

        Self { move_list: list }
    }

    /// Returns an iterator over the moves in the list
    pub fn iter(&self) -> std::slice::Iter<'_, SingleMove> {
        self.move_list.iter()
    }

    /// Returns the list size
    pub fn len(&self) -> usize {
        self.move_list.len()
    }

    /// Converts the scramble to its standard string representation
    pub fn to_str(&self) -> String {
        let mut scramble_string = String::new();
        for mv in &self.move_list {
            // Map the axis enum to its character
            let axis = match mv.axis {
                MoveAxis::U => 'U',
                MoveAxis::F => 'F',
                MoveAxis::R => 'R',
                MoveAxis::B => 'B',
                MoveAxis::L => 'L',
                MoveAxis::D => 'D',
            };

            // Map the move direction to its notation
            let dir = match mv.dir {
                MoveDirection::CCw => "\' ",
                MoveDirection::Dbl => "2 ",
                MoveDirection::Clk => " ",
            };

            scramble_string.push(axis);
            scramble_string.push_str(dir);
        }

        scramble_string
    }
}

impl fmt::Display for Scramble {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Standard display implementation calling to_str()
        write!(f, "{}", self.to_str())
    }
}

/// Standard list of all 18 possible atomic Rubik's cube moves
/// Covers all 6 faces, each with 3 possible directions (Clockwise, Anti-clockwise, Double)
pub const MOVE_LIST: [SingleMove; 18] = [
    SingleMove {
        axis: MoveAxis::U,
        dir: MoveDirection::Clk,
    },
    SingleMove {
        axis: MoveAxis::U,
        dir: MoveDirection::CCw,
    },
    SingleMove {
        axis: MoveAxis::U,
        dir: MoveDirection::Dbl,
    },
    SingleMove {
        axis: MoveAxis::F,
        dir: MoveDirection::Clk,
    },
    SingleMove {
        axis: MoveAxis::F,
        dir: MoveDirection::CCw,
    },
    SingleMove {
        axis: MoveAxis::F,
        dir: MoveDirection::Dbl,
    },
    SingleMove {
        axis: MoveAxis::R,
        dir: MoveDirection::Clk,
    },
    SingleMove {
        axis: MoveAxis::R,
        dir: MoveDirection::CCw,
    },
    SingleMove {
        axis: MoveAxis::R,
        dir: MoveDirection::Dbl,
    },
    SingleMove {
        axis: MoveAxis::B,
        dir: MoveDirection::Clk,
    },
    SingleMove {
        axis: MoveAxis::B,
        dir: MoveDirection::CCw,
    },
    SingleMove {
        axis: MoveAxis::B,
        dir: MoveDirection::Dbl,
    },
    SingleMove {
        axis: MoveAxis::L,
        dir: MoveDirection::Clk,
    },
    SingleMove {
        axis: MoveAxis::L,
        dir: MoveDirection::CCw,
    },
    SingleMove {
        axis: MoveAxis::L,
        dir: MoveDirection::Dbl,
    },
    SingleMove {
        axis: MoveAxis::D,
        dir: MoveDirection::Clk,
    },
    SingleMove {
        axis: MoveAxis::D,
        dir: MoveDirection::CCw,
    },
    SingleMove {
        axis: MoveAxis::D,
        dir: MoveDirection::Dbl,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_parsing() {
        let moves = Scramble::new("R U L' D2 B F'");
        assert_eq!(moves.move_list.len(), 6);

        assert_eq!(
            moves.move_list[0],
            SingleMove {
                axis: MoveAxis::R,
                dir: MoveDirection::Clk
            }
        );
        assert_eq!(
            moves.move_list[1],
            SingleMove {
                axis: MoveAxis::U,
                dir: MoveDirection::Clk
            }
        );
        assert_eq!(
            moves.move_list[2],
            SingleMove {
                axis: MoveAxis::L,
                dir: MoveDirection::CCw
            }
        );
        assert_eq!(
            moves.move_list[3],
            SingleMove {
                axis: MoveAxis::D,
                dir: MoveDirection::Dbl
            }
        );
        assert_eq!(
            moves.move_list[4],
            SingleMove {
                axis: MoveAxis::B,
                dir: MoveDirection::Clk
            }
        );
        assert_eq!(
            moves.move_list[5],
            SingleMove {
                axis: MoveAxis::F,
                dir: MoveDirection::CCw
            }
        );
    }
    #[test]
    fn test_empty_parsing() {
        let moves = Scramble::new("");
        assert_eq!(moves.move_list.len(), 0);

        let moves = Scramble::new("   ");
        assert_eq!(moves.move_list.len(), 0);
    }

    #[test]
    fn test_invalid_parsing() {
        let moves = Scramble::new("X Y Z");
        assert_eq!(moves.move_list.len(), 0);
    }

    #[test]
    fn test_single_move_invert() {
        let m1 = SingleMove::new("U").unwrap();
        assert_eq!(m1.invert().dir, MoveDirection::CCw);

        let m2 = SingleMove::new("R'").unwrap();
        assert_eq!(m2.invert().dir, MoveDirection::Clk);

        let m3 = SingleMove::new("F2").unwrap();
        assert_eq!(m3.invert().dir, MoveDirection::Dbl);
    }

    #[test]
    fn test_single_move_combine() {
        let m1 = SingleMove::new("R").unwrap();
        let m2 = SingleMove::new("R").unwrap();
        let combined = SingleMove::combine(m1, m2).unwrap();
        assert_eq!(combined.dir, MoveDirection::Dbl);

        let m3 = SingleMove::new("R2").unwrap();
        let m4 = SingleMove::new("R").unwrap();
        let combined2 = SingleMove::combine(m3, m4).unwrap();
        assert_eq!(combined2.dir, MoveDirection::CCw);
    }
}
