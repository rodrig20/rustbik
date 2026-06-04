use bytemuck::cast_slice;
use std::sync::OnceLock;
use std::time::Instant;
use std::{fs, usize};

use super::{G1_MOVE_LIST, KociembaCube, TABLE_DIR};
use crate::moves::{MoveAxis, SingleMove};
use crate::{Cube, MOVE_LIST};

struct G2Frame {
    state: (usize, usize, usize),
    move_idx: usize,
    depth: usize,
    last_axis: Option<MoveAxis>,
    last_last_axis: Option<MoveAxis>,
    last_move_idx: usize,
}

struct G2Solver;

impl G2Solver {
    fn ep_no_uds_table() -> &'static [u8] {
        static DATA: OnceLock<Vec<u8>> = OnceLock::new();
        DATA.get_or_init(|| {
            fs::read(format!("{}/ep_no_uds_table.bin", TABLE_DIR))
                .expect("Failed to read ep_no_uds_table.bin")
        })
    }

    fn ep_uds_table() -> &'static [u8] {
        static DATA: OnceLock<Vec<u8>> = OnceLock::new();
        DATA.get_or_init(|| {
            fs::read(format!("{}/ep_uds_table.bin", TABLE_DIR))
                .expect("Failed to read ep_uds_table.bin")
        })
    }

    fn cp_table() -> &'static [u8] {
        static DATA: OnceLock<Vec<u8>> = OnceLock::new();
        DATA.get_or_init(|| {
            fs::read(format!("{}/cp_table.bin", TABLE_DIR)).expect("Failed to read cp_table.bin")
        })
    }

    fn ep_table() -> &'static [u8] {
        static DATA: OnceLock<Vec<u8>> = OnceLock::new();
        DATA.get_or_init(|| {
            fs::read(format!("{}/ep_table.bin", TABLE_DIR))
                .expect("Failed to read ep_table.bin")
        })
    }

    fn cp_uds_table() -> &'static [u8] {
        static DATA: OnceLock<Vec<u8>> = OnceLock::new();
        DATA.get_or_init(|| {
            fs::read(format!("{}/cp_uds_table.bin", TABLE_DIR)).expect("Failed to read cp_uds_table.bin")
        })
    }

    fn cp_move() -> &'static [[u16; 10]] {
        static DATA: OnceLock<Vec<u8>> = OnceLock::new();
        let raw = DATA.get_or_init(|| {
            fs::read(format!("{}/cp_move.bin", TABLE_DIR)).expect("Failed to read cp_move.bin")
        });
        cast_slice(raw)
    }

    fn ep_no_uds_move() -> &'static [[u16; 10]] {
        static DATA: OnceLock<Vec<u8>> = OnceLock::new();
        let raw = DATA.get_or_init(|| {
            fs::read(format!("{}/ep_no_uds_move.bin", TABLE_DIR))
                .expect("Failed to read ep_no_uds_move.bin")
        });
        cast_slice(raw)
    }

    fn ep_uds_move() -> &'static [[u16; 10]] {
        static DATA: OnceLock<Vec<u8>> = OnceLock::new();
        let raw = DATA.get_or_init(|| {
            fs::read(format!("{}/ep_uds_move.bin", TABLE_DIR))
                .expect("Failed to read ep_uds_move.bin")
        });
        cast_slice(raw)
    }

    pub fn solve(cube: &Cube, max_limit: usize) -> Option<Vec<SingleMove>> {
        let kcube = KociembaCube(*cube);
        let (cp0, enu0, eu0) = (
            kcube.get_cp_coord() as usize,
            kcube.get_ep_no_uds_coord() as usize,
            kcube.get_ep_uds_coord() as usize,
        );

        let h0 = *[
            Self::cp_uds_table()[(eu0*40320)+ cp0],
            Self::ep_table()[(eu0*40320)+ enu0],
        ]
        .iter()
        .max()
        .unwrap() as usize;

        if h0 > max_limit {
            return None;
        }

        let mut stack = Vec::new();
        for limit in h0..=max_limit {
            stack.clear();
            stack.push(G2Frame {
                state: (cp0, enu0, eu0),
                move_idx: 0,
                depth: 0,
                last_axis: None,
                last_last_axis: None,
                last_move_idx: 0,
            });

            while let Some(_) = stack.last() {
                let (cp, ep_no_uds, ep_uds, depth, move_idx, last_axis, last_last_axis);
                {
                    let frame = stack.last().unwrap();
                    (cp, ep_no_uds, ep_uds) = frame.state;
                    depth = frame.depth;
                    move_idx = frame.move_idx;
                    last_axis = frame.last_axis;
                    last_last_axis = frame.last_last_axis;
                }

                let h = *[
                    Self::cp_uds_table()[(ep_uds*40320)+cp],
                    Self::ep_table()[(ep_uds*40320)+ep_no_uds],
                ]
                .iter()
                .max()
                .unwrap() as usize;

                // Goal reached
                if h == 0 && move_idx == 0 {
                    return Some(
                        stack
                            .iter()
                            .skip(1)
                            .map(|f| G1_MOVE_LIST[f.last_move_idx])
                            .collect(),
                    );
                }

                // Pruning or exhausted
                if depth + h > limit || move_idx >= 10 {
                    stack.pop();
                    continue;
                }

                // Next move
                let current_move_idx = move_idx;
                if let Some(f) = stack.last_mut() {
                    f.move_idx += 1;
                }

                let mv = &G1_MOVE_LIST[current_move_idx];

                if let Some(la) = last_axis {
                    if mv.axis == la {
                        continue;
                    }
                    if let Some(lla) = last_last_axis {
                        if mv.axis == lla && la.group() == mv.axis.group() {
                            continue;
                        }
                    }
                }

                let next_state = (
                    Self::cp_move()[cp][current_move_idx] as usize,
                    Self::ep_no_uds_move()[ep_no_uds][current_move_idx] as usize,
                    Self::ep_uds_move()[ep_uds][current_move_idx] as usize,
                );

                stack.push(G2Frame {
                    state: next_state,
                    move_idx: 0,
                    depth: depth + 1,
                    last_axis: Some(mv.axis),
                    last_last_axis: last_axis,
                    last_move_idx: current_move_idx,
                });
            }
        }
        None
    }
}

