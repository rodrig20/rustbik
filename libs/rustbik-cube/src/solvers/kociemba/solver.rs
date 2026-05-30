use std::fs;

use super::{G1_MOVE_LIST, KociembaCube, LOW_MEMORY, TABLE_DIR};
use crate::moves::{MoveAxis, SingleMove};
use crate::{Cube, MOVE_LIST};

/// Performs IDA* search to reach G1 state (phase 1) in low-memory mode
/// This version calculates state coordinates and consults pruning tables on every step
/// but does not use large precomputed move tables for state transitions
fn search_g1_low_memory(
    cube: &mut KociembaCube,
    depth: usize,
    limit: usize,
    path: &mut Vec<usize>,
    eo_table: &[u8],
    co_table: &[u8],
    uds_table: &[u8],
    last_axis: Option<MoveAxis>,
) -> bool {
    // Combine heuristics from three pruning tables (EO, CO, UDS slice)
    // The maximum of these heuristics is a lower bound on the remaining depth
    let h = *[
        eo_table[cube.get_eo_coord() as usize],
        co_table[cube.get_co_coord() as usize],
        uds_table[cube.get_uds_coord() as usize],
    ]
    .iter()
    .max()
    .unwrap() as usize;

    // Standard IDA* pruning: if current depth + estimated remaining depth > limit, backtrack
    if depth + h > limit {
        return false;
    }
    // Phase 1 goal reached (G1 state)
    if h == 0 {
        return true;
    }

    // Try all 18 standard moves
    for i in 0..18 {
        let mv = &MOVE_LIST[i];
        // Optimization: avoid consecutive moves on the same axis (e.g., R R' or R R2)
        if let Some(axis) = last_axis {
            if mv.axis == axis {
                continue;
            }
        }

        path.push(i);
        cube.turn(&mv);

        if search_g1_low_memory(
            cube,
            depth + 1,
            limit,
            path,
            eo_table,
            co_table,
            uds_table,
            Some(mv.axis),
        ) {
            return true;
        }

        // Backtrack: revert the move and pop from path
        cube.turn(&mv.invert());
        path.pop();
    }
    false
}

/// Performs IDA* search to reach G1 state using precomputed move tables
/// This is significantly faster as it avoids full cube state updates,
/// instead operating directly on coordinate indices
fn search_g1(
    eo: usize,
    co: usize,
    uds: usize,
    depth: usize,
    limit: usize,
    path: &mut Vec<usize>,
    eo_table: &[u8],
    co_table: &[u8],
    uds_table: &[u8],
    eo_move: &[[u16; 18]],
    co_move: &[[u16; 18]],
    uds_move: &[[u16; 18]],
    last_axis: Option<MoveAxis>,
) -> bool {
    // Check heuristic (pruning table) to estimate remaining depth
    let h = *[eo_table[eo], co_table[co], uds_table[uds]]
        .iter()
        .max()
        .unwrap() as usize;

    // Prune the branch if it's impossible to reach the goal within the limit
    if depth + h > limit {
        return false;
    }
    // Goal reached
    if h == 0 {
        return true;
    }

    // Try all 18 moves, avoiding consecutive moves on the same axis
    for i in 0..18 {
        let mv = &MOVE_LIST[i];
        if let Some(axis) = last_axis {
            if mv.axis == axis {
                continue;
            }
        }

        path.push(i);
        // Fast state transition using precomputed move tables
        if search_g1(
            eo_move[eo][i] as usize,
            co_move[co][i] as usize,
            uds_move[uds][i] as usize,
            depth + 1,
            limit,
            path,
            eo_table,
            co_table,
            uds_table,
            eo_move,
            co_move,
            uds_move,
            Some(mv.axis),
        ) {
            return true;
        }
        // Backtrack
        path.pop();
    }
    false
}

