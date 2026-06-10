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
        for i in (0..7).rev() {
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

    /// Creates a cube from an edge orientation coordinate (0-2047)
    /// Preserves UDS edges (slots 8-11) — they stay in canonical position with orientation 0.
    pub fn from_eo(eo: u16) -> Self {
        let mut edges: u64 = 0;
        let mut parity = 0;

        // Reconstruct edge orientation for the first 11 edges
        for i in 0..11 {
            let ori = (eo >> i) & 1;
            parity ^= ori;
            edges |= ((ori as u64) << 4 | (i as u64)) << (i * 5);
        }
        // The 12th edge orientation is derived from parity to ensure the total is even
        edges |= ((parity as u64) << 4 | 11) << (11 * 5);

        Self(Cube {
            edges,
            corners: Cube::new().corners,
        })
    }

    /// Creates a cube from a corner orientation coordinate (0-2186)
    pub fn from_co(co: u16) -> Self {
        const CORNER_MASK: u8 = 0b01011010;
        let mut corners: u64 = 0;
        let mut temp_co = co;
        let mut sum_ori = 0;

        // Reconstruct corner orientations for the first 7 corners
        for i in (0..7).rev() {
            let ori = (temp_co % 3) as u8;
            temp_co /= 3;

            let real_ori = if (CORNER_MASK >> i) & 1 == 1 {
                match ori {
                    1 => 2,   
                    2 => 1,   
                    _ => ori 
                }
            } else {
                ori
            };
            sum_ori += real_ori;
            corners |= ((ori as u64) << 3 | (i as u64)) << (i * 5);
        }

        // The 8th corner orientation ensures the sum of all orientations is a multiple of 3
        let last_ori = (3 - (sum_ori % 3)) % 3;
        corners |= ((last_ori as u64) << 3 | 7) << (7 * 5);

        let mut cube = Cube::new();
        cube.corners = corners;
        Self(cube)
    }

    /// Creates a cube from a UDS slice combination coordinate (0-494)
    pub fn from_uds_ori(uds: u16) -> Self {
        let mut occupied = [false; 12];
        let mut s = uds;
        let mut k = 4;

        // Convert lexicographical index back to slice edge positions using nCr
        for n in (0..12).rev() {
            let ncr = n_cr(n as i32, (k - 1) as i32) as u16;
            if k > 0 && s >= ncr {
                s -= ncr;
            } else if k > 0 {
                occupied[n as usize] = true;
                k -= 1;
                if k == 0 {
                    break;
                }
            }
        }

        let mut edges: u64 = 0;
        let slice_pieces = [8, 9, 10, 11];
        let other_pieces = [0, 1, 2, 3, 4, 5, 6, 7];
        let mut s_idx = 0;
        let mut o_idx = 0;

        // Populate edges based on calculated slice edge positions
        for i in 0..12 {
            let piece_id = if occupied[i] {
                let id = slice_pieces[s_idx];
                s_idx += 1;
                id
            } else {
                let id = other_pieces[o_idx];
                o_idx += 1;
                id
            };
            edges |= (piece_id as u64) << (i * 5);
        }

        let mut cube = Cube::new();
        cube.edges = edges;
        Self(cube)
    }

    /// Creates a cube from an edge orientation coordinate (0-2047) and a UDS slice coordinate (0-494)
    pub fn from_eo_uds(eo: u16, uds: u16) -> Self {
        let mut occupied = [false; 12];
        let mut s = uds;
        let mut k = 4;

        // Reconstruct UDS slice edge positions
        for n in (0..12).rev() {
            let ncr = n_cr(n as i32, (k - 1) as i32) as u16;
            if k > 0 && s >= ncr {
                s -= ncr;
            } else if k > 0 {
                occupied[n as usize] = true;
                k -= 1;
                if k == 0 {
                    break;
                }
            }
        }

        let mut edges: u64 = 0;
        let slice_pieces = [8, 9, 10, 11];
        let other_pieces = [0, 1, 2, 3, 4, 5, 6, 7];
        let mut s_idx = 0;
        let mut o_idx = 0;

        let mut parity = 0;
        // Populate edges with combined orientation and slice positions
        for i in 0..12 {
            let piece_id = if occupied[i] {
                let id = slice_pieces[s_idx];
                s_idx += 1;
                id
            } else {
                let id = other_pieces[o_idx];
                o_idx += 1;
                id
            };

            let ori = if i < 11 {
                let o = (eo >> i) & 1;
                parity ^= o;
                o
            } else {
                parity
            };

            edges |= ((ori as u64) << 4 | (piece_id as u64)) << (i * 5);
        }

        let mut cube = Cube::new();
        cube.edges = edges;
        Self(cube)
    }

    /// Creates a cube from a permutation coordinate of the 8 non-slice edges (0-40319)
    pub fn from_ep_no_uds(ep: u16) -> Self {
        let mut sums = [0u8; 8];
        let mut temp = ep;
        for i in 1..8 {
            sums[i] = (temp % (i as u16 + 1)) as u8;
            temp /= i as u16 + 1;
        }

        let mut available = [0u8, 1, 2, 3, 4, 5, 6, 7];
        let mut perm = [0u8; 8];
        let mut current_len = 8;
        for i in (1..8).rev() {
            let index = current_len - 1 - sums[i] as usize;
            perm[i] = available[index];
            for k in index..current_len - 1 {
                available[k] = available[k + 1];
            }
            current_len -= 1;
        }
        perm[0] = available[0];

        let mut edges: u64 = 0;
        // Place the 8 permuted edges in the first 8 slots
        for i in 0..8 {
            edges |= (perm[i] as u64) << (i * 5);
        }
        // Place the 4 slice edges in the last 4 slots in their canonical positions
        for i in 8..12 {
            edges |= (i as u64) << (i * 5);
        }

        Self(Cube {
            edges,
            corners: Cube::new().corners,
        })
    }

    /// Creates a cube from a full edge permutation coordinate (0-12!-1)
    pub fn from_ep12(ep: u32) -> Self {
        let mut sums = [0u8; 12];
        let mut temp = ep;
        for i in 1..12 {
            sums[i] = (temp % (i as u32 + 1)) as u8;
            temp /= i as u32 + 1;
        }

        let mut available = [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        let mut perm = [0u8; 12];
        let mut current_len = 12;
        for i in (1..12).rev() {
            let index = current_len - 1 - sums[i] as usize;
            perm[i] = available[index];
            for k in index..current_len - 1 {
                available[k] = available[k + 1];
            }
            current_len -= 1;
        }
        perm[0] = available[0];

        let mut edges: u64 = 0;
        for i in 0..12 {
            edges |= (perm[i] as u64) << (i * 5);
        }

        Self(Cube {
            edges,
            corners: Cube::new().corners,
        })
    }

    /// Creates a cube from a corner permutation coordinate (0-40319)
    pub fn from_cp(cp: u16) -> Self {
        let mut sums = [0u8; 8];
        let mut temp = cp;
        for i in 1..8 {
            sums[i] = (temp % (i as u16 + 1)) as u8;
            temp /= i as u16 + 1;
        }

        let mut available = [0u8, 1, 2, 3, 4, 5, 6, 7];
        let mut perm = [0u8; 8];
        let mut current_len = 8;
        for i in (1..8).rev() {
            let index = current_len - 1 - sums[i] as usize;
            perm[i] = available[index];
            for k in index..current_len - 1 {
                available[k] = available[k + 1];
            }
            current_len -= 1;
        }
        perm[0] = available[0];

        let mut corners: u64 = 0;
        for i in 0..8 {
            corners |= (perm[i] as u64) << (i * 5);
        }

        Self(Cube {
            edges: Cube::new().edges,
            corners,
        })
    }

    /// Creates a cube from a permutation coordinate of the 4 slice edges (0-23)
    pub fn from_uds_perm(uds_perm: u16) -> Self {
        let mut sums = [0u8; 4];
        let mut temp = uds_perm;
        for i in 1..4 {
            sums[i] = (temp % (i as u16 + 1)) as u8;
            temp /= i as u16 + 1;
        }

        let mut available = [8u8, 9, 10, 11];
        let mut perm = [0u8; 4];
        let mut current_len = 4;
        for i in (1..4).rev() {
            let index = current_len - 1 - sums[i] as usize;
            perm[i] = available[index];
            for k in index..current_len - 1 {
                available[k] = available[k + 1];
            }
            current_len -= 1;
        }
        perm[0] = available[0];

        let mut edges: u64 = 0;
        // Place the 8 non-slice edges in their canonical positions
        for i in 0..8 {
            edges |= (i as u64) << (i * 5);
        }
        // Place the 4 permuted slice edges in slots 8-11
        for i in 0..4 {
            edges |= (perm[i] as u64) << ((i + 8) * 5);
        }

        Self(Cube {
            edges,
            corners: Cube::new().corners,
        })
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

    #[test]
    fn test_from_eo_roundtrip() {
        for eo in 0..2048 {
            let cube = KociembaCube::from_eo(eo);
            assert_eq!(
                cube.get_eo_coord(),
                eo,
                "Failed roundtrip for EO coordinate: {}",
                eo
            );
        }
    }

    #[test]
    fn test_from_co_roundtrip() {
        for co in 0..2187 {
            let cube = KociembaCube::from_co(co);
            assert_eq!(
                cube.get_co_coord(),
                co,
                "Failed roundtrip for CO coordinate: {}",
                co
            );
        }
    }

    #[test]
    fn test_from_uds_roundtrip() {
        for uds in 0..495 {
            let cube = KociembaCube::from_uds_ori(uds);
            assert_eq!(
                cube.get_uds_coord(),
                uds,
                "Failed roundtrip for UDS coordinate: {}",
                uds
            );
        }
    }

    #[test]
    fn test_from_eo_uds_roundtrip() {
        for eo in 0..2048 {
            for uds in 0..495 {
                let cube = KociembaCube::from_eo_uds(eo, uds);
                assert_eq!(
                    cube.get_eo_coord(),
                    eo,
                    "Failed roundtrip EO in eo_uds: eo={}, uds={}",
                    eo,
                    uds
                );
                assert_eq!(
                    cube.get_uds_coord(),
                    uds,
                    "Failed roundtrip UDS in eo_uds: eo={}, uds={}",
                    eo,
                    uds
                );
            }
        }
    }

    #[test]
    fn test_from_cp_roundtrip() {
        for cp in 0..40320 {
            let cube = KociembaCube::from_cp(cp);
            assert_eq!(
                cube.get_cp_coord(),
                cp,
                "Failed roundtrip for CP coordinate: {}",
                cp
            );
        }
    }

    #[test]
    fn test_from_ep_no_uds_roundtrip() {
        for ep in 0..40320 {
            let cube = KociembaCube::from_ep_no_uds(ep as u16);
            assert_eq!(
                cube.get_ep_no_uds_coord(),
                ep as u16,
                "Failed roundtrip for EP_NO_UDS coordinate: {}",
                ep
            );
        }
    }

    #[test]
    fn test_from_ep12_roundtrip() {
        for ep in (0..479001600).step_by(1003) {
            let cube = KociembaCube::from_ep12(ep);
            assert_eq!(
                cube.get_ep_coord(),
                ep,
                "Failed roundtrip for EP coordinate: {}",
                ep
            );
        }
    }

    #[test]
    fn test_from_uds_perm_roundtrip() {
        for uds_perm in 0..24 {
            let cube = KociembaCube::from_uds_perm(uds_perm);
            assert_eq!(
                cube.get_ep_uds_coord(),
                uds_perm,
                "Failed roundtrip for UDS perm coordinate: {}",
                uds_perm
            );
        }
    }
}