/// Solves a Rubik's cube using Kociemba's two-phase algorithm
pub fn solve(cube: &Cube) -> Option<Vec<SingleMove>> {
    let mut solution = Vec::new();
    let mut current_cube = cube.clone();

    // 1st stage: solve to G1
    if let Some(mut moves) = G1Solver::solve(&current_cube) {
        for mv in &moves {
            current_cube.turn(mv);
        }
        solution.append(&mut moves);
    } else {
        return None;
    }

    // 2nd stage: solve from G1 to solved (Phase 2 usually doesn't exceed 18 moves)
    if let Some(mut moves) = G2Solver::solve(&current_cube, 18) {
        solution.append(&mut moves);
        Some(solution)
    } else {
        None
    }
}

pub fn solve_max_moves(cube: &Cube, max_moves: usize) -> Option<Vec<SingleMove>> {
    println!("Starting Kociemba solver (max moves: {})...", max_moves);
    let total_start = Instant::now();

    // Reuse a single G1Solver across depth limits
    let mut g1_solver = G1Solver::new(cube, 0);
    for g1_limit in 0..=max_moves {
        let depth_start = Instant::now();
        println!("Searching at G1 depth {}...", g1_limit);

        g1_solver.reset(g1_limit);
        let mut solutions_at_this_depth = 0;

        for path_indices in &mut g1_solver {
            solutions_at_this_depth += 1;
            let mut solution = Vec::with_capacity(max_moves);
            let mut current_cube = cube.clone();

            for &mv_idx in &path_indices {
                let mv = MOVE_LIST[mv_idx];
                current_cube.turn(&mv);
                solution.push(mv);
            }

            // Phase 2: Solve from G1 to solved state
            // Intelligent limit: Phase 2 only needs to find a solution that fits within remaining moves
            let remaining_limit = max_moves.saturating_sub(solution.len());
            if let Some(mut g2_moves) = G2Solver::solve(&current_cube, remaining_limit) {
                solution.append(&mut g2_moves);
                let duration = total_start.elapsed();
                println!(
                    "Solution found! Length: {} (Total time: {:?})",
                    solution.len(),
                    duration
                );
                return Some(solution);
            }
        }

        let depth_duration = depth_start.elapsed();
        println!(
            "G1 Depth {} finished in {:?} (found {} G1 candidates)",
            g1_limit, depth_duration, solutions_at_this_depth
        );
    }

    println!(
        "No solution found within {} moves. (Total time: {:?})",
        max_moves,
        total_start.elapsed()
    );
    None
}