/// Orchestrates phase 1 (G0 to G1) of Kociemba's algorithm
fn solve_g1(cube: &Cube) -> Option<Vec<SingleMove>> {
    let eo_table = fs::read(format!("{}/eo_table.bin", TABLE_DIR)).ok()?;
    let co_table = fs::read(format!("{}/co_table.bin", TABLE_DIR)).ok()?;
    let uds_table = fs::read(format!("{}/uds_table.bin", TABLE_DIR)).ok()?;

    if LOW_MEMORY {
        let mut new_cube = KociembaCube(*cube);
        for limit in 0..12 {
            let mut path: Vec<usize> = Vec::with_capacity(limit);
            if search_g1_low_memory(
                &mut new_cube,
                0,
                limit,
                &mut path,
                &eo_table,
                &co_table,
                &uds_table,
                None,
            ) {
                return Some(path.iter().map(|&i| MOVE_LIST[i]).collect());
            }
        }
    } else {
        let eo_move_raw = fs::read(format!("{}/eo_move.bin", TABLE_DIR)).ok()?;
        let co_move_raw = fs::read(format!("{}/co_move.bin", TABLE_DIR)).ok()?;
        let uds_move_raw = fs::read(format!("{}/uds_move.bin", TABLE_DIR)).ok()?;

        let eo_move: [[u16; 18]; 2048] =
            unsafe { std::ptr::read(eo_move_raw.as_ptr() as *const _) };
        let co_move: [[u16; 18]; 2187] =
            unsafe { std::ptr::read(co_move_raw.as_ptr() as *const _) };
        let uds_move: [[u16; 18]; 495] =
            unsafe { std::ptr::read(uds_move_raw.as_ptr() as *const _) };

        let kociemba_cube = KociembaCube(*cube);
        let eo = kociemba_cube.get_eo_coord() as usize;
        let co = kociemba_cube.get_co_coord() as usize;
        let es = kociemba_cube.get_uds_coord() as usize;

        for limit in 0..12 {
            let mut path: Vec<usize> = Vec::with_capacity(limit);
            if search_g1(
                eo, co, es, 0, limit, &mut path, &eo_table, &co_table, &uds_table, &eo_move,
                &co_move, &uds_move, None,
            ) {
                return Some(path.iter().map(|&i| MOVE_LIST[i]).collect());
            }
        }
    }
    None
}

/// Performs IDA* search to reach G2 state (phase 2)
fn search_g2_low_memory(
    cube: &mut KociembaCube,
    depth: usize,
    limit: usize,
    path: &mut Vec<usize>,
    ep_no_uds_table: &[u8],
    ep_uds_table: &[u8],
    cp_table: &[u8],
    last_axis: Option<MoveAxis>,
) -> bool {
    let h_cp = cp_table[cube.get_cp_coord() as usize];
    if depth + h_cp as usize > limit {
        return false;
    }

    let h_ep_no_uds = ep_no_uds_table[cube.get_ep_no_uds_coord() as usize];
    if depth + h_ep_no_uds as usize > limit {
        return false;
    }

    let h_ep_uds = ep_uds_table[cube.get_ep_uds_coord() as usize];
    if depth + h_ep_uds as usize > limit {
        return false;
    }

    if cube.is_solved() {
        return true;
    }

    for i in 0..10 {
        let mv = &G1_MOVE_LIST[i];
        if let Some(axis) = last_axis {
            if mv.axis == axis {
                continue;
            }
        }

        path.push(i);
        let mut next_cube = cube.clone();
        next_cube.turn(&mv);

        if search_g2_low_memory(
            &mut next_cube,
            depth + 1,
            limit,
            path,
            ep_no_uds_table,
            ep_uds_table,
            cp_table,
            Some(mv.axis),
        ) {
            return true;
        }

        path.pop();
    }
    false
}

/// Performs IDA* search to reach G2 state using precomputed move tables
fn search_g2(
    cp: usize,
    ep_no_uds: usize,
    ep_uds: usize,
    depth: usize,
    limit: usize,
    path: &mut Vec<usize>,
    cp_table: &[u8],
    ep_no_uds_table: &[u8],
    ep_uds_table: &[u8],
    cp_move: &[[u16; 10]],
    ep_no_uds_move: &[[u16; 10]],
    ep_uds_move: &[[u16; 10]],
    last_axis: Option<MoveAxis>,
) -> bool {
    // Check heuristic from all G2 pruning tables to estimate remaining depth
    let h = *[
        cp_table[cp],
        ep_no_uds_table[ep_no_uds],
        ep_uds_table[ep_uds],
    ]
    .iter()
    .max()
    .unwrap() as usize;

    // Prune the branch if it's impossible to reach the goal within the limit
    if depth + h > limit {
        return false;
    }
    // Goal reached
    if h == 0 {
        return true;
    }

    // Try all 10 allowed moves in phase 2, avoiding consecutive moves on the same axis
    for i in 0..10 {
        let mv = &G1_MOVE_LIST[i];
        if let Some(axis) = last_axis {
            if mv.axis == axis {
                continue;
            }
        }

        path.push(i);
        if search_g2(
            cp_move[cp][i] as usize,
            ep_no_uds_move[ep_no_uds][i] as usize,
            ep_uds_move[ep_uds][i] as usize,
            depth + 1,
            limit,
            path,
            cp_table,
            ep_no_uds_table,
            ep_uds_table,
            cp_move,
            ep_no_uds_move,
            ep_uds_move,
            Some(mv.axis),
        ) {
            return true;
        }
        // Backtrack
        path.pop();
    }
    false
}

