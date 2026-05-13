use rand::prelude::*;
use std::fmt;

/// Represents the face or axis around which a move is performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MoveAxis {
    U, // Up
    F, // Front
    R, // Right
    B, // Back
    L, // Left
    D, // Down
}

/// Represents the direction and count of a single move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MoveDirection {
    Clk, // Clockwise
    CCw, // Anti-clockwise
    Dbl, // Double move (180 degrees)
}

/// A single atomic move in a sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SingleMove {
    pub(crate) axis: MoveAxis,
    pub(crate) dir: MoveDirection,
}

impl SingleMove {
    /// Returns the indices and orientation swap data required for this move.
    /// Format: ([edge_indices], [corner_indices], [orientation_swap_values])
    pub(crate) fn mask(&self) -> ([usize; 4], [usize; 4], [u64; 2]) {
        match self.axis {
            MoveAxis::U => ([0, 1, 2, 3], [0, 1, 2, 3], [1, 2]),
            MoveAxis::F => ([1, 4, 9, 5], [0, 4, 5, 1], [0, 2]),
            MoveAxis::R => ([0, 7, 8, 4], [0, 3, 7, 4], [0, 1]),
            MoveAxis::B => ([3, 6, 11, 7], [2, 6, 7, 3], [0, 2]),
            MoveAxis::L => ([2, 5, 10, 6], [1, 5, 6, 2], [0, 1]),
            MoveAxis::D => ([8, 11, 10, 9], [4, 7, 6, 5], [1, 2]),
        }
    }
}

/// A list of moves parsed from a string (e.g., "R U R' U'")
pub struct Scramble {
    pub(crate) move_list: Vec<SingleMove>,
}

impl Scramble {
    /// Parses a space-separated string of moves into a `MoveList`
    pub fn new(move_string: &str) -> Scramble {
        let mut list: Vec<SingleMove> = vec![];
        for mv in move_string.split(" ") {
            if mv.is_empty() {
                continue;
            }
            let mut mv_chars = mv.chars();
            let axis = match mv_chars.next().unwrap_or(' ') {
                'U' => Some(MoveAxis::U),
                'F' => Some(MoveAxis::F),
                'R' => Some(MoveAxis::R),
                'B' => Some(MoveAxis::B),
                'L' => Some(MoveAxis::L),
                'D' => Some(MoveAxis::D),
                _ => None,
            };

            let dir = match mv_chars.next() {
                Some('\'') => Some(MoveDirection::CCw),
                Some('2') => Some(MoveDirection::Dbl),
                None => Some(MoveDirection::Clk),
                _ => None,
            };

            if let (Some(axis), Some(dir)) = (axis, dir) {
                list.push(SingleMove { axis, dir });
            }
        }
        Self { move_list: list }
    }

    pub fn random(size: usize) -> Self{
        let mut list: Vec<SingleMove> = vec![];
        let mut rng = rand::rng();
        for _ in 0..size {
            let axis = match rng.random_range(0..6) {
                0 => MoveAxis::U,
                1 => MoveAxis::F,
                2 => MoveAxis::R,
                3 => MoveAxis::B,
                4 => MoveAxis::L,
                _ => MoveAxis::D,
            };

            let dir = match rng.random_range(0..3) {
                0 => MoveDirection::CCw,
                1 => MoveDirection::Dbl,
                _ => MoveDirection::Clk,
            };
            list.push(SingleMove { axis, dir });
        }

        Self { move_list: list }
    }

    /// Returns an iterator over the moves in the list
    pub(crate) fn iter(&self) -> std::slice::Iter<'_, SingleMove> {
        self.move_list.iter()
    }

    /// Returns the list size
    pub fn len(&self) -> usize {
        self.move_list.len()
    }

    pub fn to_str(&self) -> String{
        let mut scramble_string = String::new();
        for mv in &self.move_list {
            let axis = match mv.axis {
                MoveAxis::U => 'U',
                MoveAxis::F => 'F',
                MoveAxis::R => 'R',
                MoveAxis::B => 'B',
                MoveAxis::L => 'L',
                MoveAxis::D => 'D',
            };

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
        // Chamamos o net_map e escrevemos o resultado no "buffer" do formatador
        write!(f, "{}", self.to_str())
    }
}

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
}