type MoveTable = [[u16; 18]];

struct G1Frame {
    state: (usize, usize, usize),
    move_idx: usize,
    depth: usize,
    last_axis: Option<MoveAxis>,
    last_last_axis: Option<MoveAxis>,
    last_move_idx: usize,
}

pub struct G1Solver {
    stack: Vec<G1Frame>,
    limit: usize,
    initial_state: (usize, usize, usize),
}

impl G1Solver {
    pub fn new(cube: &Cube, limit: usize) -> Self {
        let kociemba_cube = KociembaCube(*cube);
        let state = (
            kociemba_cube.get_eo_coord() as usize,
            kociemba_cube.get_co_coord() as usize,
            kociemba_cube.get_uds_coord() as usize,
        );

        Self {
            stack: vec![G1Frame {
                state,
                move_idx: 0,
                depth: 0,
                last_axis: None,
                last_last_axis: None,
                last_move_idx: 0,
            }],
            limit,
            initial_state: state,
        }
    }

    // Pruning Tables (Always loaded via OnceLock)
    fn eo_table() -> &'static [u8] {
        static DATA: OnceLock<Vec<u8>> = OnceLock::new();
        DATA.get_or_init(|| {
            fs::read(format!("{}/eo_table.bin", TABLE_DIR)).expect("Failed to read eo_table.bin")
        })
    }

    fn co_table() -> &'static [u8] {
        static DATA: OnceLock<Vec<u8>> = OnceLock::new();
        DATA.get_or_init(|| {
            fs::read(format!("{}/co_table.bin", TABLE_DIR)).expect("Failed to read co_table.bin")
        })
    }

    fn uds_table() -> &'static [u8] {
        static DATA: OnceLock<Vec<u8>> = OnceLock::new();
        DATA.get_or_init(|| {
            fs::read(format!("{}/uds_table.bin", TABLE_DIR)).expect("Failed to read uds_table.bin")
        })
    }

    fn eo_uds_table() -> &'static [u8] {
        static DATA: OnceLock<Vec<u8>> = OnceLock::new();
        DATA.get_or_init(|| {
            fs::read(format!("{}/eo_uds_table.bin", TABLE_DIR))
                .expect("Failed to read eo_uds_table.bin")
        })
    }

    fn co_uds_table() -> &'static [u8] {
        static DATA: OnceLock<Vec<u8>> = OnceLock::new();
        DATA.get_or_init(|| {
            fs::read(format!("{}/co_uds_table.bin", TABLE_DIR))
                .expect("Failed to read co_uds_table.bin")
        })
    }

    /// Resets the solver to search at a new depth limit, reusing the stack allocation
    pub fn reset(&mut self, limit: usize) {
        self.limit = limit;
        self.stack.clear();
        self.stack.push(G1Frame {
            state: self.initial_state,
            move_idx: 0,
            depth: 0,
            last_axis: None,
            last_last_axis: None,
            last_move_idx: 0,
        });
    }

    // Move Tables
    fn eo_move() -> &'static MoveTable {
        static DATA: OnceLock<Vec<u8>> = OnceLock::new();
        let raw = DATA.get_or_init(|| {
            fs::read(format!("{}/eo_move.bin", TABLE_DIR)).expect("Failed to read eo_move.bin")
        });
        cast_slice(raw)
    }

    fn co_move() -> &'static MoveTable {
        static DATA: OnceLock<Vec<u8>> = OnceLock::new();
        let raw = DATA.get_or_init(|| {
            fs::read(format!("{}/co_move.bin", TABLE_DIR)).expect("Failed to read co_move.bin")
        });
        cast_slice(raw)
    }

    fn uds_move() -> &'static MoveTable {
        static DATA: OnceLock<Vec<u8>> = OnceLock::new();
        let raw = DATA.get_or_init(|| {
            fs::read(format!("{}/uds_move.bin", TABLE_DIR)).expect("Failed to read uds_move.bin")
        });
        cast_slice(raw)
    }

    /// Conveniece method to solve Phase 1 in up to 12 moves
    pub fn solve(cube: &Cube) -> Option<Vec<SingleMove>> {
        let mut solver = Self::new(cube, 0);
        for limit in 0..12 {
            solver.reset(limit);
            let mut result = None;
            while let Some(path) = solver.next() {
                result = Some(path);
            }
            if let Some(path_indices) = result {
                return Some(path_indices.iter().map(|&i| MOVE_LIST[i]).collect());
            }
        }
        None
    }
}

