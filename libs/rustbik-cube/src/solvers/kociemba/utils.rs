use crate::Cube;
use crate::moves::{MoveAxis, MoveDirection, Scramble, SingleMove};
use std::ops::{Deref, DerefMut};

/// Specialized wrapper for Kociemba coordinate calculations
#[derive(Clone, Copy)]
pub struct KociembaCube(pub Cube);

impl Deref for KociembaCube {
    type Target = Cube;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for KociembaCube {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Delegates constructors to the underlying Cube struct
macro_rules! delegate_args {
    ($($name:ident ($($arg:ident : $type:ty),*)),*) => {
        impl KociembaCube {
            $(
                #[allow(dead_code)]
                pub fn $name($($arg: $type),*) -> Self {
                    Self(Cube::$name($($arg),*))
                }
            )*
        }
    };
}

// Delegates Cube constructors to KociembaCube using the delegate_args macro
delegate_args!(
    new(),
    new_random(size: usize),
    new_with(move_list: &Scramble),
    new_from_minimal(representation: u128)
);

impl KociembaCube {
    /// Calculates the edge orientation coordinate (0-2047)
    pub fn get_eo_coord(&self) -> u16 {
        let mut coord = 0_u16;
        // 12th edge orientation is determined by parity (sum is always even)
        // We only need 11 bits to uniquely represent EO
        for i in 0..11 {
            let ori = (self.0.edges >> (i * 5 + 4)) & 0b1;
            coord |= (ori as u16) << i;
        }
        coord
    }

    #[allow(dead_code)]
    /// Calculates the full edge permutation coordinate using Lehmer code
    pub fn get_ep_coord(&self) -> u32 {
        let mut coord: u32 = 0;

        // Calculates a unique rank for the 12-edge permutation using Lehmer code
        // (Factorial Number System)
        for i in (1..12).rev() {
            let mut sum: u32 = 0;
            let i_edge_id = ((self.0.edges >> (i * 5)) & 0b1111) as u32;

            for j in (0..i).rev() {
                let j_edge_id = ((self.0.edges >> (j * 5)) & 0b1111) as u32;
                if j_edge_id > i_edge_id {
                    sum += 1;
                }
            }
            coord = (coord + sum) * (i as u32);
        }
        coord
    }

    /// Calculates the permutation coordinate for the 8 outer edges
    pub fn get_ep_no_uds_coord(&self) -> u16 {
        let mut perm = [0u8; 8];
        let mut count = 0;

        // Collect only the 8 edges not in the UDS slice
        for i in 0..12 {
            let id = ((self.0.edges >> (i * 5)) & 0x0F) as u8;
            if id < 8 {
                perm[count] = id;
                count += 1;
            }
        }
        // Apply Lehmer code on normalized IDs (0-7) to get rank
        let mut coord: u16 = 0;
        for i in (1..8).rev() {
            let mut sum: u16 = 0;
            for j in (0..i).rev() {
                if perm[j] > perm[i] {
                    sum += 1;
                }
            }
            coord = (coord + sum) * (i as u16);
        }
        coord
    }

    /// Calculates the permutation coordinate for the 4 slice edges
    pub fn get_ep_uds_coord(&self) -> u16 {
        let mut perm = [0u8; 4];
        let mut count = 0;

        // Collect only the 4 slice edges and normalize (8-11 -> 0-3)
        for i in 0..12 {
            let id = ((self.0.edges >> (i * 5)) & 0x0F) as u8;
            if id >= 8 {
                perm[count] = id - 8;
                count += 1;
            }
        }
        // Lehmer code on normalized IDs (0-3) to uniquely identify permutation
        let mut coord: u16 = 0;
        for i in (1..4).rev() {
            let mut sum: u16 = 0;
            for j in (0..i).rev() {
                if perm[j] > perm[i] {
                    sum += 1;
                }
            }
            coord = (coord + sum) * (i as u16);
        }
        coord
    }

    /// Calculates the UDS slice combination coordinate (0-494)
    pub fn get_uds_coord(&self) -> u16 {
        let slice_ids = [8, 9, 10, 11];
        let mut occupied = [false; 12];

        // Identify positions of the 4 slice edges in the cube
        for i in 0..12 {
            let edge_val = (self.0.edges >> (i * 5)) & 0x0F;
            if slice_ids.contains(&(edge_val as usize)) {
                occupied[i] = true;
            }
        }
        let mut s = 0;
        let mut k = -1;

        // Calculate lexicographical rank for 12C4 slice combinations
        for n in 0..12 {
            if (&occupied)[n] {
                k += 1
            } else {
                s += n_cr(n as i32, k);
            }
        }
        s as u16
    }

    /// Calculates the corner orientation coordinate in base 3 (0-2186)
    pub fn get_co_coord(&self) -> u16 {
        let mut coord = 0_u16;
        let mut multiplier = 1;

        // 8th corner orientation is determined by the sum of orientations modulo 3
        for i in 0..7 {
            // Extract orientation (bits 3-4) and calculate base-3 representation
            let ori = (self.0.corners >> (i * 5 + 3)) & 0b11;
            coord += ori as u16 * multiplier;
            multiplier *= 3;
        }
        coord
    }

    /// Calculates the corner permutation coordinate (0-40319)
    pub fn get_cp_coord(&self) -> u16 {
        let mut coord: u16 = 0;

        // Rank the corner permutation using Lehmer code on 8 pieces
        for i in (1..8).rev() {
            let mut sum: u16 = 0;
            let i_corner_id = ((self.0.corners >> (i * 5)) & 0b111) as u16;

            for j in (0..i).rev() {
                let j_corner_id = ((self.0.corners >> (j * 5)) & 0b111) as u16;
                if j_corner_id > i_corner_id {
                    sum += 1;
                }
            }
            coord = (coord + sum) * (i as u16);
        }
        coord
    }
}

/// Calculates the mathematical combination nCr
fn n_cr(n: i32, k: i32) -> usize {
    if k < 0 || k > n {
        0
    } else {
        let mut res = 1;
        let mut k = k;
        // Optimize using symmetry property: nCr(n, k) == nCr(n, n-k)
        if k > n - k {
            k = n - k;
        }
        // Iteratively calculate the combination to avoid overflow
        for i in 0..k {
            res = res * (n - i) as usize / (i + 1) as usize;
        }
        res
    }
}

/// Subset of moves allowed in Group G1 for the Kociemba solver
pub(crate) const G1_MOVE_LIST: [SingleMove; 10] = [
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
        dir: MoveDirection::Dbl,
    },
    SingleMove {
        axis: MoveAxis::R,
        dir: MoveDirection::Dbl,
    },
    SingleMove {
        axis: MoveAxis::B,
        dir: MoveDirection::Dbl,
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
    fn test_coordinates_solved_state() {
        let cube = KociembaCube::new();
        assert_eq!(cube.get_eo_coord(), 0);
        assert_eq!(cube.get_ep_coord(), 0);
        assert_eq!(cube.get_ep_no_uds_coord(), 0);
        assert_eq!(cube.get_ep_uds_coord(), 0);
        assert_eq!(cube.get_uds_coord(), 0);
        assert_eq!(cube.get_co_coord(), 0);
        assert_eq!(cube.get_cp_coord(), 0);
    }

    #[test]
    fn test_coordinates_after_move() {
        let mut cube = KociembaCube::new();
        cube.turn(&SingleMove::new("R").unwrap());
        // An R move does NOT change edge orientation (EO should remain 0)
        assert_eq!(cube.get_eo_coord(), 0);
        // But it DOES change corner orientation
        assert!(cube.get_co_coord() != 0);
    }
}