/// Orchestrates phase 2 (G1 to solved) of Kociemba's algorithm
fn solve_g2(cube: &Cube) -> Option<Vec<SingleMove>> {
    let ep_no_uds_table = fs::read(format!("{}/ep_no_uds_table.bin", TABLE_DIR)).ok()?;
    let ep_uds_table = fs::read(format!("{}/ep_uds_table.bin", TABLE_DIR)).ok()?;
    let cp_table = fs::read(format!("{}/cp_table.bin", TABLE_DIR)).ok()?;

    if LOW_MEMORY {
        let mut new_cube = KociembaCube(*cube);
        for limit in 0..18 {
            let mut path: Vec<usize> = Vec::with_capacity(limit);
            if search_g2_low_memory(
                &mut new_cube,
                0,
                limit,
                &mut path,
                &ep_no_uds_table,
                &ep_uds_table,
                &cp_table,
                None,
            ) {
                return Some(path.iter().map(|&i| G1_MOVE_LIST[i]).collect());
            }
        }
    } else {
        let cp_move_raw = fs::read(format!("{}/cp_move.bin", TABLE_DIR)).ok()?;
        let ep_no_uds_move_raw = fs::read(format!("{}/ep_no_uds_move.bin", TABLE_DIR)).ok()?;
        let ep_uds_move_raw = fs::read(format!("{}/ep_uds_move.bin", TABLE_DIR)).ok()?;

        let cp_move: [[u16; 10]; 40320] =
            unsafe { std::ptr::read(cp_move_raw.as_ptr() as *const _) };
        let ep_no_uds_move: [[u16; 10]; 40320] =
            unsafe { std::ptr::read(ep_no_uds_move_raw.as_ptr() as *const _) };
        let ep_uds_move: [[u16; 10]; 24] =
            unsafe { std::ptr::read(ep_uds_move_raw.as_ptr() as *const _) };

        let kociemba_cube = KociembaCube(*cube);
        let cp = kociemba_cube.get_cp_coord() as usize;
        let ep_no_uds = kociemba_cube.get_ep_no_uds_coord() as usize;
        let ep_uds = kociemba_cube.get_ep_uds_coord() as usize;

        for limit in 0..18 {
            let mut path: Vec<usize> = Vec::with_capacity(limit);
            if search_g2(
                cp,
                ep_no_uds,
                ep_uds,
                0,
                limit,
                &mut path,
                &cp_table,
                &ep_no_uds_table,
                &ep_uds_table,
                &cp_move,
                &ep_no_uds_move,
                &ep_uds_move,
                None,
            ) {
                return Some(path.iter().map(|&i| G1_MOVE_LIST[i]).collect());
            }
        }
    }
    None
}

/// Solves a Rubik's cube using Kociemba's two-phase algorithm
pub fn solve(cube: &Cube) -> Option<Vec<SingleMove>> {
    let mut solution = Vec::new();
    let mut current_cube = cube.clone();

    // 1st stage: solve to G1
    if let Some(mut moves) = solve_g1(&current_cube) {
        for mv in &moves {
            current_cube.turn(mv);
        }
        solution.append(&mut moves);
    } else {
        return None;
    }

    // 2nd stage: solve from G1 to solved
    if let Some(mut moves) = solve_g2(&current_cube) {
        solution.append(&mut moves);
        Some(solution)
    } else {
        None
    }
}
