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
            let ori = (self.edges() >> (i * 5 + 4)) & 0b1;
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
            let i_edge_id = ((self.edges() >> (i * 5)) & 0b1111) as u32;

            for j in (0..i).rev() {
                let j_edge_id = ((self.edges() >> (j * 5)) & 0b1111) as u32;
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
            let id = ((self.edges() >> (i * 5)) & 0x0F) as u8;
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
            let id = ((self.edges() >> (i * 5)) & 0x0F) as u8;
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
            let edge_val = (self.edges() >> (i * 5)) & 0x0F;
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
            let ori = (self.corners() >> (i * 5 + 3)) & 0b11;
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
            let i_corner_id = ((self.corners() >> (i * 5)) & 0b111) as u16;

            for j in (0..i).rev() {
                let j_corner_id = ((self.corners() >> (j * 5)) & 0b111) as u16;
                if j_corner_id > i_corner_id {
                    sum += 1;
                }
            }
            coord = (coord + sum) * (i as u16);
        }
        coord
    }

    fn rotate_axis(
        &self,
        next_edge_pos: [u64; 12],
        next_corner_pos: [u64; 8],
        corner_ori_rules: [[u64; 3]; 2],
        edge_ori_mask: u16,
    ) -> Self {
        const CORNER_DIAGONAL_MASK: u8 = 0b01011010;

        let mut new_edges: u64 = 0;
        let mut new_corners: u64 = 0;

        for i in 0..12 {
            let current_edge = (self.edges >> (5 * i)) & 0b11111;
            let current_edge_id = current_edge & 0b1111;
            let future_edge_id = next_edge_pos[current_edge_id as usize];
            let need_swap = ((edge_ori_mask >> i) & 1) != ((edge_ori_mask >> current_edge_id) & 1);
            let new_edge = ((need_swap as u64 ^ ((current_edge >> 4) & 1)) << 4) | future_edge_id;
            new_edges |= new_edge << (5 * next_edge_pos[i as usize]);
        }

        for i in 0..8 {
            let current_corner = (self.corners >> (5 * i)) & 0b11111;
            let current_corner_id = current_corner & 0b111;
            let future_corner_id = next_corner_pos[current_corner_id as usize];
            let diagonal = ((CORNER_DIAGONAL_MASK >> i) & 1
                != (CORNER_DIAGONAL_MASK >> current_corner_id) & 1)
                as usize;
            let new_corner = (corner_ori_rules[diagonal][(current_corner >> 3) as usize] << 3)
                | future_corner_id;
            new_corners |= new_corner << (5 * next_corner_pos[i as usize]);
        }

        KociembaCube(Cube {
            edges: new_edges,
            corners: new_corners,
        })
    }
    pub fn rotate_x(&self) -> Self {
        const NEXT_EDGE_POS: [u64; 12] = [8, 5, 9, 1, 11, 7, 10, 3, 4, 6, 2, 0];
        const NEXT_CORNER_POS: [u64; 8] = [4, 5, 1, 0, 7, 6, 2, 3];
        const CORNER_ORI_RULES: [[u64; 3]; 2] = [[0, 2, 1], [2, 1, 0]];
        const EDGE_ORI_MASK: u16 = 0b111101010101;

        self.rotate_axis(
            NEXT_EDGE_POS,
            NEXT_CORNER_POS,
            CORNER_ORI_RULES,
            EDGE_ORI_MASK,
        )
    }

    pub fn rotate_y(&self) -> Self {
        const NEXT_EDGE_POS: [u64; 12] = [1, 2, 3, 0, 5, 6, 7, 4, 9, 10, 11, 8];
        const NEXT_CORNER_POS: [u64; 8] = [1, 2, 3, 0, 5, 6, 7, 4];
        const CORNER_ORI_RULES: [[u64; 3]; 2] = [[0, 2, 1], [0, 2, 1]];
        const EDGE_ORI_MASK: u16 = 0b111100000000;

        self.rotate_axis(
            NEXT_EDGE_POS,
            NEXT_CORNER_POS,
            CORNER_ORI_RULES,
            EDGE_ORI_MASK,
        )
    }

    pub fn rotate_z(&self) -> Self {
        const NEXT_EDGE_POS: [u64; 12] = [4, 8, 0, 11, 6, 9, 2, 10, 5, 1, 3, 7];
        const NEXT_CORNER_POS: [u64; 8] = [4, 0, 3, 7, 5, 1, 2, 6];
        const CORNER_ORI_RULES: [[u64; 3]; 2] = [[0, 2, 1], [1, 0, 2]];
        const EDGE_ORI_MASK: u16 = 0;

        self.rotate_axis(
            NEXT_EDGE_POS,
            NEXT_CORNER_POS,
            CORNER_ORI_RULES,
            EDGE_ORI_MASK,
        )
    }

    pub fn reflection(&self) -> Self {
        const NEXT_EDGE_POS: [u64; 12] = [2, 1, 0, 3, 6, 5, 4, 7, 9, 8, 11, 10];
        const NEXT_CORNER_POS: [u64; 8] = [1, 0, 3, 2, 5, 4, 7, 6];
        const CORNER_ORI_RULES: [[u64; 3]; 2] = [[0, 1, 2], [0, 1, 2]];
        const EDGE_ORI_MASK: u16 = 0;

        self.rotate_axis(
            NEXT_EDGE_POS,
            NEXT_CORNER_POS,
            CORNER_ORI_RULES,
            EDGE_ORI_MASK,
        )
    }

    pub fn apply_uds_symmetry(&self, i: u8) -> Self {
        match i {
            0 => *self,
            1 => self.rotate_y(),
            2 => self.rotate_y().rotate_y(),
            3 => self.rotate_y().rotate_y().rotate_y(),

            4 => self.rotate_x().rotate_x(),
            5 => self.rotate_x().rotate_x().rotate_y(),
            6 => self.rotate_z().rotate_z(),
            7 => self.rotate_z().rotate_z().rotate_y(),

            8 => self.reflection(),
            9 => self.rotate_y().reflection(),
            10 => self.rotate_y().rotate_y().reflection(),
            11 => self.rotate_y().rotate_y().rotate_y().reflection(),

            12 => self.rotate_x().rotate_x().reflection(),
            13 => self.rotate_x().rotate_x().rotate_y().reflection(),
            14 => self.rotate_z().rotate_z().reflection(),
            15 => self.rotate_z().rotate_z().rotate_y().reflection(),

            _ => self.apply_uds_symmetry(i % 16),
        }
    }

    pub fn get_canonical(&self) -> (u16, u8) {
        let mut min_coord: u16 = self.get_uds_coord();
        let mut best_sym = 0;

        for i in 1..16 {
            let sym_cube = self.apply_uds_symmetry(i);

            let uds = sym_cube.get_uds_coord();
            if uds < min_coord {
                min_coord = uds;
                best_sym = i;
            }
        }
        (min_coord, best_sym)
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