impl Iterator for G1Solver {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (eo, co, uds, depth, move_idx, last_axis, last_last_axis);

            if let Some(frame) = self.stack.last() {
                (eo, co, uds) = frame.state;
                depth = frame.depth;
                move_idx = frame.move_idx;
                last_axis = frame.last_axis;
                last_last_axis = frame.last_last_axis;
            } else {
                return None;
            }

            let h = *[
                Self::eo_uds_table()[(uds * 2048) + eo],
                Self::co_uds_table()[(uds * 2187) + co],
            ]
            .iter()
            .max()
            .unwrap() as usize;

            // Goal reached
            if h == 0 && move_idx == 0 {
                let path = self.stack.iter().skip(1).map(|f| f.last_move_idx).collect();

                if let Some(frame) = self.stack.last_mut() {
                    frame.move_idx = 18; // Mark as exhausted for Phase 1
                }
                return Some(path);
            }

            // Pruning: if current depth + estimated remaining depth > limit, backtrack
            if depth + h > self.limit || move_idx >= 18 {
                self.stack.pop();
                continue;
            }

            // Try the next move
            let current_move_idx = move_idx;
            if let Some(frame) = self.stack.last_mut() {
                frame.move_idx += 1;
            }

            let mv = &MOVE_LIST[current_move_idx];

            // Avoid consecutive moves on the same axis or redundant axial group moves
            let mut skip = false;
            if let Some(la) = last_axis {
                if mv.axis == la {
                    skip = true;
                } else if let Some(lla) = last_last_axis {
                    if mv.axis == lla && la.group() == mv.axis.group() {
                        skip = true;
                    }
                }
            }

            if skip {
                continue;
            }

            // Transition to the next state
            let next_state = (
                Self::eo_move()[eo][current_move_idx] as usize,
                Self::co_move()[co][current_move_idx] as usize,
                Self::uds_move()[uds][current_move_idx] as usize,
            );

            // Push the new frame to the stack
            self.stack.push(G1Frame {
                state: next_state,
                move_idx: 0,
                depth: depth + 1,
                last_axis: Some(mv.axis),
                last_last_axis: last_axis,
                last_move_idx: current_move_idx,
            });
        }
    }
}
